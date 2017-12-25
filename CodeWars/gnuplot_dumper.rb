class GnuplotDumper
  def self.dump(field_value_to_cells)
    new.dump(field_value_to_cells)
  end

  def dump(field_value_to_cells)
    colrow_to_field_value = field_value_to_cells.each_with_object({}) do |(field_value, cells), memo|
      cells.each do |cell|
        memo[cell.to_a] = field_value
      end
    end

    excel_row_arrays = Array.new(32) { Array.new(32) { 0} }

    colrow_to_field_value.each do |(x, y), field_value|
      excel_row_arrays[y / 32][x / 32] = field_value
    end

    File.open(filename, 'w') do |file|
      0.upto(31).each do |row_idx|
        file.puts excel_row_arrays[row_idx].join("\t")
      end
    end


    `gnuplot  gpmap.script > "map #{$world.tick_index}.png"`
    true
  end

  def filename
    'surface_map.dat'
  end
end
