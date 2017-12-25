require './point'
require './strategy_helpers'

class Emitter
  attr_reader :location, :effect_radius, :max_effect_value

  # @param location [Point]
  def initialize(location:, effect_radius:, max_effect_value:)
    @location         = location
    @effect_radius    = effect_radius
    @max_effect_value = max_effect_value
  end

  # @param point [Point] where we need to measure field
  def value_at_point(point, squadron: nil)
    value_at_distance(location.distance_to_point(point))
  end

  # @param point [Point] where we need to measure field
  def within_range_of(point)
    location.distance_to_point(point) <= effect_radius
  end

  def value_at_distance(dist, squadron: nil)
    fail NotImplementedError
  end

  def x
    location.x
  end

  def y
    location.y
  end
end


class SafeDistanceEmitter < Emitter
  include StrategyHelpers

  attr_reader :preferable_distance, :preferable_distance_variation

  def initialize(preferable_distance:, preferable_distance_variation:, **kwargs)
    super(**kwargs)

    @max_effect_value              = max_effect_value
    @preferable_distance           = preferable_distance
    @preferable_distance_variation = preferable_distance_variation
  end

  def value_at_distance(dist, squadron: nil)
    pref_dist = preferable_distance
    if squadron
      pref_dist *= 2.0 if squadron.lost_formation?
      pref_dist -= 10 if nuke_almost_ready?(ticks = 100)
    end

    if dist.between?(0, pref_dist - preferable_distance_variation)
      -10.0
    elsif dist.between?(pref_dist - preferable_distance_variation + 1, pref_dist + preferable_distance_variation)
      max_effect_value
    elsif dist < effect_radius
      # gradual [linear] decay beyond preferable distance
      max_effect_value * (1.0 - (dist - pref_dist).fdiv(effect_radius - pref_dist))
    else
      0.0
    end

  end
end

class LinearFalloffEmitter < Emitter
  def value_at_distance(dist, squadron: nil)
    if dist.between?(0, effect_radius)
      max_effect_value * (1.0 - (dist.fdiv(effect_radius)))
    else
      0.0
    end
  end
end

class ExponentialFalloffEmitter < Emitter
  attr_reader :exponent, :bias

  def initialize(exponent:, bias: 0, **kwargs)
    super(**kwargs)
    @exponent = exponent
    @bias = bias
  end

  def value_at_distance(dist, squadron: nil)
    if dist.between?(0, effect_radius)
      max_effect_value * 2 ** -(1.fdiv(exponent) * dist) + bias
    else
      0.0
    end
  end
end


if __FILE__ == $0
  emitter = ExponentialFalloffEmitter.new(max_effect_value: -500, exponent: 50, effect_radius: 150, location: Point.new(0, 0))

  0.upto(12).each do |hundred|
    puts emitter.value_at_distance(hundred * 100)
  end
end
