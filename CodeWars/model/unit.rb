# noinspection RubyInstanceVariableNamingConvention
class Unit
  # @return [Integer]
  attr_reader :id

  # @return [Float]
  attr_reader :x

  # @return [Float]
  attr_reader :y

  # @param [Integer] id
  # @param [Float] x
  # @param [Float] y
  def initialize(id, x, y)
    @id = id
    @x = x
    @y = y
  end

  # @param [Float] x
  # @param [Float] y
  # @return [Float]
  def distance_to(x, y)
    Math::hypot(x - @x, y - @y)
  end

  # @param [Unit] unit
  # @return [Float]
  def distance_to_unit(unit)
    distance_to(unit.x, unit.y)
  end

  # @param [Float] x
  # @param [Float] y
  # @return [Float]
  def squared_distance_to(x, y)
    x_range = x - @x
    y_range = y - @y
    x_range * x_range + y_range * y_range
  end

  # @param [Unit] unit
  # @return [Float]
  def squared_distance_to_unit(unit)
    squared_distance_to(unit.x, unit.y)
  end
end