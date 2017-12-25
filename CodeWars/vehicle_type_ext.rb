require './model/vehicle_type'

module VehicleTypeExt
  def aerial
    @aerial ||= [self::FIGHTER, self::HELICOPTER]
  end

  def ground
    @ground ||= [self::ARRV, self::TANK, self::IFV]
  end

  def all
    @all ||= ground + aerial
  end

  def name(type)
    case type
    when self::FIGHTER
      'Fighter'
    when self::HELICOPTER
      'Helicopter'
    when self::TANK
      'Tank'
    when self::IFV
      'IFV'
    when self::ARRV
      'ARRV'
    else
      'Unknown'
    end
  end

  def strong_against?(my_type, enemy_type)
    case enemy_type
    when *aerial
      [self::IFV, self::FIGHTER].include?(my_type)
    when self::IFV, self::TANK
      [self::HELICOPTER, self::TANK].include?(my_type)
    else # self::ARRV
      ![self::ARRV, self::FIGHTER].include?(my_type)
    end
  end

  def weak_against?(my_type, enemy_type)
    case enemy_type
    when self::IFV, self::FIGHTER
      aerial.include?(my_type)
    when self::HELICOPTER, self::TANK
      ground.include?(my_type)
    else # self::ARRV
      false
    end
  end

  def ignores?(my_type, enemy_type)
    case enemy_type
    when *ground
      my_type == self::FIGHTER
    when self::FIGHTER
      my_type == self::TANK
    else
      false
    end
  end
end
