require './strategy_base'

module Strategies
  class Sandwich < Base

    every 50.ticks do |tick|
      puts '-' * 10
      (UNITS_GROUND + UNITS_SUPPORT).each do |unit_type|
        units = board.vehicles.mine.of_type(unit_type)
        lt    = units.lefttop_point
        br    = units.bottomright_point

        puts "tick #{tick}: unit #{unit_type}, #{lt} #{br}"
      end
      puts '-' * 10
    end

    def handle_tick

      if world.tick_index == 0
        initiate_sandwich
      end
    end


    private

    def initiate_sandwich
      unit_groups = (UNITS_GROUND + UNITS_SUPPORT).each_with_object({}) do |unit_type, memo|
        units           = board.vehicles.mine.of_type(unit_type)
        coords_ary      = [18, 92, 166]
        p               = units.lefttop_point
        position_number = coords_ary.index(p.x.to_i) + coords_ary.index(p.y.to_i) * 3
        puts "unit_type: #{unit_type} at position #{position_number}"
        pipeline.select(vehicle_type: unit_type)
        pipeline.assign(group: unit_type + 1)
        pipeline.assign(group: 90 + position_number)
        pipeline.assign(group: 85)
        memo[position_number] = units
      end
      build_formation(unit_groups)
    end

    def build_formation(unit_groups)
      resulting_shape, num_turns, row_or_col, movements = starting_positions_to_movements[unit_groups.keys.sort]
      puts "units will be in a #{resulting_shape} formation"
      execute_movements(movements) if movements

      start_sandwich = -> {
        case resulting_shape
        when :vertical
          build_vertical_sandwich(row_or_col)
        when :horizontal
          build_horizontal_sandwich(row_or_col)
        else
          # do nothing
        end
      }

      if num_turns.zero?
        start_sandwich.call
      else
        delayed.after(ticks_per_turn * num_turns) do
          start_sandwich.call
        end
      end
    end

    def execute_movements(movements)
      movements.each do |pos, direction|
        case direction
        when Array
          execute_rotation(pos, *direction)
        when Hash
          dir, other_movements = direction.first
          execute_movement(pos, dir) do
            execute_movements(other_movements)
          end
        else
          execute_movement(pos, direction)
        end

      end
    end

    def execute_rotation(pos, center_pos, angle_degrees)
      center_points_x = center_points_y = [18 + 27, 92 + 27, 166 + 27]
      center_point = Point.new(center_points_x[pos / 3], center_points_y[pos % 3])

      pivot_point_diffs = { up: [0, -74], down: [0, 74], right: [74, 0], left: [-74, 0] }
      center_point.x += pivot_point_diffs[center_pos][0]
      center_point.y += pivot_point_diffs[center_pos][1]

      pipeline.select_group(group: 90 + pos)
      pipeline.rotate(x: center_point.x, y: center_point.y, angle_degrees: angle_degrees)
    end

    def execute_movement(pos, direction, &block)
      movement_vectors = { up: [0, -74], down: [0, 74], right: [74, 0], left: [-74, 0] }
      pipeline.select_group(group: 90 + pos)
      x, y = movement_vectors[direction]

      if block
        pipeline.move(x: x, y: y, max_speed: lowest_ground_speed) do
          delayed.after(ticks_per_turn) do
            block.call
          end
        end
      else
        pipeline.move(x: x, y: y, max_speed: lowest_ground_speed)
      end
    end

    def build_vertical_sandwich(col)
      puts "tick #{world.tick_index}: building vertical sandwich at col #{col}"
      groups                = [81, 82, 83]
      x                     = y = [18, 92, 166]
      factor                = 2.78
      scaled_squad_size     = (54 * factor).ceil
      padding               = 20

      middle_group_x_offset = 10
      middle_group_y_offset = padding * factor + 30
      bottom_group_x_offset  = 5
      bottom_group_y_offset  = middle_group_y_offset * 2

      unscaled_squad_size = 60
      pipeline.select(top: 166, left: x[col], bottom: 166 + unscaled_squad_size, right: x[col] + unscaled_squad_size)
      pipeline.assign(group: groups[2])
      pipeline.move(x: bottom_group_x_offset, y: bottom_group_y_offset, max_speed: lowest_ground_speed) do
        delayed.when_stop_moving(board.vehicles.ground.mine.group(groups[2]), after: 250.ticks) do
          pipeline.select_group(group: groups[2])
          pipeline.scale(
            x:      x[col] + bottom_group_x_offset,
            y:      166 + bottom_group_y_offset,
            factor: factor
          # max_speed: lowest_ground_speed
          )
        end
      end

      pipeline.select(top: 92, left: x[col], bottom: 92 + unscaled_squad_size, right: x[col] + unscaled_squad_size)
      pipeline.assign(group: groups[1])
      pipeline.move(x: middle_group_x_offset, y: middle_group_y_offset, max_speed: lowest_ground_speed) do
        delayed.when_stop_moving(board.vehicles.ground.mine.group(groups[1]), after:  250.ticks) do
          pipeline.select_group(group: groups[1])
          pipeline.scale(
            x:      x[col] + middle_group_x_offset,
            y:      92 + middle_group_y_offset,
            factor: factor
          )
        end
      end

      pipeline.select(top: 18, left: x[col], bottom: 18 + unscaled_squad_size, right: x[col] + unscaled_squad_size)
      pipeline.assign(group: groups[0])
      delayed.when_stop_moving(board.vehicles.ground.mine.group(groups[2]), after: 450.ticks) do
        pipeline.select_group(group: groups[0])
        pipeline.scale(y: 18, x: x[col], factor: factor) do
          delayed.when_stop_moving(board.vehicles.ground.mine, after: 600.ticks) do
            pipeline.select_group(group: groups[0])
            pipeline.move(y: (92 + middle_group_y_offset) - 18, x: 0)

            pipeline.select_group(group: groups[2])
            pipeline.move(y: 92 - (middle_group_y_offset + bottom_group_y_offset) + 5.21, x: 0) do
              delayed.when_stop_moving(board.vehicles.ground.mine, after: 200.ticks) do
                pipeline.select(
                  top:   92 + middle_group_y_offset + 5,
                  left:    x[col],
                  bottom:  92 + middle_group_y_offset + scaled_squad_size,
                  right: x[col] + middle_group_x_offset + scaled_squad_size
                )

                pipeline.move(y: -scaled_squad_size, x: 0)

                delayed.when_stop_moving(board.vehicles.ground.mine, after:  200.ticks) do
                  cp = board.vehicles.ground.mine.center_point
                  pipeline.select_group(group: 85)
                  pipeline.rotate(x: cp.x, y: cp.y, angle_degrees: -45, max_speed: lowest_ground_speed)
                end
              end
            end
          end
        end
      end
    end

    def build_horizontal_sandwich(row)
      puts "tick #{world.tick_index}: building horizontal sandwich at col #{row}"
      groups                = [81, 82, 83]
      x                     = y = [18, 92, 166]
      factor                = 2.78
      scaled_squad_size     = (54 * factor).ceil
      padding               = 20
      middle_group_x_offset = padding * factor + 30
      middle_group_y_offset = 10
      right_group_x_offset  = middle_group_x_offset * 2
      right_group_y_offset  = 5

      unscaled_squad_size = 60
      pipeline.select(left: 166, top: y[row], right: 166 + unscaled_squad_size, bottom: y[row] + unscaled_squad_size)
      pipeline.assign(group: groups[2])
      pipeline.move(x: right_group_x_offset, y: right_group_y_offset, max_speed: lowest_ground_speed) do
        delayed.when_stop_moving(board.vehicles.ground.mine.group(groups[2]), after:  250.ticks) do
          pipeline.select_group(group: groups[2])
          pipeline.scale(
            x:      166 + right_group_x_offset,
            y:      y[row] + right_group_y_offset,
            factor: factor
          # max_speed: lowest_ground_speed
          )
        end
      end

      pipeline.select(left: 92, top: y[row], right: 92 + unscaled_squad_size, bottom: y[row] + unscaled_squad_size)
      pipeline.assign(group: groups[1])
      pipeline.move(x: middle_group_x_offset, y: middle_group_y_offset, max_speed: lowest_ground_speed) do
        delayed.when_stop_moving(board.vehicles.ground.mine.group(groups[1]), after: 250.ticks) do
          pipeline.select_group(group: groups[1])
          pipeline.scale(
            x:      92 + middle_group_x_offset,
            y:      y[row] + middle_group_y_offset,
            factor: factor
          # max_speed: lowest_ground_speed
          )
        end
      end

      pipeline.select(left: 18, top: y[row], right: 18 + unscaled_squad_size, bottom: y[row] + unscaled_squad_size)
      pipeline.assign(group: groups[0])
      delayed.when_stop_moving(board.vehicles.ground.mine.group(groups[2]), after: 450.ticks) do
        pipeline.select_group(group: groups[0])
        pipeline.scale(x: 18, y: y[row], factor: factor) do
          delayed.when_stop_moving(board.vehicles.ground.mine, after: 600.ticks) do
            pipeline.select_group(group: groups[0])
            pipeline.move(x: (92 + middle_group_x_offset) - 18, y: 0)

            pipeline.select_group(group: groups[2])
            pipeline.move(x: 92 - (middle_group_x_offset + right_group_x_offset) + 5.21, y: 0) do
              delayed.when_stop_moving(board.vehicles.ground.mine, after: 200.ticks) do
                pipeline.select(
                  left:   92 + middle_group_x_offset + 5,
                  top:    y[row],
                  right:  92 + middle_group_x_offset + scaled_squad_size,
                  bottom: y[row] + middle_group_y_offset + scaled_squad_size
                )

                pipeline.move(x: -scaled_squad_size, y: 0)

                delayed.when_stop_moving(board.vehicles.ground.mine, after: 200.ticks) do
                  cp = board.vehicles.ground.mine.center_point
                  pipeline.select_group(group: 85)
                  pipeline.rotate(x: cp.x, y: cp.y, angle_degrees: 45, max_speed: lowest_ground_speed)
                end
              end
            end
          end
        end
      end
    end

    def ticks_per_turn
      300 / 0.6
    end

    def starting_positions_to_movements
      {
        [0, 1, 2] => [
          :horizontal, turns = 0, row = 0
        ],
        [0, 1, 3] => [
          :vertical, turns = 2, col = 0,
          [[3, :down], [0, :down => [[1, :left]]]]
        ],
        [0, 1, 4] => [
          :horizontal, turns = 2, row = 0,
          [1, :right => [[4, :up]]]
        ],
        [0, 1, 5] => [
          :horizontal, turns = 1, row = 0,
          [[5, :up]]
        ],
        [0, 1, 6] => [
          :vertical, turns = 2, col = 1,
          [[6, :right], [1, :down => [[0, :right]]]]
        ],
        [0, 1, 7] => [
          :vertical, turns = 2, col = 1,
          [[1, :down => [[0, :right]]]]
        ],
        [0, 1, 8] => [
          :horizontal, turns = 1, row = 1,
          [[0, :down], [1, :down], [8, :up]]
        ],
        [0, 2, 3] => [
          :horizontal, turns = 2, row = 1,
          [[3, :right => [[0, :down], [2, :down]]]]
        ],
        [0, 2, 4] => [
          :horizontal, turns = 1, row = 0,
          [[4, :up]],
        ],
        [0, 2, 5] => [
          :horizontal, turns = 2, row = 0,
          [[2, :left => [[5, :up]]]]
        ],
        [0, 2, 6] => [
          :vertical, turns = 2, col = 0,
          [[0, :down], [2, :left => [[2, :left]]]]
        ],
        [0, 2, 7] => [
          :horizontal, turns = 1, row = 1,
          [[0, :down], [7, :up], [2, :down]],
        ],
        [0, 2, 8] => [
          :vertical, turns = 2, col = 2,
          [[2, :down], [0, :right => [[0, :right]]]]
        ],
        [0, 3, 4] => [
          :horizontal, turns = 2, row = 1,
          [[4, :right], [3, :right => [[0, :down]]]]
        ],
        [0, 3, 5] => [
          :horizontal, turns = 2, row = 1,
          [[3, :right => [[0, :down]]]]
        ],
        [0, 3, 6] => [
          :vertical, turns = 0, col = 0,
        ],
        [0, 3, 7] => [
          :vertical, turns = 1, col = 0,
          [[7, :left]],
        ],
        [0, 3, 8] => [
          :vertical, turns = 1, col = 1,
          [[0, :right], [3, :right], [8, :left]]
        ],
        [0, 4, 5] => [
          :horizontal, turns = 1, row = 1,
          [[0, :down]]
        ],
        [0, 4, 6] => [
          :vertical, turns = 1, col = 0,
          [[4, :left]]
        ],
        [0, 4, 7] => [
          :vertical, turns = 1, col = 1,
          [[0, :right]]
        ],
        [0, 4, 8] => [
          :vertical, turns = 1, col = 1,
          [[0, :right], [8, :left]]
        ],
        [0, 5, 6] => [
          :vertical, turns = 1, col = 1,
          [[0, :right], [5, :left], [6, :right]]
        ],
        [0, 5, 7] => [
          :vertical, turns = 1, col = 1,
          [[0, :right], [5, :left]],
        ],
        [0, 5, 8] => [
          :vertical, turns = 1, col = 1,
          [[0, :right], [5, :left], [8, :left]]
        ],
        [0, 6, 7] => [
          :vertical, turns = 2, col = 1,
          [[0, :right], [7, :up => [[6, :right]]]]
        ],
        [0, 6, 8] => [
          :horizontal, turns = 2, row = 1,
          [[0, :right => [[0, :down]]], [6, :up], [8, :up]]
        ],
        [0, 7, 8] => [
          :horizontal, turns = 1, row = 1,
          [[0, :down], [7, :up], [8, :up]]
        ],
        [1, 2, 3] => [
          :horizontal, turns = 1, row = 0,
          [[3, :up]],
        ],
        [1, 2, 4] => [
          :horizontal, turns = 2, row = 0,
          [[1, :left => [[4, :up]]]]
        ],
        [1, 2, 5] => [
          :vertical, turns = 2, col = 2,
          [[5, :down], [2, :down => [[1, :right]]]]
        ],
        [1, 2, 6] => [
          :horizontal, turns = 1, row = 1,
          [[6, :up], [1, :down], [2, :down]]
        ],
        [1, 2, 7] => [
          :vertical, turns = 2, col = 2,
          [[7, :right], [2, :down => [[1, :right]]]]
        ],
        [1, 2, 8] => [
          :vertical, turns = 2, col = 2,
          [[2, :down => [[1, :right]]]]
        ],
        [1, 3, 4] => [
          :horizontal, turns = 2, row = 1,
          [[4, :right => [[1, :down]]]]
        ],
        [1, 3, 5] => [
          :horizontal, turns = 1, row = 1,
          [[1, :down]]
        ],
        [1, 3, 6] => [
          :vertical, turns = 1, col = 0,
          [[1, :left]]
        ],
        [1, 3, 7] => [
          :vertical, turns = 1, col = 1,
          [[3, :right]]
        ],
        [1, 3, 8] => [
          :vertical, turns = 1, col = 1,
          [[3, :right], [8, :left]]
        ],
        [1, 4, 5] => [
          :horizontal, turns = 2, row = 1,
          [[4, :right => [[1, :down]]]]
        ],
        [1, 4, 6] => [
          :vertical, turns = 1, col = 1,
          [[6, :right]]
        ],
        [1, 4, 7] => [
          :vertical, turns = 0, col = 1,
        ],
        [1, 4, 8] => [
          :vertical, turns = 1, col = 1,
          [[8, :left]]
        ],
        [1, 5, 6] => [
          :vertical, turns = 1, col = 1,
          [[5, :left], [6, :right]]
        ],
        [1, 5, 7] => [
          :vertical, turns = 1, col = 1,
          [[5, :left]]
        ],
        [1, 5, 8] => [
          :vertical, turns = 1, col = 2,
          [[1, :right]]
        ],
        [1, 6, 7] => [
          :horiozntal, turns = 2, row = 2,
          [[7, :right], [1, :down => [[1, :down]]]]
        ],
        [1, 6, 8] => [
          :horizontal, turns = 1, row = 1,
          [[6, :up], [1, :down], [8, :up]]
        ],
        [1, 7, 8] => [
          :horizontal, turns = 2, row = 2,
          [[7, :left], [1, :down => [[1, :down]]]]
        ],
        [2, 3, 4] => [
          :horizontal, turns = 1, row = 1,
          [[2, :down]]
        ],
        [2, 3, 5] => [
          :horizontal, turns = 2, row = 1,
          [[5, :left => [[2, :down]]]]
        ],
        [2, 3, 6] => [
          :vertical, turns = 1, col = 1,
          [[2, :left], [3, :right], [6, :right]]
        ],
        [2, 3, 7] => [
          :vertical, turns = 1, col = 1,
          [[2, :left], [3, :right]]
        ],
        [2, 3, 8] => [
          :vertical, turns = 1, col = 1,
          [[2, :left], [3, :right], [8, :left]]
        ],
        [2, 4, 5] => [
          :horizontal, turns = 2, row = 1,
          [[4, :left], [5, :left => [[2, :down]]]]
        ],
        [2, 4, 6] => [
          :horizontal, turns = 1, row = 1,
          [[2, :down], [6, :up]]
        ],
        [2, 4, 7] => [
          :vertical, turns = 1, col = 1,
          [[2, :left]]
        ],
        [2, 4, 8] => [
          :vertical, turns = 1, col = 2,
          [[4, :right]]
        ],
        [2, 5, 6] => [
          :vertical, turns = 1, col = 1,
          [[2, :left], [5, :left], [6, :right]]
        ],
        [2, 5, 7] => [
          :vertical, turns = 1, col = 2,
          [[7, :right]]
        ],
        [2, 5, 8] => [
          :vertical, turns = 0, col = 2,
        ],
        [2, 6, 7] => [
          :horizontal, turns = 1, row = 1,
          [[2, :down], [6, :up], [7, :up]]
        ],
        [2, 6, 8] => [
          :horizontal, turns = 2, row = 2,
          [[8, :left], [2, :down => [[2, :down]]]]
        ],
        [2, 7, 8] => [
          :vertical, turns = 2, col = 2,
          [[7, :up => [[7, :right]]]]
        ],
        [3, 4, 5] => [
          :horizontal, turns = 0, row = 1,
        ],
        [3, 4, 6] => [
          :horizontal, turns = 2, row = 1,
          [[4, :right], [3, :right => [[6, :up]]]]
        ],
        [3, 4, 7] => [
          :horizontal, turns = 2, row = 1,
          [[4, :right => [[7, :up]]]]
        ],
        [3, 4, 8] => [
          :horizontal, turns = 1, row = 1,
          [[8, :up]]
        ],
        [3, 5, 6] => [
          :horizontal, turns = 2, row = 1,
          [[6, :right => [[6, :up]]]]
        ],
        [3, 5, 7] => [
          :horizontal, turns = 1, row = 1,
          [[7, :up]]
        ],
        [3, 5, 8] => [
          :horizontal, turns = 2, row = 1,
          [[8, :left => [[8, :up]]]]
        ],
        [3, 6, 7] => [
          :horizontal, turns = 2, row = 2,
          [[7, :right], [3, :right => [[3, :down]]]]
        ],
        [3, 6, 8] => [
          :horizontal, turns = 2, row = 2,
          [[3, :right => [[3, :down]]]]
        ],
        [3, 7, 8] => [
          :horizontal, turns = 1, row = 2,
          [[3, :down]]
        ],
        [4, 5, 6] => [
          :horizontal, turns = 1, row = 1,
          [[6, :up]]
        ],
        [4, 5, 7] => [
          :horizontal, turns = 2, row = 1,
          [[7, :left => [[7, :up]]]]
        ],
        [4, 5, 8] => [
          :horizontal, turns = 2, row = 1,
          [[4, :left], [5, :left => [[8, :up]]]]
        ],

        [4, 6, 7] => [
          :horizontal, turns = 2, row = 2,
          [[4, rotate = [:down, 90]]]
        ],
        [4, 6, 8] => [
          :horizontal, turns = 1, row = 2,
          [[4, :down]]
        ],
        [4, 7, 8] => [
          :horizontal, turns = 2, row = 2,
          [[4, :left => [[4, :down]]]]
        ],
        [5, 6, 7] => [
          :horizontal, turns = 1, row = 2,
          [[5, :down]]
        ],
        [5, 6, 8] => [
          :horizontal, turns = 2, row = 2,
          [[5, :left => [[5, :down]]]]
        ],
        [5, 7, 8] => [
          :horizontal, turns = 2, row = 2,
          [[7, :left], [8, :left => [[5, :down]]]]
        ],
        [6, 7, 8] => [
          :horizontal, turns = 0, row = 2,
        ]

      }
    end

    def lowest_ground_speed
      @lowest_ground_speed ||= [$game.tank_speed, $game.ifv_speed, $game.arrv_speed].min * 0.6
    end


  end
end
