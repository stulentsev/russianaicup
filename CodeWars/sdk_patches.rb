require './model/game'
require './model/move'
require './model/player'
require './model/world'
require './model/vehicle_type'
require './model/facility_type'
require './remote_process_client'

require './query'
require './facility_ext'
require './vehicle_ext'
require './vehicle_type_ext'

class Unit
  alias_method :distance_to_point, :distance_to_unit
end

module BetterFacilityApi
  def facilities
    facilities_from_superclass = super
    return unless facilities_from_superclass

    Query.new(facilities_from_superclass)
  end
end

class World
  prepend BetterFacilityApi

  def center_point
    Point(width.fdiv(2), height.fdiv(2))
  end

  def with_facilities?
    !(facilities.nil? || facilities.empty?)
  end
end


module FacilityType
  def self.name(type)
    case type
    when CONTROL_CENTER
      'Control Center'
    when VEHICLE_FACTORY
      'Factory'
    else
      'Unknown'
    end
  end
end

Vehicle.prepend(VehicleExt)
Facility.prepend(FacilityExt)
VehicleType.extend(VehicleTypeExt)
