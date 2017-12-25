require './squadron'
require './potential_utils'
require './emitter'

module Strategies
  class AirForce < Base
    include PotentialUtils

    TORNADO_CORE_UNIT_GROUPS_OFFSET = 20
    TORNADO_CORE_GROUP              = 15

    HUNTER_GROUPS_OFFSET = 40

    NUKE_EVASION_SCALE_FACTOR = 3.0

    def handle_tick
      if world.tick_index == 0
        form_squadrons
        delayed.after(200.ticks) do
          initial_scaleout
        end
      end

      @count_down ||= 0
      if @count_down.to_i > 0
        @count_down -= 1
        return
      end
      check_for_nuke

      if nuke_ready? && world.tick_index % 20 == 0
        launch_nuke_if_acceptable_enemy_cell_exists?
      end

      if world.tick_index > 300
        move_aerial_units
        move_tornado_core
      end
    end

    private

    def group_units
      UNITS_ALL.each do |unit_type|
        pipeline.select(vehicle_type: unit_type)
        pipeline.assign(group: unit_type + TORNADO_CORE_UNIT_GROUPS_OFFSET)
        pipeline.assign(group: TORNADO_CORE_GROUP)
      end
    end

    def move_aerial_units
      timeout = wandering_groups.size * 15
      if !wandering_groups.empty? && world.tick_index % timeout == 0 && world.tick_index > 20
        # update_friendlies_by_cell
        update_enemies_by_cell unless @enemy_minefield_formation

        wandering_groups.each do |squadron|
          next if squadron.units.select(&:alive?).empty?

          sq_location = squadron.location

          if world.tick_index % 100 == 0 && squadron.lost_formation?
            pipeline.select_group(group: squadron.group)

            # closest_cell_xy = enemies_by_cell.select{|p, enemy_count| enemy_count >= 5 }.sort_by do |(x, y), enemy_count|
            #   Point.from_cell(x, y).distance_to_point(sq_location)
            # end.first.first
            #
            # vector = sq_location - Point.from_cell(*closest_cell_xy)
            #
            # vector = vector * 2.3
            #
            # dest_location = sq_location + vector
            dest_location = sq_location
            pipeline.scale(x: dest_location.x, y: dest_location.y, factor: 0.1)
          else
            dest_point = if @enemy_minefield_formation && !squadron.low_health?
                           target_unit_group = if squadron.unit_type == VehicleType::HELICOPTER
                                                 board.vehicles.not_mine.ground
                                               else
                                                 board.vehicles.not_mine.ground
                                               end
                           target_unit_group.sort_by { |u| u.distance_to_point(squadron.center_point) }.first.center_point
                         else
                           choose_destination_cell_for(squadron)
                         end
            diff_point = dest_point - sq_location
            next if diff_point.x.to_i == 0 && diff_point.y.to_i == 0
            pipeline.select_group(group: squadron.group)
            pipeline.move_by(point: diff_point)
          end
        end
      end
    end

    def wandering_groups
      friendly_squadrons.shuffle #.reverse.take(1)
    end

    def show_units_with_vision
      board.vehicles.each do |v|
        rew.living_unit(
          v.x, v.y, v.radius, v.durability, v.max_durability,
          v.player_id == me.id ? -1 : 1, 0, UnitType.from_vehicle_type(v.type), 0, 0,
          v.selected ? 1 : 0
        )

        color = (v.player_id == me.id) ? Color.green : Color.red
        rew.circle(v.x, v.y, v.effective_vision_range, color.opacity(1), 4)
      end
      rew.end_frame
    end


    attr_reader :friendlies_by_cell, :enemies_by_cell

    def form_squadrons
      split(board.vehicles.fighters.mine, HUNTER_GROUPS_OFFSET + 0)
      split(board.vehicles.helicopters.mine, HUNTER_GROUPS_OFFSET + 4)
    end

    def split(units, group_offset)
      ltx, lty = units.lefttop_point.map(&:round)

      cur_group = group_offset
      offsets_x = [0, 27]
      offsets_y = [0, 27]

      offsets_x.reverse_each do |offset_x|
        offsets_y.reverse_each do |offset_y|
          pipeline.select(
            left:  ltx + offset_x, top: lty + offset_y,
            right: ltx + offset_x + 27, bottom: lty + offset_y + 27
          )
          cg = cur_group
          pipeline.assign(group: cur_group) do
            friendly_squadrons << Squadron.new(units: board.vehicles.mine.selected.to_a, group: cg)
          end

          displacement = 250
          dest_vector  = if ltx == 166 && lty == 166 # right-bottom corner
                           Point.new(displacement - ltx, displacement - lty)
                         elsif ltx == 166 # right column
                           Point.new(displacement - ltx, 0)
                         elsif lty == 166 # bottom row
                           Point.new(0, displacement - lty)
                         else
                           if board.vehicles.mine.aerial.any? { |u| u.y == lty && u.x > ltx + 54 } # squad to the right
                             # move down
                             Point.new(0, displacement - lty)
                           else
                             # move right
                             Point.new(displacement - ltx, 0)
                           end
                         end

          pipeline.scale(x: ltx + offset_x + 13, y: lty + offset_y + 13, factor: 0.1)
          pipeline.move(x: dest_vector.x, y: dest_vector.y)
          cur_group += 1
        end
      end
    end

    def check_for_nuke
      if nuke_incoming?
        emergency_nuke_evasion_maneuver! unless evading_nuke?
      else
        if evading_nuke?
          restore_formation_after_evasive_maneuver
          stop_nuke_evasion
        end
      end
    end

    def emergency_nuke_evasion_maneuver!
      start_nuke_evasion
      pipeline.priority do |priority_pipeline|
        nx = world.opponent_player.next_nuclear_strike_x
        ny = world.opponent_player.next_nuclear_strike_y

        scale_out_from_the_nuke(nx, ny, priority_pipeline)
        @count_down           = $game.tactical_nuclear_strike_delay + 10
        @current_center_point = Point.new(nx, ny)
      end
    end

    def scale_out_from_the_nuke(nx, ny, priority_pipeline)
      nuke_radius      = $game.tactical_nuclear_strike_radius
      selection_radius = nuke_radius * 1.1

      priority_pipeline.select(left:  nx - selection_radius, top: ny - selection_radius,
                               right: nx + selection_radius, bottom: ny + selection_radius)
      priority_pipeline.assign(group: GROUP_NUKE_TARGET)
      priority_pipeline.scale(x: nx, y: ny, factor: NUKE_EVASION_SCALE_FACTOR)
    end

    def restore_formation_after_evasive_maneuver
      pipeline.priority do |priority_pipeline|
        priority_pipeline.select_group(group: GROUP_NUKE_TARGET)
        priority_pipeline.scale(x: @current_center_point.x, y: @current_center_point.y, factor: 1.0 / NUKE_EVASION_SCALE_FACTOR)
        priority_pipeline.disband(group: GROUP_NUKE_TARGET)
        @current_center_point = nil
        @count_down           = 30
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
      return if world.tick_index < 600
      return if @enemy_minefield_formation

      strikable_points = enemies_by_cell.keys.select do |x, y|
        friendly_squadrons.any? { |s| s.see_point?(Point.from_cell(x, y)) }
      end.map { |x, y| Point.from_cell(x, y) }

      nukable_damage_threshold = 700
      strikable_points.sort_by { |strike_point| board.enemy_damage_at_point(strike_point) - board.ally_damage_at_point(strike_point) }.reverse_each do |strike_point|
        next if board.enemy_damage_at_point(strike_point) < nukable_damage_threshold

        board.vehicles.mine.select do |unit|
          striking_distance = unit.effective_vision_range - unit.distance_to_point(strike_point)
          unit.effective_vision_range > unit.distance_to_point(strike_point) &&
            striking_distance.between?(unit.effective_vision_range * 0.5, unit.effective_vision_range * 0.8)
        end.take(3).each do |highlighter|
          dist  = highlighter.distance_to_point(strike_point)
          range = highlighter.effective_vision_range
          if dist < range
            puts '-------------'
            puts "MISSILE AWAY!"
            puts '-------------'
            pipeline.priority do |priority_pipeline|
              priority_pipeline.select
              priority_pipeline.stop_movement
              priority_pipeline.tactical_nuke(x: strike_point.x, y: strike_point.y, vehicle_id: highlighter.id)
              scale_out_from_the_nuke(strike_point.x, strike_point.y, priority_pipeline)
            end
            @count_down = 30
            return
          end
        end
      end
    end

    def choose_destination_cell_for(squadron)
      emitters             = field_emitters_for(squadron)
      sq_location          = squadron.location
      field_value_to_cells = all_cells.each_with_object({}) do |(x, y), result|
        location = Point.from_cell(x, y)
        next unless location.distance_to_point(sq_location) < SQUADRON_MOVE_RADIUS
        combined_field_value = emitters.select { |e| e.within_range_of(location) }.reduce(0) do |memo, emitter|
          memo + emitter.value_at_point(location, squadron: squadron)
        end

        field_value         = (combined_field_value + base_potential_map[x][y]).round(2)
        result[field_value] ||= []
        result[field_value] << location
      end

      chosen_cell = field_value_to_cells.max_by(&:first).last.sample

      squadron.destination = chosen_cell

      chosen_cell
    end

    def all_cells
      @all_cells ||= begin
        cell_range = (0...32).to_a
        cell_range.product(cell_range)
      end
    end

    def field_emitters_for(squadron)
      artificial_field_emitters_for(squadron) +
        enemy_cell_emitters_for(squadron) +
        friendly_squadron_emitters(except: squadron)
    end

    def enemy_cell_emitters_for(squadron)
      # my_health = squadron.total_health
      enemies_by_cell.flat_map do |(x, y), enemy_count|
        enemies = enemies_at_cell(x, y)

        if squadron.easy_prey?(enemies)
          # puts "detected easy prey(type=#{enemies.count_by(&:type)} for #{squadron.unit_type}"
          LinearFalloffEmitter.new(
            location:         Point.from_cell(x, y),
            effect_radius:    150,
            max_effect_value: 100.0
          )
        else # keep distance
          [
            SafeDistanceEmitter.new(
              location:                      Point.from_cell(x, y),
              max_effect_value:              BASE_ENEMY_CELL_ATTRACTION * enemy_count,
              effect_radius:                 ENEMY_CELL_EFFECT_RADIUS,
              preferable_distance:           PREFERABLE_DISTANCE_FROM_ENEMY_CELL,
              preferable_distance_variation: PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION
            )
          ]
        end
      end
    end

    def artificial_field_emitters_for(squadron)
      [].tap do |emitters|
        emitters.push(LinearFalloffEmitter.new(
          location:         Point.new(world.width - 100, world.height - 100),
          effect_radius:    world.width,
          max_effect_value: 100.0
        )) if world.tick_index.between?(0, 1000)

        if squadron.low_health? && board.vehicles.mine.arrvs.count > 10
          puts "LOW HEALTH for squadron #{squadron.id}"
          emitters.push(LinearFalloffEmitter.new(
            location:         board.vehicles.mine.arrvs.center_point,
            effect_radius:    world.width * 2,
            max_effect_value: 30000.0
          ))
        end

        # $world.facilities.control_centers.untaken.each do |cc|
        #   emitters.push(LinearDecayEmitter.new(
        #     location:         cc.location,
        #     effect_radius:    400,
        #     max_effect_value: 30
        #   ))
        # end

      end
    end

    def friendly_squadron_emitters(except:)
      friendly_squadrons.reject { |s| s.id == except.id }.flat_map do |squadron|
        [
          ExponentialFalloffEmitter.new(
            max_effect_value: -500,
            exponent:         25,
            bias:             -10,
            effect_radius:    150,
            location:         squadron.location
          ),
        # LinearFalloffEmitter.new(
        #   location:         squadron.location,
        #   max_effect_value: BASE_FRIENDLY_SQUAD_ATTRACTION,
        #   effect_radius:    FRIENDLY_SQUAD_EFFECT_RADIUS
        # )
        ].tap do |result|
          if squadron.destination
            result.push(
              ExponentialFalloffEmitter.new(
                max_effect_value: -500,
                exponent:         25,
                bias:             -10,
                effect_radius:    150,
                location:         squadron.destination
              )
            )
          end
        end
      end
    end

    def recalculate_clusters
      enemies_ary = enemy_units.to_a

      clusterizer = DBSCAN::Clusterer.new(enemies_ary, min_points: enemies_ary.size.fdiv(50).ceil, epsilon: 10)
      clusterizer.clusterize!
      @enemy_minefield_formation = clusterizer.clusters[-1].length.fdiv(enemies_ary.length) > 0.7
      new_clusters               = clusterizer.results

      @clusters = new_clusters
    end

    def clusters
      @clusters ||= []
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

    SQUADRON_MOVE_RADIUS = 1.5.cells

    MAP_EDGE_EFFECT_RADIUS   = 3.0
    BASE_MAP_EDGE_ATTRACTION = -1.5

    ENEMY_CELL_EFFECT_RADIUS                      = 350.meters
    BASE_ENEMY_CELL_ATTRACTION                    = 1.0
    PREFERABLE_DISTANCE_FROM_ENEMY_CELL           = 90.meters
    PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION = 10.meters

    FRIENDLY_SQUAD_EFFECT_RADIUS   = 3.5.cells
    BASE_FRIENDLY_SQUAD_ATTRACTION = -25.5

    def friendly_squadrons
      @friendly_squadrons ||= []
    end

    def enemies_at_cell(x, y)
      board.vehicles.not_mine.at_cell(x, y)
    end

    def update_enemies_by_cell
      @enemies_by_cell = count_units_by_cell(board.vehicles.not_mine)
    end

    def update_friendlies_by_cell
      @friendlies_by_cell = count_units_by_cell(board.vehicles.mine.aerial)
    end

    def count_units_by_cell(units)
      units.each_with_object(Hash.new(0)) do |v, memo|
        memo[[v.x.to_i / 32, v.y.to_i / 32]] += 1
      end
    end

    def initial_scaleout
      unit_types = UNITS_ALL
      unit_types.each do |unit_type|
        ltx, lty      = board.vehicles.mine.of_type(unit_type).lefttop_point.map(&:round)
        scale_centers = {
          [18, 18]   => [18, 18],
          [18, 92]   => [18, 92 + 27],
          [18, 166]  => [18, 166 + 54],

          [92, 18]   => [92 + 27, 18],
          [92, 92]   => [92 + 27, 92 + 27],
          [92, 166]  => [92 + 27, 166 + 54],

          [166, 18]  => [166 + 54, 18],
          [166, 92]  => [166 + 54, 92 + 27],
          [166, 166] => [166 + 54, 166 + 54],
        }

        x, y             = scale_centers[[ltx, lty]]
        selection_width  = VehicleType.aerial.include?(unit_type) ? 27 : 54
        selection_height = 54
        pipeline.select(left: ltx - 1, top: lty - 1, right: ltx + selection_width + 1, bottom: lty + selection_height + 1)
        pipeline.assign(group: unit_type + TORNADO_CORE_UNIT_GROUPS_OFFSET)
        pipeline.assign(group: TORNADO_CORE_GROUP)
        pipeline.scale(x: x, y: y, factor: 2.4) do
          delayed.after(420.ticks) do
            @stage = :move_tornado_core
          end
        end
      end

    end

    def tornado_actions
      @tornado_actions ||= begin
        initial  = ([:rotate, :shrink] * 7).to_enum
        followup = [:hunt, :hunt, :hunt, :rotate, :shrink].cycle

        initial + followup
      end
    end

    def move_tornado_core
      @tornado_count_down ||= 0
      if @tornado_count_down.to_i > 0
        @tornado_count_down -= 1
        return
      end

      return unless @stage == :move_tornado_core

      my_center = board.vehicles.group(TORNADO_CORE_GROUP).center_point
      return if my_center.x.nan? || my_center.y.nan?

      if @stage == :move_tornado_core
        mx, my          = *my_center
        factor          = 0.95
        angle           = 45
        shrink_period   = 15
        rotation_period = 35
      else
        mx              = 119
        my              = 119
        factor          = 0.83
        angle           = 110
        shrink_period   = 15
        rotation_period = 40
      end

      pipeline.select_group(group: TORNADO_CORE_GROUP)
      tornado_action = tornado_actions.next
      case tornado_action
      when :shrink
        pipeline.scale(x: mx, y: my, factor: factor) do
          @tornado_count_down = shrink_period
        end
      when :rotate
        pipeline.rotate(x: mx, y: my, angle_degrees: angle) do
          @tornado_count_down = rotation_period
        end
      when :hunt
        target_point = if @enemy_minefield_formation
                   Point.new(119, 119)
                 else
                   squad = clusters.sort_by do |squad|
                     squad.distance_to_point(my_center)
                   end.first

                   target = squad || world
                   target.center_point - Point.new(300, 300) # keep distance
                 end

        diff   = target_point - my_center
        pipeline.move_by(x: diff.x, y: diff.y, max_speed: 0.25) do
          @tornado_count_down = 40
        end
      end

      @tornado_count_down = 1000
    end


    def enemy_units
      board.vehicles.not_mine
    end

    def lowest_ground_speed
      @lowest_ground_speed ||= [$game.tank_speed, $game.ifv_speed, $game.arrv_speed].min * 0.6
    end


    WEATHER_COEFFICIENTS = {
      0 => 0,
      1 => -1, # clouds
      2 => -3, # rain
    }
  end
end
