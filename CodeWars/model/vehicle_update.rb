# noinspection RubyInstanceVariableNamingConvention,RubyParameterNamingConvention
class VehicleUpdate
  # @return [Integer]
  attr_reader :id

  # @return [Float]
  attr_reader :x

  # @return [Float]
  attr_reader :y

  # @return [Integer]
  attr_reader :durability

  # @return [Integer]
  attr_reader :remaining_attack_cooldown_ticks

  # @return [TrueClass, FalseClass]
  attr_reader :selected

  # @return [Array<Integer>, NilClass]
  attr_reader :groups

  # @param [Integer] id
  # @param [Float] x
  # @param [Float] y
  # @param [Integer] durability
  # @param [Integer] remaining_attack_cooldown_ticks
  # @param [TrueClass, FalseClass] selected
  # @param [Array<Integer>, NilClass] groups
  def initialize(id, x, y, durability, remaining_attack_cooldown_ticks, selected, groups)
    @id = id
    @x = x
    @y = y
    @durability = durability
    @remaining_attack_cooldown_ticks = remaining_attack_cooldown_ticks
    @selected = selected
    @groups = groups
  end
end