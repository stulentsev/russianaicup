require 'digest/md5'

class Cluster
  POSITION_HISTORY_SIZE                  = 3
  POSITION_UPDATE_FREQUENCY_IN_TICKS     = 20
  CLUSTER_INFO_UPDATE_FREQUENCY_IN_TICKS = 300

  attr_reader :units, :id, :direction, :speed

  attr_accessor :assigned_strike_point

  def initialize(units)
    @units = units
    units.each {|u| u.cluster = self }
    @id    = Digest::MD5.hexdigest(units.map(&:id).sort.map(&:to_s).join)
  end

  def size
    units.size
  end

  def vehicles
    units
  end

  def alive?
    alive_units.count > 0
  end

  def suggested_strike_point
    if high_speed_units?
      compute_direction_and_speed
      projected_center_point
    else
      center_point
    end
  end

  def damage_at_strike_point
    damage_at_point(assigned_strike_point)
  end

  def damage_at_point(point)
    max_damage = $game.max_tactical_nuclear_strike_damage
    radius = $game.tactical_nuclear_strike_radius

    alive_units.reduce(0) do |total_damage, unit|
      distance_to_epicenter = unit.distance_to(point.x, point.y)

      added_damage = if distance_to_epicenter > radius
        0
      else
        [distance_to_epicenter.fdiv(radius) * max_damage, unit.durability].min
      end

      total_damage + added_damage
    end
  end

  # check if this cluster is capable of high-speed maneuvers
  def high_speed_units?
    alive_units.count(&:aerial?) >= alive_units.count * 0.8
  end

  def reset_center_point
    @center_point = nil
  end

  def center_point
    # TODO: cache
    @center_point ||= compute_center_point
  end
  alias_method :location, :center_point

  def sandwich?
    @sandwich ||= begin
      types = alive_units.count_by(&:type)
      total = alive_units.size

      types.values.all? { |type_count| type_count.fdiv(total) > 0.1 }
    end
  end

  def distance_to_point(point)
    center_point.distance_to_point(point)
  end

  def info
    "Cluster #{id}: #{alive_units.size}/#{units.size} units, travelling at #{speed} meters/tick"
  end

  def inspect
    info
  end

  def to_s
    info
  end

  def compute_direction_and_speed
    if units.empty? || last_positions.size < 2
      @speed     = 0
      @direction = nil
      return
    end

    p1, p2     = last_positions.last(2)
    @direction = Point[p2[0] - p1[0], p2[1] - p1[1]] / POSITION_UPDATE_FREQUENCY_IN_TICKS.to_f
    @speed     = Math::hypot(*@direction)
  end

  def projected_center_point(after_ticks: 30)
    return center_point if direction.nil?

    projected_travel_vector = @direction * after_ticks
    center_point + projected_travel_vector
  end

  def mark_position
    cp = compute_center_point
    last_positions.push(cp)
    @center_point = cp
    last_positions.shift if last_positions.size > POSITION_HISTORY_SIZE
  end

  private

  def alive_units
    units.select(&:alive?)
  end

  def compute_center_point
    alive_units.center_point
  end

  def last_positions
    @last_positions ||= []
  end

end
