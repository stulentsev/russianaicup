require './strategy_base'

module Strategies
  class Minefield < Base

    every 1.tick do
      if nuke_almost_ready?
        recalculate_clusters
        update_cluster_positions
      end
    end

    def handle_tick
      if world.tick_index == 0
        initial_flyout
      end

      maybe_recalculate_clusters

      if world.my_player.next_nuclear_strike_vehicle_id == -1
        if prev_highlighter.to_i > 0
          puts "highlighter unset"
        end
      else
        if prev_highlighter.to_i <= 0
          puts "highlighter set"
        end
      end

      self.prev_highlighter = world.my_player.next_nuclear_strike_vehicle_id

      if world.tick_index % 50 == 0
        if nuke_ready?
          launch_nuke_if_acceptable_target_exists?
        end
      end
    end

    private

    attr_accessor :prev_highlighter

    def initial_flyout
      destination_points = Array.new(5) { Point.new(512 - 200 + rand(400), 512 - 200 + rand(400)) }

      unit_types = VehicleType.all
      unit_types.each_with_index do |unit_type, idx|
        delayed.after((idx * 80).ticks) do
          units = board.vehicles.mine.of_type(unit_type)
          cp = units.center_point
          destination_points.sort_by!{|p| p.distance_to_point(cp) }
          dp = destination_points.shift
          pipeline.select(vehicle_type: unit_type)
          puts "center_point: #{cp}"
          puts "destination point: #{dp}"
          puts "delta: #{dp - cp}"

          pipeline.move_by(point: dp - cp, max_speed: $game.helicopter_speed) do
            delayed.when_stop_moving(units, after: 100) do
              pipeline.select(vehicle_type: unit_type)
              pipeline.scale(x: dp.x, y: dp.y, factor: 10)
            end
          end
        end
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
      new_clusters = DBSCAN(enemies_ary, min_points: enemies_ary.size.fdiv(50).ceil, epsilon: 15)
      @clusters    = new_clusters
    end

    def clusters
      @clusters ||= []
    end

    def enemy_units
      board.vehicles.not_mine
    end

    def update_cluster_positions
      clusters.each do |c|
        c.mark_position
      end
    end

    def launch_nuke_if_acceptable_target_exists?
      clusters.sort_by{|c| c.damage_at_point(c.center_point) }.reverse_each do |cluster|
        cp = cluster.suggested_strike_point
        highlighters = board.vehicles.mine.aerial.reject do |unit|
          unit.effective_vision_range - 5 < unit.distance_to_point(cp)
        end.to_a.sort_by do |unit|
          unit.effective_vision_range - unit.distance_to_point(cp)
        end
        highlighter  = highlighters.last

        next unless highlighter

        dist  = highlighter.distance_to_unit(cp)
        range = highlighter.vision_range
        if dist < range
          puts "Missile away! (squad of #{cluster.size}, #{dist} meters away)"
          pipeline.tactical_nuke(x: cp.x, y: cp.y, vehicle_id: highlighter.id)
          return
        else
          puts "Could have launched the nuke, but out of range of our fighters. (#{dist} vs #{range})"
        end
      end
    end
  end
end
