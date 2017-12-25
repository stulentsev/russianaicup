require './cluster'

module DBSCAN
  class Clusterer
    attr_accessor :points, :options, :clusters

    def initialize(units, options = {})
      options[:epsilon] ||= 20
      options[:min_points] ||= 10

      c = 0

      @points, @options, @clusters = units.map { |unit| point = Point.new(unit); c += 1; point }, options, { -1 => [] }

      clusterize!
    end

    def clusterize!
      current_cluster = -1
      @points.each do |point|
        unless point.visited?
          point.visit!
          neighbors = immediate_neighbors(point)
          if neighbors.size >= options[:min_points]
            current_cluster           += 1
            point.cluster             = current_cluster
            cluster                   = [point].push(add_connected(neighbors, current_cluster))
            clusters[current_cluster] = cluster.flatten
          else
            clusters[-1].push(point)
          end
        end
      end

    end

    def results
      @clusters.map do |cid, dbscan_points|
        next if cid == -1 # unclustered
        next if dbscan_points.empty?

        Cluster.new(dbscan_points.map(&:unit))
      end.compact
    end

    def labeled_results
      hash = {}
      @clusters.each do |cluster_index, elements|
        hash.store(cluster_index, [])
        elements.each do |e|
          hash[cluster_index].push(e.label)
        end
      end
      hash
    end

    def immediate_neighbors(point)
      neighbors = []
      @points.each do |p|
        if p.unit != point.unit
          d = point.unit.distance_to_unit(p.unit)
          neighbors.push(p) if d < options[:epsilon]
        end
      end
      neighbors
    end

    def add_connected(neighbors, current_cluster)
      cluster_points = []
      neighbors.each do |point|
        unless point.visited?
          point.visit!
          new_points = immediate_neighbors(point)

          if new_points.size >= options[:min_points]
            new_points.each do |p|
              unless neighbors.include?(p)
                neighbors.push(p)
              end
            end
          end
        end

        unless point.cluster
          cluster_points.push(point)
          point.cluster = current_cluster
        end
      end

      cluster_points
    end
  end

  class Point
    attr_accessor :unit, :cluster, :visited, :label

    def initialize(unit)
      @unit = unit
      @cluster = nil
      @visited = false
    end

    def visited?
      @visited
    end

    def visit!
      @visited = true
    end
  end
end

def DBSCAN(units, epsilon: 15, min_points: 10)
  clusterizer = DBSCAN::Clusterer.new(units, epsilon: epsilon, min_points: min_points)
  clusterizer.clusterize!
  clusterizer.results
end
