require 'json'
require 'socket'

module AreaType
  UNKNOWN = 0
  FOREST  = 1
  SWAMP   = 2
  RAIN    = 3
  CLOUD   = 4

  def self.random
    const_get(constants.sample)
  end

  def self.from_terrain_type(terrain_type)
    {
      TerrainType::SWAMP  => AreaType::SWAMP,
      TerrainType::FOREST => AreaType::FOREST,
    }[terrain_type]
  end

  def self.from_weather_type(weather_type)
    {
      WeatherType::CLOUD => AreaType::CLOUD,
      WeatherType::RAIN  => AreaType::RAIN,
    }[weather_type]
  end
end

module Side
  OUR     = -1
  NEUTRAL = 0
  ENEMY   = 1

  def self.random
    const_get(constants.sample)
  end
end

module UnitType
  UNKNOWN    = 0
  TANK       = 1
  IFV        = 2
  ARRV       = 3
  HELICOPTER = 4
  FIGHTER    = 5

  def self.random
    const_get(constants.sample)
  end

  def self.from_vehicle_type(vehicle_type)
    {
      VehicleType::TANK       => UnitType::TANK,
      VehicleType::IFV        => UnitType::IFV,
      VehicleType::ARRV       => UnitType::ARRV,
      VehicleType::HELICOPTER => UnitType::HELICOPTER,
      VehicleType::FIGHTER    => UnitType::FIGHTER,
    }[vehicle_type]
  end
end

class RewindClient

  def initialize(host = '127.0.0.1', port = 9111)
    @socket = TCPSocket.new(host, port)
    socket.setsockopt(Socket::IPPROTO_TCP, Socket::TCP_NODELAY, 1)
  end

  def frame
    yield self
  ensure
    end_frame
  end

  def start_frame
    @did_draw = false
  end

  # finalizes frame composition. Must be called for anything to be drawn.
  # You may want to use RewindClient#frame method, which calls this at the end of the block.
  def end_frame
    send_json(type: 'end') if @did_draw
  end

  # @param x [Float]
  # @param y [Float]
  # @param r [Float] radius
  # @param color [Integer] RGBA (see Color class)
  # @param layer [Integer] layer to draw on
  def circle(x, y, r, color, layer)
    send_json(type: 'circle', x: x, y: y, r: r, color: color.to_i, layer: layer)
  end

  def rect(x1, y1, x2, y2, color, layer)
    send_json(type: 'rectangle', x1: x1, y1: y1, x2: x2, y2: y2, color: color.to_i, layer: layer)
  end

  def line(x1, y1, x2, y2, color, layer)
    send_json(type: 'line', x1: x1, y1: y1, x2: x2, y2: y2, color: color.to_i, layer: layer)
  end

  def popup(x, y, r, text)
    send_json(type: 'popup', x: x, y: y, r: r, text: text)
  end

  def living_unit(x, y, r, hp, max_hp, side, course = 0, unit_type = UnitType::UNKNOWN, rem_cooldown = 0, max_cooldown = 0, selected = false)
    send_json(
      type:         'unit',
      x:            x, y: y, r: r, hp: hp, max_hp: max_hp, enemy: side,
      unit_type:    unit_type, course: course.round(3),
      rem_cooldown: rem_cooldown, cooldown: max_cooldown,
      selected:     selected ? 1 : 0,
    )
  end

  # @param cell_x [Integer] x of top-left facility cell
  # @param cell_y [Integer] y of top-left facility cell
  # @param type [Integer] one of FacilityType constants
  # @param side [Integer] ally, neutral or enemy
  # @param production [Integer] current production progress, set to 0 if no production
  # @param max_production [Integer] maximum production progress, used together with `production`
  # @param capture [Integer] current capture progress, should be in range [-max_capture, max_capture], where negative values mean that facility is being captured by enemy
  # @param max_capture [Integer] maximum capture progress, used together with `capture`
  def facility(cell_x, cell_y, type, side, production, max_production, capture, max_capture)
    send_json(
      type:           'facility',
      x:              cell_x,
      y:              cell_y,
      facility_type:  type,
      enemy:          side,
      production:     production,
      max_production: max_production,
      capture:        capture,
      max_capture:    max_capture
    )
  end


  def area_description(cell_x, cell_y, area_type)
    send_json(type: 'area', x: cell_x, y: cell_y, area_type: area_type)
  end

  def message(msg)
    send_json(type: 'message', message: msg)
  end

  def close
    socket.close
  end

  private

  attr_reader :socket

  def send_json(hash)
    @did_draw = true
    puts hash if $log
    socket.write(hash.to_json)
    socket.write("\n")
    socket.flush
  end
end

if __FILE__ == $0
  rc = RewindClient.new

  rc.circle(100, 100, 50, Color.new(255, 0, 0, 50), 0)
  rc.circle(150, 150, 50, Color.new(0, 255, 0, 50), 0)
  rc.circle(200, 200, 50, Color.new(0, 0, 255, 50), 0)
  rc.circle(250, 250, 50, Color.new(0, 0, 0, 50), 0)
  rc.circle(300, 300, 50, Color.new(255, 255, 255, 50), 0)
  rc.end_frame

  rc.close
end
