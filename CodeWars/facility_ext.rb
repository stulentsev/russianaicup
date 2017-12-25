require 'forwardable'

require './model/facility'

module FacilityExt
  extend Forwardable

  def_delegators :center_point,
                 :distance_to_point, :distance_to

  attr_accessor :targeted_for_capturing_by

  def player_id
    owner_player_id
  end

  def mine?
    owner_player_id == $me.id
  end

  def enemy?
    ![$me.id, -1].include?(owner_player_id)
  end

  def untaken?
    owner_player_id == -1
  end

  def no_production?
    vehicle_type.nil? || vehicle_type == -1
  end

  def center_point
    Point.new(left + width / 2, top + height / 2)
  end
  alias_method :location, :center_point

  def control_center?
    type == FacilityType::CONTROL_CENTER
  end
  alias_method :command_center?, :control_center?

  def factory?
    type == FacilityType::VEHICLE_FACTORY
  end
  alias_method :vehicle_factory?, :factory?

  def width
    64
  end

  def height
    64
  end

  def point_within_bounds?(point)
    return false unless point

    point.x.between?(left, left + width) &&
      point.y.between?(top, top + height)
  end

  def inspect
    if control_center?
      "[#{FacilityType.name(type)}, at #{location}"
    else
      production_name = no_production? ? 'nothing' : VehicleType.name(vehicle_type)
      "[#{FacilityType.name(type)}], at #{location}, producing #{production_name}"
    end
  end
end
