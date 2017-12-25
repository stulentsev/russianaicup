require './model/circular_unit'
require './model/vehicle_update'
require './model/vehicle_type'

# noinspection RubyInstanceVariableNamingConvention,RubyParameterNamingConvention,RubyTooManyInstanceVariablesInspection
class Vehicle < CircularUnit
  # @return [Integer]
  attr_reader :player_id

  # @return [Integer]
  attr_reader :durability

  # @return [Integer]
  attr_reader :max_durability

  # @return [Float]
  attr_reader :max_speed

  # @return [Float]
  attr_reader :vision_range

  # @return [Float]
  attr_reader :squared_vision_range

  # @return [Float]
  attr_reader :ground_attack_range

  # @return [Float]
  attr_reader :squared_ground_attack_range

  # @return [Float]
  attr_reader :aerial_attack_range

  # @return [Float]
  attr_reader :squared_aerial_attack_range

  # @return [Integer]
  attr_reader :ground_damage

  # @return [Integer]
  attr_reader :aerial_damage

  # @return [Integer]
  attr_reader :ground_defence

  # @return [Integer]
  attr_reader :aerial_defence

  # @return [Integer]
  attr_reader :attack_cooldown_ticks

  # @return [Integer]
  attr_reader :remaining_attack_cooldown_ticks

  # @return [Integer, NilClass]
  attr_reader :type

  # @return [TrueClass, FalseClass]
  attr_reader :aerial

  # @return [TrueClass, FalseClass]
  attr_reader :selected

  # @return [Array<Integer>, NilClass]
  attr_reader :groups

  # @param [Integer] id
  # @param [Float] x
  # @param [Float] y
  # @param [Float] radius
  # @param [Integer] player_id
  # @param [Integer] durability
  # @param [Integer] max_durability
  # @param [Float] max_speed
  # @param [Float] vision_range
  # @param [Float] squared_vision_range
  # @param [Float] ground_attack_range
  # @param [Float] squared_ground_attack_range
  # @param [Float] aerial_attack_range
  # @param [Float] squared_aerial_attack_range
  # @param [Integer] ground_damage
  # @param [Integer] aerial_damage
  # @param [Integer] ground_defence
  # @param [Integer] aerial_defence
  # @param [Integer] attack_cooldown_ticks
  # @param [Integer] remaining_attack_cooldown_ticks
  # @param [Integer, NilClass] type
  # @param [TrueClass, FalseClass] aerial
  # @param [TrueClass, FalseClass] selected
  # @param [Array<Integer>, NilClass] groups
  def initialize(id, x, y, radius, player_id, durability, max_durability, max_speed, vision_range, squared_vision_range,
                 ground_attack_range, squared_ground_attack_range, aerial_attack_range, squared_aerial_attack_range,
                 ground_damage, aerial_damage, ground_defence, aerial_defence, attack_cooldown_ticks,
                 remaining_attack_cooldown_ticks, type, aerial, selected, groups)
    super(id, x, y, radius)

    @player_id = player_id
    @durability = durability
    @max_durability = max_durability
    @max_speed = max_speed
    @vision_range = vision_range
    @squared_vision_range = squared_vision_range
    @ground_attack_range = ground_attack_range
    @squared_ground_attack_range = squared_ground_attack_range
    @aerial_attack_range = aerial_attack_range
    @squared_aerial_attack_range = squared_aerial_attack_range
    @ground_damage = ground_damage
    @aerial_damage = aerial_damage
    @ground_defence = ground_defence
    @aerial_defence = aerial_defence
    @attack_cooldown_ticks = attack_cooldown_ticks
    @remaining_attack_cooldown_ticks = remaining_attack_cooldown_ticks
    @type = type
    @aerial = aerial
    @selected = selected
    @groups = groups
  end

  # @param [VehicleUpdate] vehicle_update
  def update(vehicle_update)
    if @id != vehicle_update.id
      raise ArgumentError, "Received wrong message [actual=#{vehicle_update.id}, expected=#{@id}]."
    end

    @x = vehicle_update.x
    @y = vehicle_update.y
    @durability = vehicle_update.durability
    @remaining_attack_cooldown_ticks = vehicle_update.remaining_attack_cooldown_ticks
    @selected = vehicle_update.selected
    @groups = vehicle_update.groups
  end
end