require './model/terrain_type'
require './model/weather_type'
require './color'
require './rewind_client'
require './jam'

module Visualizable
  def show_terrain(terrain_by_x_y)
    0.upto(31) do |x|
      0.upto(31) do |y|
        rew.area_description(x, y, AreaType.from_terrain_type(terrain_by_x_y[x][y]).to_i)
      end
    end
  end

  def show_weather(weather_by_x_y)
    0.upto(31) do |x|
      0.upto(31) do |y|
        rew.area_description(x, y, AreaType.from_weather_type(weather_by_x_y[x][y]).to_i)
      end
    end
  end

  def show_units_with_vision
    board.vehicles.aerial.mine.each do |v|
      color = (v.player_id == me.id) ? Color.green : Color.red
      jam.circle(v.x, v.y, v.effective_vision_range, color.opacity(1))
    end
  end

  def show_facility(facility)
    productions = {
      VehicleType::FIGHTER    => 90,
      VehicleType::HELICOPTER => 75,
    }
    rew.facility(
      facility.left / 32,
      facility.top / 32,
      facility.type,
      facility.mine? ? -1 : (facility.enemy? ? 1 : 0),
      facility.production_progress,
      productions[facility.vehicle_type] || 60,
      0, # TODO: add capture points later
      0
    )
  end

  def show_directions
    wandering_groups.each do |squadron|
      next unless squadron.destination
      next if squadron.dead?

      loc = squadron.location
      dest = squadron.destination
      jam.line(loc.x, loc.y, dest.x, dest.y, Color.green)
    end
  end

  def show_units(units)
    units.each do |unit|
      show_unit(unit)
    end
  end

  def show_unit(unit)
    rew.living_unit(
      unit.x,
      unit.y,
      unit.radius,
      unit.durability,
      unit.max_durability,
      unit.mine? ? -1 : 1,
      0,
      UnitType.from_vehicle_type(unit.type),
      0, 0,
      unit.selected? ? 1 : 0
    )
  end

  def show_field_values(unit_location, destination, field_value_to_cells)
    max_negative, max_positive = field_value_to_cells.keys.minmax

    field_value_to_cells.each do |field_strength, cells|
      color = if field_strength.positive?
                Color.green.saturation(field_strength.fdiv(max_positive))
              elsif field_strength.negative?
                Color.red.saturation(field_strength.fdiv(max_negative))
              else
                Color.white
              end
      cells.each do |cell|
        rew.rect(cell.x - 16, cell.y - 16, cell.x + 16, cell.y + 16, color.to_i, 2)
      end
    end; true

    rew.circle(unit_location.x, unit_location.y, 16, Color.blue.to_i, 3)
    rew.rect(destination.x - 5, destination.y - 5, destination.x + 5, destination.y + 5, Color.blue, 3)
    rew.line(unit_location.x, unit_location.y, destination.x, destination.y, Color.blue, 3)
  end

  def show_nuke(player: , color: )
    return if player.next_nuclear_strike_tick_index == -1

    color = if player.next_nuclear_strike_tick_index <= $world.tick_index + 1
              Color.red
            else
              until_strike = player.next_nuclear_strike_tick_index - $world.tick_index
              delay = $game.tactical_nuclear_strike_delay

              color.opacity((1 - until_strike.fdiv(delay)) * 255)
            end

    # nuke area
    rew.circle(
      player.next_nuclear_strike_x,
      player.next_nuclear_strike_y,
      $game.tactical_nuclear_strike_radius,
      color,
      1
    )

    # nuke highlighter
    unit = board.vehicles_by_id[player.next_nuclear_strike_vehicle_id]
    color = player.me ? Color.orange : Color.blueish

    rew.circle(
      unit.x,
      unit.y,
      unit.radius * 5,
      color,
      2
    )
  end

  def jam
    $jam ||= if AppSettings.jam
               Jam.new
             else
               NoopRewindClient.new
             end
  end

  def rew
    $rewind_client ||= if AppSettings.rewind
      RewindClient.new
    else
      NoopRewindClient.new
    end
  end

  class NoopRewindClient
    def method_missing(*)

    end
  end
end
