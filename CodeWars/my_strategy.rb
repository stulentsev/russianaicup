require 'forwardable'
require 'benchmark'

require './model/game'
require './model/move'
require './model/player'
require './model/world'

require './app_settings'
require './sdk_patches'
require './point'
require './schedulable'
require './ruby_stdlib_patches'
require './command_pipeline'
require './conditional_command_pipeline'
require './query'
require './state'
require './strategy_helpers'
require './cluster'
require './dbscan'
require './factors'
require './jam'

require './brain_base'
require './brain_potential'
require './brain_capture_facility'
require './brain_stalker'
require './brain_null'

require './strategy_tornado'
require './strategy_minefield'
require './strategy_primitive'
require './strategy_nuke_squadrons'
require './strategy_sandwich'
require './strategy_better_sandwich'
require './strategy_airforce'

$localhost = AppSettings.localhost

$ticks_passed   = 0
$time_taken     = 0
$halt_threshold = $localhost ? 1000.0 : 200.0

at_exit do
  puts "Ticks: #{$ticks_passed}, total time taken: #{$time_taken.round(2)} seconds, avg time per tick: #{(($time_taken / $ticks_passed) * 1000).round(2)} milliseconds"
end

class MyStrategy
  def move(me, world, game, move)
    $me    = me
    $world = world
    $game  = game

    if world.tick_index == 0
      srand(game.random_seed)
    end

    secs = Benchmark.realtime {
      if $time_taken < $halt_threshold
        strategy.move(me, world, game, move)
      end
    }
    puts "tick #{world.tick_index}: took #{secs * 1000} milliseconds"# if $localhost
    $ticks_passed += 1
    $time_taken   += secs
  end

  private


  def strategy
    @strategy ||= if $localhost
                    ARGV.empty? ? active_strategy : Strategies.const_get(ARGV.first).new
                  else
                    active_strategy
                  end
  end

  def active_strategy
    if $world.with_facilities?
      Strategies::NukeSquadrons.new
    else
      # Strategies::BetterSandwich.new
      Strategies::Tornado.new
      # Strategies::AirForce.new
    end
  end

end
