require 'matrix'

require './brain_base'

module Brains
  class CaptureFacility < Base
    def handle_move

      chosen_facility = select_facility

      if chosen_facility.nil?
        pipeline.select_group(group: squadron.group)
        loc = squadron.location
        cp = world.center_point
        vec = Vector[*(cp - loc)]
        x, y = (vec.normalize * 100).to_a

        squadron.destination = Point[x, y] + loc
        pipeline.move(x: x, y: y, max_speed: squadron.min_speed)
        wait_for(200.ticks)

        return
      end

      chosen_facility.targeted_for_capturing_by = squadron
      squadron.destination = chosen_facility.location
      pipeline.select_group(group: squadron.group)
      pipeline.move_by(
        point: chosen_facility.location - squadron.location,
        max_speed: squadron.min_speed
      )
      wait_for(200.ticks)
    end

    private

    def select_facility
      targeted_facility || closest_available_facility
    end

    def targeted_facility
      facilities.detect{|f| f.targeted_for_capturing_by == squadron }
    end

    def closest_available_facility
      facilities.not_mine.select{|f| f.targeted_for_capturing_by.nil? }.to_a.sort_by do |f|
        f.distance_to_point(squadron.location)
      end.first
    end
  end
end
