require './model/facility'
require './model/player'
require './model/terrain_type'
require './model/vehicle'
require './model/vehicle_update'
require './model/weather_type'

# noinspection RubyTooManyInstanceVariablesInspection
class World
  # @return [Integer]
  attr_reader :tick_index

  # @return [Integer]
  attr_reader :tick_count

  # @return [Float]
  attr_reader :width

  # @return [Float]
  attr_reader :height

  # @return [Array<Player, NilClass>, NilClass]
  attr_reader :players

  # @return [Array<Vehicle, NilClass>, NilClass]
  attr_reader :new_vehicles

  # @return [Array<VehicleUpdate, NilClass>, NilClass]
  attr_reader :vehicle_updates

  # @return [Array<Array<Integer, NilClass>, NilClass>, NilClass]
  attr_reader :terrain_by_cell_x_y

  # @return [Array<Array<Integer, NilClass>, NilClass>, NilClass]
  attr_reader :weather_by_cell_x_y

  # @return [Array<Facility, NilClass>, NilClass]
  attr_reader :facilities

  # @param [Integer] tick_index
  # @param [Integer] tick_count
  # @param [Float] width
  # @param [Float] height
  # @param [Array<Player, NilClass>, NilClass] players
  # @param [Array<Vehicle, NilClass>, NilClass] new_vehicles
  # @param [Array<VehicleUpdate, NilClass>, NilClass] vehicle_updates
  # @param [Array<Array<Integer, NilClass>, NilClass>, NilClass] terrain_by_cell_x_y
  # @param [Array<Array<Integer, NilClass>, NilClass>, NilClass] weather_by_cell_x_y
  # @param [Array<Facility, NilClass>, NilClass] facilities
  def initialize(tick_index, tick_count, width, height, players, new_vehicles, vehicle_updates, terrain_by_cell_x_y,
                 weather_by_cell_x_y, facilities)
    @tick_index = tick_index
    @tick_count = tick_count
    @width = width
    @height = height
    @players = players
    @new_vehicles = new_vehicles
    @vehicle_updates = vehicle_updates
    @terrain_by_cell_x_y = terrain_by_cell_x_y
    @weather_by_cell_x_y = weather_by_cell_x_y
    @facilities = facilities
  end

  # @return [Player, NilClass]
  def my_player
    @players.each { |player| return player if player.me }
    nil
  end

  # @return [Player, NilClass]
  def opponent_player
    @players.each { |player| return player unless player.me }
    nil
  end
end