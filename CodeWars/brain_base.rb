require 'forwardable'
require './visualizable'
require './gnuplot_dumper'

module Brains
  ScheduledAction = Struct.new(:action_type, :cooldown)

  class Base
    extend Forwardable
    include Visualizable

    attr_reader :squadron, :strategy, :id

    def_delegators :strategy,
                   :world, :board, :pipeline

    def_delegators :world,
                   :facilities

    def_delegators :strategy,
                   :enemies_by_cell, :enemies_by_type_by_cell,
                   :enemies_at_cell, :all_cells, :friendly_squadrons, :clusters

    def_delegators :squadron,
                   :aerial?

    def initialize(squadron = nil, strategy = nil)
      @squadron = squadron
      @strategy = strategy
      @id       = SecureRandom.uuid
      initialize_brain
    end

    def initialize_brain

    end

    def deactivate_brain
      switch_brain(Brains::Null)
    end

    def convert_to_stalker
      switch_brain(Brains::Stalker)
    end

    def convert_to_hunter
      switch_brain(Brains::Hunter)
    end

    def move
      return if turn_cooldown?

      handle_move
      true
    end

    def schedule(action_type, cooldown)
      scheduled_actions.push(ScheduledAction.new(action_type, cooldown))
    end

    def scheduled_actions
      @scheduled_actions ||= []
    end

    def turn_cooldown?
      @turn_cooldown ||= 0

      if @turn_cooldown <= 0
        false
      else
        @turn_cooldown -= 1
        true
      end
    end

    def wait_for(ticks)
      @turn_cooldown = ticks
    end

    private

    def switch_brain(brain_class)
      squadron.brain = brain_class.new(squadron, strategy)
    end
  end
end
