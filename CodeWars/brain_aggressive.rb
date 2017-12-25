require 'forwardable'
require './brain_base'

module Brains
  class Aggressive < Base
    extend Forwardable

    def handle_move
      loc = squadron.location

      (x, y), enemy_count = enemies_by_cell.sort_by do |(x, y), enemy_count|
        Point.from_cell(x, y).distance_to_point(loc)
      end.first
      dest_point = Point.from_cell(x, y)

      squadron.destination = dest_point
      pipeline.select_group(group: squadron.group)
      diff = dest_point - loc
      pipeline.move(x: diff.x, y: diff.y, max_speed: squadron.min_speed)
      wait_for(60.ticks)
    end
  end
end
