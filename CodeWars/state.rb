require './query'

class State
  attr_accessor :player, :world

  def add_new_vehicles(vehicles)
    vehicles.each do |vehicle|
      vehicles_by_id.store(vehicle.id, vehicle)
    end
  end

  def apply_vehicle_updates(vehicle_updates)
    vehicle_updates.each do |vehicle_update|
      existing_vehicle = vehicles_by_id[vehicle_update.id]
      save_movement(vehicle_update, existing_vehicle) if existing_vehicle&.update_movement_status?

      existing_vehicle.update(vehicle_update)

      vehicles_by_id.delete(vehicle_update.id) if vehicle_update.durability == 0
    end
  end

  def vehicles
    Query.new(vehicles_by_id.each_value)
  end

  def facilities
    $world.facilities
  end

  alias_method :units, :vehicles

  def vehicles_by_id
    @vehicles_by_id ||= {}
  end

  def ally_damage_at_point(point)
    unit_damage_at_point(point, vehicles.mine)
  end

  def enemy_damage_at_point(point)
    unit_damage_at_point(point, vehicles.not_mine)
  end

  private
  def unit_damage_at_point(point, vehicles)
    max_damage = $game.max_tactical_nuclear_strike_damage
    radius     = $game.tactical_nuclear_strike_radius

    vehicles.reduce(0) do |total_damage, unit|
      distance_to_epicenter = unit.distance_to(point.x, point.y)

      added_damage = if distance_to_epicenter > radius
                       0
                     else
                       [distance_to_epicenter.fdiv(radius) * max_damage, unit.durability].min
                     end

      total_damage + added_damage
    end
  end

  def save_movement(vehicle_update, vehicle)
    if vehicle_moved?(vehicle_update)
      vehicle.last_moved_at = world.tick_index
    end
  end

  def vehicle_moved?(new_vehicle_state)
    prev_vehicle_state = vehicles_by_id[new_vehicle_state.id]
    return true unless prev_vehicle_state

    !close_enough?(new_vehicle_state.x, prev_vehicle_state.x) || !close_enough?(new_vehicle_state.y, prev_vehicle_state.y)
  end

  def close_enough?(x1, x2)
    (x1 - x2).abs <= 0.00001
  end
end
