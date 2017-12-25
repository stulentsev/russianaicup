module PotentialUtils
  def print_potentials(map)
    map.each do |row|
      puts row.map { |x| x.to_s.center(3) }.join(' ')
    end
  end

  # def combine_maps(base, *layers)
  #   create_map do |x, y|
  #     base[x][y] + layers.reduce(0) do |memo, map|
  #       memo + case map
  #              when CellInfluenceMap
  #                map.potential_for(x, y)
  #              else # must be 2-d array
  #                memo + map[x][y]
  #              end
  #     end
  #   end
  # end

  def create_map
    Array.new(32) do |x|
      Array.new(32) do |y|
        yield(x, y)
      end
    end
  end
end
