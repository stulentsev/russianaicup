require 'securerandom'
require './enemy_cell'

class Squadron
  attr_reader :id, :units, :group, :movement_start_tick, :unit_type, :speed

  attr_accessor :destination, :movement_time_in_ticks, :turns_cooldown, :brain, :healing

  def initialize(units:, group:, brain: Brains::Null.new)
    @id    = SecureRandom.uuid
    @units = units
    @group = group
    @movement_time_in_ticks = 50
    @brain = brain
    if units.count > 0
      @speed = units.map(&:max_speed).min
      @aerial = VehicleType.aerial.include?(units.first.type)
      @unit_type = units.first.type
    end
  end

  def inspect
    "<Squadron:#{id} using #{brain.class}, #{live_units.count} units, location: #{location}, heading to #{destination}"
  end

  def location
    live_units.center_point
  end

  def alive?
    units.any?(&:alive?)
  end

  def size
    units.size
  end

  def dead?
    !alive?
  end

  def move
    brain.move
  end

  def compact
    brain.schedule(:rotate_left, 50)
    brain.schedule(:shrink, 50)
    brain.schedule(:rotate_right, 50)
    brain.schedule(:shrink, 50)
    @compaction_attempted_at = $world.tick_index

    true
  end

  def attempted_compaction?
    return false unless @compaction_attempted_at

    @compaction_attempted_at + 600 > $world.tick_index
  end

  def remove_dead_units
    units.reject!(&:dead?)
  end

  def keep_distance_from?(squadron)
    aerial? == squadron.aerial?
  end

  def aerial?
    @aerial
  end

  def ground?
    !aerial?
  end

  def arrv?
    unit_type == VehicleType::ARRV
  end

  def min_speed
    @min_speed ||= speed * 0.6
  end

  def start_moving!
    @movement_start_tick = $world.tick_index
  end

  def finished_moving?
    $world.tick_index >= movement_start_tick + movement_time_in_ticks
  end

  def low_health?
    relative_health <= 0.8
  end

  def good_health?
    relative_health >= 0.92
  end

  def easy_prey?(enemies)
    cell = EnemyCell.new(enemies)
    cell.weak_against?(unit_type) && !cell.strong_against?(unit_type)
  end

  def total_health
    live_units.map(&:durability).reduce(:+)
  end

  def lost_formation?
    units.combination(2).any? do |u1, u2|
      u1.distance_to_unit(u2) > ACCEPTABLE_MAX_DISTANCE_BETWEEN_UNITS
    end
  end

  def see_point?(point)
    units.any?{|u| u.distance_to_point(point) < u.effective_vision_range }
  end

  ACCEPTABLE_MAX_DISTANCE_BETWEEN_UNITS = 90.meters

  private

  def relative_health
    remaining_health = live_units.map(&:durability).reduce(:+)
    total_health     = live_units.map(&:max_durability).reduce(:+)
    remaining_health.fdiv(total_health)
  end

  def live_units
    units.select(&:alive?)
  end

end
