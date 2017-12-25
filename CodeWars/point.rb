Point = Struct.new(:x, :y) do
  def self.[](x, y)
    Point.new(x, y)
  end

  # @param [Float] other_x
  # @param [Float] other_y
  # @return [Float]
  def distance_to(other_x, other_y)
    # (other_x - x).abs + (other_y - y).abs # manhattan distance, slightly faster
    Math.hypot(other_x - x, other_y - y)
  end

  # @param [Unit] point
  # @return [Float]
  def distance_to_point(point)
    distance_to(point.x, point.y)
  end

  # @param other [Point, Vector, Array<Integer>] Either a point or array of two numbers
  def +(other)
    other_x, other_y = *other
    Point.new(x + other_x, y + other_y)
  end

  # @param other [Point, Vector, Array<Integer>] Either a point or array of two numbers
  def -(other)
    other_x, other_y = *other
    Point.new(x - other_x, y - other_y)
  end

  def *(factor)
    Point.new(x * factor, y * factor)
  end

  def /(factor)
    Point.new(x.fdiv(factor), y.fdiv(factor))
  end

  # @param x [Integer] cell x (0 to 31)
  # @param y [Integer] cell y (0 to 31)
  # @return [Point] center of the cell
  def self.from_cell(x, y)
    new(x * 32 + 16, y * 32 + 16)
  end

  def to_cell
    Point.new(x / 32, y / 32)
  end

  def to_a
    [x, y]
  end

  def to_s
    "(#{x}x#{y})"
  end
end

def Point(x, y)
  Point.new(x, y)
end
