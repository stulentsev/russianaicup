require './point'

module VehicleExt
  extend Forwardable

  def initialize(*args)
    super

    @movement_watcher_count = 0
  end

  attr_accessor :cluster, :last_moved_at, :movement_watcher_count

  def location
    Point.new(x, y)
  end

  alias_method :center_point, :location

  def aerial?
    aerial
  end

  def mine?
    player_id == $me.id
  end

  def selected?
    selected
  end

  def dead?
    durability == 0
  end

  def alive?
    !dead?
  end

  def vision_range_affected?
    effective_vision_range != vision_range
  end

  def effective_vision_range
    @effective_vision_range ||= vision_range * vision_reduction_factor
  end

  def update(vehicle_update)
    super

    @effective_vision_range = nil # reset
  end

  def increment_movement_watcher_count
    self.movement_watcher_count += 1
  end

  def decrement_movement_watcher_count
    self.movement_watcher_count -= 1
  end

  def update_movement_status?
    movement_watcher_count > 0
  end

  private

  def vision_reduction_factor
    $factors.vision_factor(x: x, y: y, vehicle: self)
  end

end
