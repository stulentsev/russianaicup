require './brain_base'
require './point'

module Brains
  class Follower < Base

    attr_accessor :target_squadron

    def initialize(squadron, strategy, target_squadron)
      super(squadron, strategy)

      @target_squadron = target_squadron
    end

    def handle_move
      loc = squadron.location
      dest = target_squadron.location

      pipeline.select_group(group: squadron.group)
      pipeline.move_by(point: dest - loc)
      wait_for(20.ticks)
    end
  end
end
