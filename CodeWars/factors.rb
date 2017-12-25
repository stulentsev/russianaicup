require './model/game'
require './model/world'

class Factors
  attr_accessor :terrain_by_cell_x_y, :weather_by_cell_x_y

  def vision_factor(x:, y:, vehicle:)
    if vehicle.aerial
      case weather_by_cell_x_y[x / 32][y / 32]
      when WeatherType::CLEAR
        $game.clear_weather_vision_factor
      when WeatherType::CLOUD
        $game.cloud_weather_vision_factor
      when WeatherType::RAIN
        $game.rain_weather_vision_factor
      else
        1.0 # ¯\_(ツ)_/¯
      end
    else
      case terrain_by_cell_x_y[x / 32][y / 32]
      when TerrainType::PLAIN
        $game.plain_terrain_vision_factor
      when TerrainType::SWAMP
        $game.swamp_terrain_vision_factor
      when TerrainType::FOREST
        $game.forest_terrain_vision_factor
      else
        1.0 # ¯\_(ツ)_/¯
      end
    end
  end

  private

end

$factors = Factors.new
