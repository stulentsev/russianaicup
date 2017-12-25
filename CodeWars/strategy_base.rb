require './model/game'
require './model/move'
require './model/player'
require './model/world'

require './strategy_helpers'
require './schedulable'
require './visualizable'

if AppSettings.localhost
  require 'pry'
  require './gnuplot_dumper'
end

module Strategies
  class Base
    include StrategyHelpers
    include Schedulable
    include Visualizable

    attr_reader :me, :world, :game, :move

    UNITS_AERIAL  = [VehicleType::FIGHTER, VehicleType::HELICOPTER]
    UNITS_GROUND  = [VehicleType::IFV, VehicleType::TANK]
    UNITS_SUPPORT = [VehicleType::ARRV]
    UNITS_ALL     = [*UNITS_AERIAL, *UNITS_GROUND, *UNITS_SUPPORT]

    NUKE_EVASION_SCALE_FACTOR = 2.0
    GROUP_NUKE_TARGET = 99

    def move(me, world, game, move)
      self.me    = me
      self.world = world
      self.game  = game
      self.move  = move

      jam.begin_post
      rew.start_frame

      # will these ever be dynamic? I think not
      if world.tick_index == 0
        $factors.weather_by_cell_x_y = world.weather_by_cell_x_y
        $factors.terrain_by_cell_x_y = world.terrain_by_cell_x_y
      end

      board.add_new_vehicles(world.new_vehicles)
      board.apply_vehicle_updates(world.vehicle_updates)

      if $localhost
        show_world_state
      end

      run_scheduled_events(world.tick_index)
      delayed.handle_tick
      handle_tick

      if can_run_commands?
        pipeline.run(move)
      end
    ensure
      jam.end_post
      rew.end_frame
    end

    def handle_tick
      fail NotImplementedError
    end

    def can_run_commands?
      pipeline.has_commands? &&
        me.remaining_action_cooldown_ticks == 0 &&
        (!pipeline.soft_limit_reached? || evading_nuke?)
    end

    def pipeline
      @pipeline ||= CommandPipeline.new
    end

    def delayed
      @delayed ||= ConditionalCommandPipeline.new(board: board)
    end

    def board
      @board ||= State.new
    end

    def me=(player)
      @me          = player
      board.player = player
    end

    def world=(world)
      @world         = world
      pipeline.world = world
      board.world    = world
    end

    def game=(game)
      @game         = game
    end

    def move=(move)
      @move = move
    end

    private

    def show_world_state
      n1 = show_nuke(player: $world.my_player, color: Color.orange)
      n2 = show_nuke(player: $world.opponent_player, color: Color.blueish)

      if n1 || n2
        show_units(board.vehicles)
      end
    end

  end
end
