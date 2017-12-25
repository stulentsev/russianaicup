require './strategy_base'

module Strategies
  class Tornado < Base

    every 20.ticks do |tick|
      if tick > 301
        update_cluster_positions
      end
    end

    every 5.ticks do
      clusters.each(&:reset_center_point) if world.tick_index > 10

      if nuke_almost_ready?
        recalculate_clusters
        update_cluster_positions
      end
    end

    every 20.ticks do
      my_center        = board.vehicles.mine.center_point
      total_distance = 0
      cnt = 0
      board.vehicles.mine.each do |unit|
        dist = unit.distance_to_point(my_center)
        total_distance += dist
        cnt += 1
      end

      @tornado_core_radius = total_distance / cnt
    end

    def handle_tick
      if world.tick_index == 0
        initial_scaleout
      end

      if world.tick_index % 5 == 0 && world.tick_index > 400
        @nuke_imminent = world.opponent_player.remaining_nuclear_strike_cooldown_ticks < 150
      end

      check_for_nuke
      maybe_recalculate_clusters

      @count_down ||= 0

      if @enemy_minefield_formation
        minefield_compact_and_hunt
      else
        compact_and_hunt
      end

      @count_down -= 1
    end

    private

    def tornado_rotation_angle
      @rotation_enum ||= [60, 60, 60, 60, -70].cycle
      @rotation_enum.next
    end

    def compact_and_hunt
      return unless @count_down <= 0 && @stage == :compact_and_hunt
      my_center = board.vehicles.mine.center_point

      current_radius = @tornado_core_radius
      compact_radius = 20.0
      nuke_anticipation_radius = @first_nuke_launched ? 150.0 : 60

      target_radius = @nuke_imminent ? nuke_anticipation_radius : compact_radius
      puts "nuke imminent: #{@nuke_imminent}"

      pipeline.select

      if @stage == :compact_and_hunt
        mx, my          = *my_center
        factor          = target_radius / current_radius
        angle           = tornado_rotation_angle
        shrink_period   = @nuke_imminent ? 90 : 50
        rotation_period = @nuke_imminent ? 0 : 20
      else
        mx              = 119
        my              = 119
        factor          = 0.83
        angle           = 110
        shrink_period   = 15
        rotation_period = 40
      end

      if nuke_ready?
        launch_nuke_if_acceptable_target_exists?
      end

      next_action = tornado_actions.next

      if next_action == :shrink && (factor - 1.0).abs > 0.01
        pipeline.scale(x: mx, y: my, factor: factor)
        @count_down = shrink_period
      elsif next_action == :rotate
        pipeline.rotate(x: mx, y: my, angle_degrees: angle)
        @count_down = rotation_period
      elsif next_action == :hunt
        # go to largest group (if have several, pick the closest of them)
        squad = clusters.select(&:alive?).sort_by do |squad|
          [-squad.size, squad.distance_to_point(my_center)]
        end.first

        target = squad || board.vehicles.not_mine.ground.sort_by{|u| u.distance_to_point(my_center)}.first
        pipeline.move_by(point: target.center_point - my_center, max_speed: 0.25)
        @count_down = 20
      end
    end

    def minefield_compact_and_hunt
      return unless @count_down <= 0 && @stage == :compact_and_hunt
      my_center = board.vehicles.mine.center_point

      current_radius = @tornado_core_radius
      compact_radius = 30.0
      nuke_anticipation_radius = @first_nuke_launched ? 350.0 : 80

      target_radius = @nuke_imminent ? nuke_anticipation_radius : compact_radius
      puts "nuke imminent: #{@nuke_imminent}"

      pipeline.select

      if @stage == :compact_and_hunt
        mx, my          = *my_center
        factor          = target_radius / current_radius
        angle           = tornado_rotation_angle
        shrink_period   = @nuke_imminent ? 90 : 50
        rotation_period = @nuke_imminent ? 0 : 20
      else
        mx              = 119
        my              = 119
        factor          = 0.83
        angle           = 110
        shrink_period   = 15
        rotation_period = 40
      end

      if nuke_ready?
        launch_nuke_if_acceptable_target_exists?
      end

      next_action = tornado_actions.next

      if next_action == :shrink && (factor - 1.0).abs > 0.01
        pipeline.scale(x: mx, y: my, factor: factor)
        @count_down = shrink_period
      elsif next_action == :rotate
        pipeline.rotate(x: mx, y: my, angle_degrees: angle)
        @count_down = rotation_period
      elsif next_action == :hunt
        # go to largest group (if have several, pick the closest of them)
        squad = clusters.select(&:alive?).sort_by do |squad|
          [-squad.size, squad.distance_to_point(my_center)]
        end.first

        target = squad ||
          board.vehicles.not_mine.ground.sort_by{|u| u.distance_to_point(my_center)}.first ||
          board.vehicles.not_mine.aerial.sort_by{|u| u.distance_to_point(my_center)}.first ||
          world
        pipeline.move_by(point: target.center_point - my_center, max_speed: 0.25)
        @count_down = 20
      end
    end

    def check_for_nuke
      if nuke_incoming?
        @first_nuke_launched = true
        emergency_nuke_evasion_maneuver! unless evading_nuke?
      else
        if evading_nuke?
          restore_formation_after_evasive_maneuver
          stop_nuke_evasion
        end
      end
    end

    def show_unit_visions
      rew.frame do
        board.vehicles.each do |v|
          rew.living_unit(
            v.x, v.y, v.radius, v.durability, v.max_durability,
            v.player_id == me.id ? -1 : 1, 0, UnitType.from_vehicle_type(v.type), 0, 0,
            v.selected ? 1 : 0
          )

          if v.player_id == me.id
            rew.circle(v.x, v.y, v.effective_vision_range, Color.green.opacity(5), 4)
          end
        end
      end
    end

    def emergency_nuke_evasion_maneuver!
      start_nuke_evasion
      pipeline.priority do |pipeline|
        nx = world.opponent_player.next_nuclear_strike_x
        ny = world.opponent_player.next_nuclear_strike_y

        nuke_radius      = $game.tactical_nuclear_strike_radius
        selection_radius = nuke_radius * 1.5

        pipeline.select(left:  nx - selection_radius, top: ny - selection_radius,
                        right: nx + selection_radius, bottom: ny + selection_radius)
        pipeline.assign(group: GROUP_NUKE_TARGET)
        pipeline.scale(x: nx, y: ny, factor: NUKE_EVASION_SCALE_FACTOR)
        @count_down           = $game.tactical_nuclear_strike_delay + 10
        @current_center_point = board.vehicles.mine.center_point
      end
    end

    def restore_formation_after_evasive_maneuver
      pipeline.priority do |pipeline|
        pipeline.select_group(group: GROUP_NUKE_TARGET)
        pipeline.scale(x: @current_center_point.x, y: @current_center_point.y, factor: 1.0 / NUKE_EVASION_SCALE_FACTOR)
        pipeline.disband(group: GROUP_NUKE_TARGET)
        @current_center_point = nil
        @count_down           = 30
      end
    end

    def launch_nuke_if_acceptable_target_exists?
      return if world.tick_index < 600
      return if clusters.empty?

      my_center        = board.vehicles.mine.center_point
      my_approx_radius = @tornado_core_radius
      close_enough_range        = 300
      cluster_size_worth_nuking = board.vehicles.not_mine.count / 10

      strikable_clusters = clusters.select do |cluster|
        next if cluster.size < cluster_size_worth_nuking

        strike_point = cluster.suggested_strike_point

        strike_point_within_reach = strike_point.distance_to_point(my_center) - my_approx_radius < close_enough_range &&
          strike_point.distance_to_point(my_center) > my_approx_radius

        cluster.assigned_strike_point = strike_point

        strike_point_within_reach
      end

      damages = strikable_clusters.each_with_object({}) do |cluster, memo|
        memo[cluster.id] = {
          enemy: board.enemy_damage_at_point(cluster.assigned_strike_point),
          ally:  board.ally_damage_at_point(cluster.assigned_strike_point),

        }
      end

      strikable_clusters.select! { |c| damages[c.id][:enemy] > damages[c.id][:ally] }
      strikable_clusters.sort_by! { |c| damages[c.id][:ally] - damages[c.id][:enemy] }

      strikable_clusters.each do |cluster|
        strike_point = cluster.assigned_strike_point

        highlighters = board.vehicles.mine.to_a.sort_by do |unit|
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
          pipeline.stop_movement
          pipeline.tactical_nuke(x: strike_point.x, y: strike_point.y, vehicle_id: highlighter.id)
          @count_down = game.tactical_nuclear_strike_delay
          return
        else
          puts "Could have launched the nuke, but out of range of our fighters. (#{dist} vs #{range})"
        end
      end
    end

    def initial_scaleout
      unit_types = UNITS_ALL
      unit_types.each do |unit_type|
        lefttop       = board.vehicles.mine.of_type(unit_type).lefttop_point.map(&:to_i)
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

        x, y = scale_centers[lefttop]
        pipeline.select(vehicle_type: unit_type)
        pipeline.scale(x: x, y: y, factor: 2.4)
      end

      delayed.after(300.ticks) do
        @stage = :compact_and_hunt
      end
    end

    def tornado_actions
      @tornado_actions ||= begin
        initial  = ([:rotate, :shrink] * 7).to_enum
        followup = [:hunt, :hunt, :hunt, :shrink, :rotate,].cycle

        initial + followup
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
      enemies_ary  = enemy_units.to_a

      clusterizer = DBSCAN::Clusterer.new(enemies_ary, min_points: enemies_ary.size.fdiv(50).ceil, epsilon: 10)
      clusterizer.clusterize!
      @enemy_minefield_formation = clusterizer.clusters[-1].length.fdiv(enemies_ary.length) > 0.7
      new_clusters = clusterizer.results

      @clusters    = new_clusters
    end

    def enemy_units
      board.vehicles.not_mine
    end

    def update_cluster_positions
      clusters.each do |c|
        c.mark_position
        c.compute_direction_and_speed
        # puts c
        # puts "current location: #{c.center_point}"
        # puts "projected location in 30 ticks: #{c.projected_center_point}"
        # puts '---'
      end
      # puts '=' * 80
    end

    def clusters
      @clusters ||= []
    end
  end
end
