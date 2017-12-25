require './model/unit'

class CircularUnit < Unit
  # @return [Float]
  attr_reader :radius

  # @param [Integer] id
  # @param [Float] x
  # @param [Float] y
  # @param [Float] radius
  def initialize(id, x, y, radius)
    super(id, x, y)

    @radius = radius
  end
end