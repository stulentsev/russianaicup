require 'forwardable'

class Query
  extend Forwardable

  attr_reader :collection, :me

  # @param collection [Enumerator]
  def initialize(collection)
    @collection = case collection
                  when Enumerator::Lazy
                    collection
                  else
                    collection.lazy
                  end
  end

  def empty?
    collection.count == 0
  end

  def mine
    Query.new(collection.select { |elem| elem.player_id == $me.id })
  end

  def not_mine
    Query.new(collection.select { |elem| elem.player_id != $me.id })
  end

  def alive
    Query.new(collection.select(&:alive?))
  end

  def unclustered
    Query.new(collection.select { |elem| elem.cluster.nil? })
  end

  def of_type(type)
    Query.new(collection.select { |elem| elem.type == type })
  end

  %i[aerial selected].each do |flag|
    define_method flag do
      Query.new(collection.select { |elem| elem.send(flag) })
    end
  end

  def ground
    Query.new(collection.select { |elem| !elem.aerial })
  end

  def ungrouped
    Query.new(collection.select { |elem| elem.groups.empty? })
  end

  def group(group)
    Query.new(collection.select { |elem| elem.groups.include?(group) })
  end

  def at_cell(cx, cy)
    Query.new(collection.select { |elem|
      elem.x.between?(cx * 32, (cx + 1) * 32) &&
        elem.y.between?(cy * 32, (cy + 1) * 32)
    })
  end

  def at_facility(facility)
    Query.new(collection.select { |elem|
      elem.x.between?(facility.left, facility.left + facility.width) &&
        elem.y.between?(facility.top, facility.top + facility.height)
    })
  end

  def control_centers
    Query.new(collection.select(&:control_center?))
  end

  alias_method :command_centers, :control_centers

  def factories
    Query.new(collection.select(&:factory?))
  end

  alias_method :vehicle_factories, :factories

  def untaken
    Query.new(collection.select(&:untaken?))
  end

  def no_production
    Query.new(collection.select(&:no_production?))
  end

  %i[arrv tank ifv fighter helicopter].each do |vtype|
    define_method "#{vtype}s" do
      const_name  = vtype.upcase
      const_value = VehicleType.const_get(const_name)
      Query.new(collection.select { |elem| elem.type == const_value })
    end
  end

  def method_missing(method, *args, &block)
    if collection.respond_to?(method)
      collection.send(method, *args, &block)
    else
      super
    end
  end

  def respond_to_missing?(method)
    collection.respond_to?(method)
  end
end
