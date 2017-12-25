require './model/action_type'
require './model/vehicle_type'

# noinspection RubyInstanceVariableNamingConvention,RubyTooManyInstanceVariablesInspection
class Move
  # @return [Integer, NilClass]
  attr_accessor :action

  # @return [Integer]
  attr_accessor :group

  # @return [Float]
  attr_accessor :left

  # @return [Float]
  attr_accessor :top

  # @return [Float]
  attr_accessor :right

  # @return [Float]
  attr_accessor :bottom

  # @return [Float]
  attr_accessor :x

  # @return [Float]
  attr_accessor :y

  # @return [Float]
  attr_accessor :angle

  # @return [Float]
  attr_accessor :factor

  # @return [Float]
  attr_accessor :max_speed

  # @return [Float]
  attr_accessor :max_angular_speed

  # @return [Integer, NilClass]
  attr_accessor :vehicle_type

  # @return [Integer]
  attr_accessor :facility_id

  # @return [Integer]
  attr_accessor :vehicle_id

  def initialize
    @action = nil
    @group = 0
    @left = 0.0
    @top = 0.0
    @right = 0.0
    @bottom = 0.0
    @x = 0.0
    @y = 0.0
    @angle = 0.0
    @factor = 0.0
    @max_speed = 0.0
    @max_angular_speed = 0.0
    @vehicle_type = nil
    @facility_id = -1
    @vehicle_id = -1
  end
end