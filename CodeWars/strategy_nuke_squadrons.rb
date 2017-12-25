require './squadron'
require './potential_utils'
require './emitter'
require './visualizable'
require './nuke_evasion'

Dir['brain_*.rb'].each {|brain_file| require_relative brain_file}
# require './brain_stalker'
# require './brain_aggressive'
# require './brain_hunter'
# require './brain_follower'

module Strategies
  class NukeSquadrons < Base
    include PotentialUtils
    include Visualizable
    include NukeEvasion

    every 10.ticks do |tick|
      time = Benchmark.realtime do
        friendly_squadrons.each(&:remove_dead_units)
        friendly_squadrons.reject!(&:dead?)

        update_enemies_by_cell
        clusters.each(&:reset_center_point) if world.tick_index > 10
      end

      puts "tick #{tick}: updating enemies and cleaning dead units: #{(time * 1000).round(2)} milliseconds"
    end

    every 20.ticks do
      friendly_squadrons.select(&:healing).each do |squadron|
        if squadron.good_health?
          squadron.healing = false
        end
      end
    end

    def handle_tick
      if world.tick_index == 0
        form_initial_squadrons
      end

      maybe_recalculate_clusters
      maybe_form_new_squadrons if world.tick_index % 100 == 0
      maybe_setup_production if world.tick_index % 10 == 0

      # show_units_with_vision if $localhost
      show_directions if AppSettings.jam

      @count_down ||= 0
      if @count_down.to_i > 0
        @count_down -= 1
        return
      end

      check_for_nuke

      if nuke_almost_ready?
        recalculate_clusters
      end

      if nuke_ready? && world.tick_index % 5 == 0
        launch_nuke_if_acceptable_enemy_cell_exists?
      end

      if world.tick_index % 500 == 0
        if we_are_winning?
          switch_to_aggressive_mode
        else
          switch_to_stalking_mode
        end
      end

      friendly_squadrons.select(&:alive?).each do |fs|
        if !fs.attempted_compaction? && fs.lost_formation? && world.tick_index % 200 == 0
          fs.compact
        else
          if fs.move
            show_units(board.vehicles) if $localhost

            if $localhost
              world.facilities.each do |facility|
                show_facility(facility)
              end
            end
            break
          end
        end
      end
    end

    def maybe_form_new_squadrons
      world.facilities.factories.mine.each do |facility|
        units = board.vehicles.mine.at_facility(facility).ungrouped
        next if units.count < 9 * 11

        form_squadron_from_facility(facility)

        setup_production(facility)
      end
    end

    def maybe_setup_production
      @production_requests ||= {}

      world.facilities.factories.mine.each do |facility|
        setup_production(facility)
      end
    end

    def form_squadron_from_facility(facility)
      pipeline.select(
        left:         facility.left,
        top:          facility.top,
        right:        facility.left + facility.width,
        bottom:       facility.top + facility.height,
      )
      group = find_available_group
      pipeline.assign(group: group)
      cp = facility.center_point
      pipeline.scale(x: cp.x, y: cp.y, factor: 0.3) do
        delayed.when_stop_moving(board.vehicles.selected.to_a, after: 50.ticks) do
          create_squadron_from_group(group, brain_class: Brains::Hunter)
        end
      end

    end

    def vehicle_type_to_produce(current_vehicle_type)
      good_ground_types = [VehicleType::TANK, VehicleType::IFV]
      good_aerial_types = [VehicleType::HELICOPTER]

      return good_aerial_types.sample if current_vehicle_type.nil? # just captured the factory

      if VehicleType.aerial.include?(current_vehicle_type)
        good_ground_types.sample
      else
        good_aerial_types.sample
      end
    end

    def update_capture_target_marks
      return unless world.with_facilities?

      world.facilities.each do |facility|
        squadron = friendly_squadrons.select(&:alive?).detect {|sq| facility.point_within_bounds?(sq.destination)}
        facility.targeted_for_capturing_by = squadron

        # if facility.mine? && facility.targeted_for_capturing_by.dead?
        #   facility.targeted_for_capturing_by = nil
        # end
      end
    end

    def wandering_groups
      # TODO: move priority - give more actions to faster units
      friendly_squadrons.shuffle #.reverse.take(1)
    end

    def friendlies_by_cell
      @friendlies_by_cell ||= {}
    end

    def enemies_by_cell
      @enemies_by_cell ||= {}
    end

    def enemies_by_type_by_cell
      @enemies_by_type_by_cell ||= {}
    end

    def form_initial_squadrons
      unit_types = [VehicleType::FIGHTER, VehicleType::IFV, VehicleType::HELICOPTER,
                    VehicleType::TANK, VehicleType::ARRV]
      unit_types.each do |unit_type|
        pipeline.select(vehicle_type: unit_type)
        group = find_available_group
        pipeline.assign(group: group)
        cp = board.vehicles.mine.of_type(unit_type).center_point
        pipeline.scale(x: cp.x, y: cp.y, factor: 0.3) do
          delayed.when_stop_moving(board.vehicles.selected.to_a, after: 50.ticks) do
            create_squadron_from_group(group)
          end
        end
      end
    end

    def base_potential_map
      width_in_cells      = world.width / 32
      height_in_cells     = world.height / 32
      @base_potential_map ||= create_map do |x, y|
        effects = [0.0]
        effects.push(map_edge_formula(x)) if x <= MAP_EDGE_EFFECT_RADIUS
        effects.push(map_edge_formula(width_in_cells - x - 1)) if (width_in_cells - x - 1) < MAP_EDGE_EFFECT_RADIUS
        effects.push(map_edge_formula(y)) if y <= MAP_EDGE_EFFECT_RADIUS
        effects.push(map_edge_formula(height_in_cells - y - 1)) if (height_in_cells - y - 1) < MAP_EDGE_EFFECT_RADIUS

        effects.max_by(&:abs)
        # WEATHER_COEFFICIENTS[world.weather_by_cell_x_y[y][x]]   # TODO: use these
      end

    end

    def launch_nuke_if_acceptable_enemy_cell_exists?
      return if world.tick_index < 300
      return if clusters.empty?
      if @enemy_minefield_formation
        puts 'MINEFIELD'
        return
      end

      cluster_size_worth_nuking = enemy_units.count / 10

      strikable_clusters = clusters.select do |cluster|
        next if cluster.size < cluster_size_worth_nuking

        strike_point              = cluster.center_point
        strike_point_within_reach = friendly_squadrons.any? { |s| s.see_point?(strike_point) }

        cluster.assigned_strike_point = strike_point

        strike_point_within_reach
      end

      damages = strikable_clusters.each_with_object({}) do |cluster, memo|
        memo[cluster.id] = {
          enemy: board.enemy_damage_at_point(cluster.assigned_strike_point),
          ally:  board.ally_damage_at_point(cluster.assigned_strike_point),

        }
      end

      return if strikable_clusters.empty?

      strikable_clusters.select! { |c| damages[c.id][:enemy] > damages[c.id][:ally] }
      strikable_clusters.sort_by! { |c| damages[c.id][:ally] - damages[c.id][:enemy] }

      strikable_clusters.each do |cluster|
        strike_point = cluster.assigned_strike_point

        highlighters = board.vehicles.mine.sort_by do |unit|
          striking_distance = unit.effective_vision_range - unit.distance_to_point(strike_point)
          if striking_distance.between?(unit.effective_vision_range * 0.5, unit.effective_vision_range * 0.8)
            striking_distance * 3 # more weight
          else
            striking_distance
          end
        end
        highlighter  = highlighters.last # furthest away
        # highlighter  = highlighters.first # closest to

        dist  = highlighter.distance_to_point(strike_point)
        range = highlighter.effective_vision_range
        if dist < range
          puts '-------------'
          puts "MISSILE AWAY!"
          puts '-------------'
          pipeline.priority do |priority_pipeline|
            priority_pipeline.stop_movement
            priority_pipeline.tactical_nuke(x: strike_point.x, y: strike_point.y, vehicle_id: highlighter.id)
            @count_down = game.tactical_nuclear_strike_delay
          end
          return
        else
          puts "Could have launched the nuke, but out of range of our fighters. (#{dist} vs #{range})"
        end
      end
    end


    def all_cells
      @all_cells ||= begin
        cell_range = (0...32).to_a
        cell_range.product(cell_range)
      end
    end

    def map_edge_formula(cur_distance, safe_distance: MAP_EDGE_EFFECT_RADIUS)
      linear_decay(cur_distance, BASE_MAP_EDGE_ATTRACTION, safe_distance)
    end

    def linear_decay(cur_distance, base_value, effect_radius)
      if cur_distance.between?(0, effect_radius)
        base_value * (1.0 - (cur_distance.fdiv(effect_radius)))
      else
        0.0
      end
    end

    MAP_EDGE_EFFECT_RADIUS   = 4.0
    BASE_MAP_EDGE_ATTRACTION = -150

    def friendly_squadrons
      @friendly_squadrons ||= []
    end

    def find_available_group
      @group_space ||= (1..80).to_a
      (@group_space - friendly_squadrons.map(&:group)).sample
    end

    def enemies_at_cell(x, y)
      enemy_units.at_cell(x, y)
    end

    def update_enemies_by_cell
      @enemies_by_cell = Hash.new(0)
      @enemies_by_type_by_cell = Hash.new { |hash, key| hash[key] = Hash.new(0) }

      enemy_units.each do |unit|
        ckey = [unit.x.to_i / 32, unit.y.to_i / 32]
        @enemies_by_cell[ckey]                    += 1
        @enemies_by_type_by_cell[unit.type][ckey] += 1
      end
    end

    def update_friendlies_by_cell
      @friendlies_by_cell = count_units_by_cell(board.vehicles.mine.aerial)
    end

    def count_units_by_cell(units)
      units.each_with_object(Hash.new(0)) do |v, memo|
        memo[[v.x.to_i / 32, v.y.to_i / 32]] += 1
      end.sort_by{|point, unit_count| -unit_count}
    end

    def enemy_units
      board.vehicles.not_mine
    end

    def lowest_ground_speed
      @lowest_ground_speed ||= [$game.tank_speed, $game.ifv_speed, $game.arrv_speed].min * 0.6
    end

    def lowest_aerial_speed
      @lowest_aerial_speed ||= [$game.fighter_speed, $game.helicopter_speed].min * 0.6
    end

    def clusters
      @clusters ||= []
    end



    WEATHER_COEFFICIENTS = {
      0 => 0,
      1 => -1, # clouds
      2 => -3, # rain
    }

    def create_squadron_from_group(group, brain_class: current_brain_class)
      units = board.vehicles.mine.group(group).to_a
      return if units.empty?
      squadron = Squadron.new(units: units, group: group)

      brain_class = Brains::Hunter if squadron.aerial?
      squadron.brain = create_brain_for(squadron, brain_class: brain_class)
      friendly_squadrons << squadron
    end

    def create_brain_for(squadron, brain_class: current_brain_class)
      brain_class.new(squadron, self)
    end

    def current_brain_class
      @current_brain_class ||= Brains::Stalker
    end

    def setup_production(facility)
      queue = production_request_queue_for(facility)
      if queue.empty?
        # schedule production requests
        production_tick = $world.tick_index + 1

        3.times do
          queue.add(vehicle_type: VehicleType::TANK, at_tick: production_tick)
          production_tick += 11 * $game.tank_production_cost

          queue.add(vehicle_type: VehicleType::ARRV, at_tick: production_tick)
          production_tick += 11 * $game.arrv_production_cost

          queue.add(vehicle_type: VehicleType::IFV, at_tick: production_tick)
          production_tick += 11 * $game.ifv_production_cost
        end
      end

      if queue.next_request_ready?
        vehicle_type = queue.pop

        pipeline.setup_vehicle_production(
          facility_id: facility.id,
          vehicle_type: vehicle_type,
        )
      end

      # if !@production_requests.key?(facility.id) || @production_requests[facility.id] + 200 < world.tick_index
      #   pipeline.setup_vehicle_production(
      #     facility_id: facility.id,
      #     vehicle_type: vehicle_type_to_produce(facility.vehicle_type)
      #   )
      #   @production_requests[facility.id] = world.tick_index
      # end
    end

    def production_request_queue_for(facility)
      @production_request_queues ||= {}
      @production_request_queues[facility.id] ||= ProductionRequestQueue.new
    end

    def we_are_winning?
      return false unless world.with_facilities? # shouldn't even be here without facilities. Playing it safe.

      world.facilities.mine.count.fdiv(world.facilities.count) >= 0.8
    end

    def switch_to_aggressive_mode
      return if @current_brain_class == Brains::Hunter # aggressive already

      @current_brain_class = Brains::Hunter # affects new squadrons
      assign_new_brains
    end

    def assign_new_brains
      friendly_squadrons.each do |squadron|
        next if squadron.aerial? # they're hunters
        squadron.brain = create_brain_for(squadron)
      end
    end

    def maybe_recalculate_clusters
      @cluster_recalculation_cooldown ||= 300
      @cluster_recalculation_cooldown -= 1

      if @cluster_recalculation_cooldown <= 0
        recalculate_clusters
        @cluster_recalculation_cooldown = [enemy_units.count - 200, 80].max
        puts "next recalculation in #{@cluster_recalculation_cooldown}"
      end
    end

    def recalculate_clusters
      enemies_ary = enemy_units.to_a

      clusterizer = DBSCAN::Clusterer.new(enemies_ary, min_points: enemies_ary.size.fdiv(50).ceil, epsilon: 15)
      clusterizer.clusterize!
      @enemy_minefield_formation = clusterizer.clusters[-1].length.fdiv(enemies_ary.length) > 0.7
      new_clusters               = clusterizer.results

      @clusters = new_clusters
    end

    def switch_to_stalking_mode
      return if @current_brain_class == Brains::Stalker # stalking already

      @current_brain_class = Brains::Stalker # affects new squadrons
      assign_new_brains
    end

    class ProductionRequestQueue
      def initialize
        @queue = []
      end

      def empty?
        queue.empty?
      end

      def add(vehicle_type:, at_tick:)
        queue.push(QueueItem.new(vehicle_type, at_tick))
      end

      def next_request_ready?
        return false if queue.empty?

        item = queue.first
        item.tick_index < $world.tick_index
      end

      def pop
        item = queue.shift

        item.vehicle_type
      end

      private

      attr_reader :queue

      QueueItem = Struct.new(:vehicle_type, :tick_index)
    end

  end
end
