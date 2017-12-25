require './model/facility_type'
require './model/vehicle_type'

class Facility
  # @return [Integer]
  attr_reader :id

  # @return [Integer, NilClass]
  attr_reader :type

  # @return [Integer]
  attr_reader :owner_player_id

  # @return [Float]
  attr_reader :left

  # @return [Float]
  attr_reader :top

  # @return [Float]
  attr_reader :capture_points

  # @return [Integer, NilClass]
  attr_reader :vehicle_type

  # @return [Integer]
  attr_reader :production_progress

  # @param [Integer] id
  # @param [Integer, NilClass] type
  # @param [Integer] owner_player_id
  # @param [Float] left
  # @param [Float] top
  # @param [Float] capture_points
  # @param [Integer, NilClass] vehicle_type
  # @param [Integer] production_progress
  def initialize(id, type, owner_player_id, left, top, capture_points, vehicle_type, production_progress)
    @id = id
    @type = type
    @owner_player_id = owner_player_id
    @left = left
    @top = top
    @capture_points = capture_points
    @vehicle_type = vehicle_type
    @production_progress = production_progress
  end
end