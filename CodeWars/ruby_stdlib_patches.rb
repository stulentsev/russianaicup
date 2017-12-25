module Enumerable
  def count_by(&block)
    Hash[group_by(&block).map { |key, vals| [key, vals.size] }]
  end

  def center_distance_from(other_point)
    center_point.distance_to_point(other_point)
  end

  # @return [Array<Integer>] x, y of the point
  def center_point
    x   = 0
    y   = 0
    cnt = 0
    each do |vehicle|
      x   += vehicle.x
      y   += vehicle.y
      cnt += 1
    end

    Point[x.fdiv(cnt).round(2), y.fdiv(cnt).round(2)]
  end

  def lefttop_point
    x = nil
    y = nil
    each do |vehicle|
      x = vehicle.x if x.nil? || vehicle.x < x
      y = vehicle.y if y.nil? || vehicle.y < y
    end

    Point[x, y]
  end

  def bottomright_point
    x = nil
    y = nil
    each do |vehicle|
      x = vehicle.x if x.nil? || vehicle.x > x
      y = vehicle.y if y.nil? || vehicle.y > y
    end

    Point(x, y)
  end
end

class Enumerator
  def +(other)
    Enumerator.new do |y|
      each { |e| y << e }
      other.each { |e| y << e }
    end
  end
end

module DistanceMeasures
  def cells
    (self * 32).to_f
  end
  alias_method :cell, :cells

  def meters
    self.to_f
  end
  alias_method :meter, :meters
end

Integer.send(:include, DistanceMeasures) if defined?(Integer)
Fixnum.send(:include, DistanceMeasures) if defined?(Fixnum)
Float.send(:include, DistanceMeasures) if defined?(Float)

class Hash
  def transform_values
    return enum_for(:transform_values) unless block_given?
    result = self.class.new
    each do |key, value|
      result[key] = yield(value)
    end
    result
  end
end

module Kernel
  alias_method :old_puts, :puts
  def puts(*args)
    old_puts(*args) if $localhost
  end
end
