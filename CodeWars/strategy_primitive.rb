require './strategy_base'

module Strategies
  class Primitive < Base

    every 300.ticks do
      recalculate_clusters
    end

    # every Cluster::CLUSTER_INFO_UPDATE_FREQUENCY_IN_TICKS do
    #   recalculate_clusters
    # end
    #
    # every Cluster::POSITION_UPDATE_FREQUENCY_IN_TICKS do
    #   update_cluster_positions
    # end
    #
    def handle_tick
      if world.tick_index == 0
        sub_delayed = delayed.create_new
        sub_delayed.when_proc(-> { @far_group_scaled })
        sub_delayed.after(500.ticks)

        delayed.when_all(sub_delayed.delayed_handlers) do
          puts "tick #{world.tick_index}: all conditions triggered!"
        end
      end

      if world.tick_index == 350
        puts "tick #{world.tick_index}: far group finished moving"
        @far_group_scaled = true
      end
    end

    private

    def recalculate_clusters
      enemy_units = board.vehicles.not_mine.to_a
      if clusters.empty?
        new_clusters = DBSCAN(enemy_units, min_points: enemy_units.size.fdiv(50).ceil)
        @clusters    = new_clusters
      end
    end

    def update_cluster_positions
      clusters.each(&:mark_position)

      clusters.each do |c|
        c.compute_direction_and_speed
        puts c
        puts "current location: #{c.center_point}"
        puts "projected location in 30 ticks: #{c.projected_center_point}"
        puts '---'
      end
      puts '=' * 80
    end

    def clusters
      @clusters ||= []
    end
  end
end
