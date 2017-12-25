# noinspection RubyInstanceVariableNamingConvention,RubyParameterNamingConvention,RubyTooManyInstanceVariablesInspection
class Player
  # @return [Integer]
  attr_reader :id

  # @return [TrueClass, FalseClass]
  attr_reader :me

  # @return [TrueClass, FalseClass]
  attr_reader :strategy_crashed

  # @return [Integer]
  attr_reader :score

  # @return [Integer]
  attr_reader :remaining_action_cooldown_ticks

  # @return [Integer]
  attr_reader :remaining_nuclear_strike_cooldown_ticks

  # @return [Integer]
  attr_reader :next_nuclear_strike_vehicle_id

  # @return [Integer]
  attr_reader :next_nuclear_strike_tick_index

  # @return [Float]
  attr_reader :next_nuclear_strike_x

  # @return [Float]
  attr_reader :next_nuclear_strike_y

  # @param [Integer] id
  # @param [TrueClass, FalseClass] me
  # @param [TrueClass, FalseClass] strategy_crashed
  # @param [Integer] score
  # @param [Integer] remaining_action_cooldown_ticks
  # @param [Integer] remaining_nuclear_strike_cooldown_ticks
  # @param [Integer] next_nuclear_strike_vehicle_id
  # @param [Integer] next_nuclear_strike_tick_index
  # @param [Float] next_nuclear_strike_x
  # @param [Float] next_nuclear_strike_y
  def initialize(id, me, strategy_crashed, score, remaining_action_cooldown_ticks,
                 remaining_nuclear_strike_cooldown_ticks, next_nuclear_strike_vehicle_id,
                 next_nuclear_strike_tick_index, next_nuclear_strike_x, next_nuclear_strike_y)
    @id = id
    @me = me
    @strategy_crashed = strategy_crashed
    @score = score
    @remaining_action_cooldown_ticks = remaining_action_cooldown_ticks
    @remaining_nuclear_strike_cooldown_ticks = remaining_nuclear_strike_cooldown_ticks
    @next_nuclear_strike_vehicle_id = next_nuclear_strike_vehicle_id
    @next_nuclear_strike_tick_index = next_nuclear_strike_tick_index
    @next_nuclear_strike_x = next_nuclear_strike_x
    @next_nuclear_strike_y = next_nuclear_strike_y
  end
end