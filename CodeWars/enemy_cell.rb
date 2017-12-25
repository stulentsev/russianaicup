class EnemyCell
  attr_reader :enemies

  def initialize(enemies)
    @enemies = enemies.to_a
  end

  def dominant_unit_type
    ut, cnt = enemies.count_by(&:type).max_by(&:last)
    return ut if cnt.fdiv(enemies.count) >= 0.6 # most units are of this type

    nil
  end

  def weak_against?(unit_type)
    my_type = dominant_unit_type
    VehicleType.weak_against?(my_type, unit_type)
  end

  def strong_against?(unit_type)
    my_type = dominant_unit_type
    VehicleType.strong_against?(my_type, unit_type)
  end
end
