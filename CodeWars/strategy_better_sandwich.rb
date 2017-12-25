require './squadron'
require './potential_utils'
require './emitter'
require './nuke_evasion'

module Strategies
  class BetterSandwich < Base
    include PotentialUtils
    include NukeEvasion

    GROUP_GROUND_SANDWICH = 85
    GROUP_AERIAL_SANDWICH = 86
    GROUP_COMBINED_SANDWICH = 87
    SANDWICH_SCALE_AFTER_BUILD = 1.1

    every 20.ticks do
      update_enemies_by_cell
    end

    def handle_tick
      if world.tick_index == 0
        form_groups
      end

      @count_down ||= 0
      if @count_down.to_i > 0
        @count_down -= 1
        return
      end

      if !formation_ready? && ground_sandwich_ready? && aerial_sandwich_ready?
        move_aerial_to_ground
        @count_down = 10
      end

      if formation_ready?
        check_for_nuke

        if nuke_ready?
          launch_nuke_if_acceptable_enemy_cell_exists?
        end

        move_sandwich
      end
    end

    private

    def ground_sandwich_units
      board.vehicles.mine.group(GROUP_GROUND_SANDWICH)
    end

    def aerial_sandwich_units
      board.vehicles.mine.group(GROUP_AERIAL_SANDWICH)
    end

    attr_reader :enemies_by_cell

    def form_groups
      initiate_ground_sandwich
      initiate_aerial_sandwich
    end

    def move_sandwich
      update_enemies_by_cell
      (cx, cy), enemy_count = enemies_by_cell.first
      cp = Point.from_cell(cx, cy)

      loc = board.vehicles.mine.center_point
      dist = loc.distance_to_point(cp)
      diff = cp - loc
      # v = Vector[diff.x, diff.y]

      # mx, my = *(0.8 * dist * v.normalize) # go 80% of the way to closest enemy unit

      pipeline.select_group(group: GROUP_COMBINED_SANDWICH)
      pipeline.move(x: diff.x, y: diff.y, max_speed: lowest_ground_speed)
      @count_down = 50
    end

    def initiate_aerial_sandwich
      unit_groups = VehicleType.aerial.each_with_object({}) do |unit_type, memo|
        units           = board.vehicles.mine.of_type(unit_type)
        coords_ary      = [18, 92, 166]
        pnt             = units.lefttop_point
        position_number = coords_ary.index(pnt.x.to_i) + coords_ary.index(pnt.y.to_i) * 3
        puts "unit_type: #{unit_type} at position #{position_number}"
        pipeline.select(vehicle_type: unit_type)
        pipeline.assign(group: unit_type + 1)
        pipeline.assign(group: 90 + position_number)
        pipeline.assign(group: GROUP_AERIAL_SANDWICH)
        pipeline.assign(group: GROUP_COMBINED_SANDWICH)
        memo[position_number] = units
      end
      build_formation(:aerial, unit_groups, board.vehicles.mine.aerial, aerial_starting_positions_to_movements, GROUP_AERIAL_SANDWICH)
    end

    def initiate_ground_sandwich
      unit_groups = VehicleType.ground.each_with_object({}) do |unit_type, memo|
        units           = board.vehicles.mine.of_type(unit_type)
        coords_ary      = [18, 92, 166]
        pnt             = units.lefttop_point
        position_number = coords_ary.index(pnt.x.to_i) + coords_ary.index(pnt.y.to_i) * 3
        puts "unit_type: #{unit_type} at position #{position_number}"
        pipeline.select(vehicle_type: unit_type)
        pipeline.assign(group: unit_type + 1)
        pipeline.assign(group: 90 + position_number)
        pipeline.assign(group: GROUP_GROUND_SANDWICH)
        pipeline.assign(group: GROUP_COMBINED_SANDWICH)
        memo[position_number] = units
      end
      build_formation(:ground, unit_groups, board.vehicles.mine.ground, starting_positions_to_movements, GROUP_GROUND_SANDWICH)
    end

    def build_formation(sandwich_type, unit_groups, vehicles_query, positions_dictionary, group)
      resulting_shape, num_turns, row_or_col, movements = positions_dictionary[unit_groups.keys.sort]
      puts "units will be in a #{resulting_shape} formation"
      p movements
      execute_movements(movements) if movements

      builder_method = (sandwich_type == :aerial) ? :build_sandwich2 : :build_sandwich3

      if num_turns.zero?
        send(builder_method, resulting_shape, row_or_col, vehicles_query, group)
      else
        delayed.when_stop_moving(vehicles_query.to_a, after: 250.ticks) do
          send(builder_method, resulting_shape, row_or_col, vehicles_query, group)
        end
      end
    end

    def build_sandwich3(type, rowcol, vehicles_query, sandwich_group)
      puts "tick #{world.tick_index}: building #{type} sandwich at col/row #{rowcol}"
      groups                      = [81, 82, 83]
      blocks                      = [18, 92, 166, 240]
      unscaled_squad_size         = 60
      factor                      = 2.78
      scaled_squad_size           = (54 * factor).ceil
      padding                     = 20
      spacing_between_scaled_rows = 16.78

      position_shift_enum = [0].cycle
      if type == :horizontal
        unit_type_to_min_x            = vehicles_query.select { |v| v.y.round == blocks[rowcol] }.group_by(&:type).transform_values { |vals| vals.map(&:x).min.to_i }
        x                             = unit_type_to_min_x.values.sort
        y                             = Array.new(3) { [18, 92, 166][rowcol] }
        coord_to_unit_type            = x.zip(y).each_with_object({}) do |(x, y), memo|
          memo[[x, y]] = unit_type_to_min_x.rassoc(x).first
        end
        puts "coord to unit type: #{coord_to_unit_type}"
        middle_group_x_offset         = padding * factor + 30
        middle_group_y_offset         = 10
        far_group_x_offset            = middle_group_x_offset * 2
        far_group_y_offset            = 5
        angle_degrees                 = 45
        compaction1_near_group_vector = Point.new((x[1] + middle_group_x_offset) - x[0], 0)
        compaction1_far_group_vector  = compaction1_near_group_vector * -1

        compaction2_selection_width  = 2.0
        compaction2_selection_height = scaled_squad_size + 20
        compaction2_step_x           = spacing_between_scaled_rows
        compaction2_step_y           = 0
        compaction2_dx_enum          = [scaled_squad_size.fdiv(2)].cycle
        compaction2_dy_enum          = position_shift_enum
      else
        unit_type_to_min_y    = vehicles_query.select { |v| v.x.round == blocks[rowcol] }.group_by(&:type).transform_values { |vals| vals.map(&:y).min.to_i }
        x                     = Array.new(3) { [18, 92, 166][rowcol] }
        y                     = unit_type_to_min_y.values.sort
        coord_to_unit_type            = x.zip(y).each_with_object({}) do |(x, y), memo|
          memo[[x, y]] = unit_type_to_min_y.rassoc(y).first
        end
        puts "coord to unit type: #{coord_to_unit_type}"
        middle_group_x_offset = 10
        middle_group_y_offset = padding * factor + 30
        far_group_x_offset    = 5
        far_group_y_offset    = middle_group_y_offset * 2
        angle_degrees         = -45

        compaction1_near_group_vector = Point.new(0, (y[1] + middle_group_y_offset) - y[0])
        compaction1_far_group_vector  = compaction1_near_group_vector * -1

        compaction2_selection_width  = scaled_squad_size + 20
        compaction2_selection_height = 2.0
        compaction2_step_x           = 0
        compaction2_step_y           = spacing_between_scaled_rows
        compaction2_dx_enum          = position_shift_enum
        compaction2_dy_enum          = [scaled_squad_size.fdiv(2)].cycle
      end


      pipeline.select(left:         x[2], top: y[2],
                      right:        x[2] + unscaled_squad_size, bottom: y[2] + unscaled_squad_size,
                      vehicle_type: coord_to_unit_type[[x[2], y[2]]])
      pipeline.assign(group: groups[2])
      pipeline.move(x: far_group_x_offset, y: far_group_y_offset, max_speed: lowest_ground_speed) do
        delayed.when_stop_moving(vehicles_query.group(groups[2]), after: 250.ticks) do
          pipeline.select_group(group: groups[2])
          pipeline.scale(
            x:      x[2] + far_group_x_offset,
            y:      y[2] + far_group_y_offset,
            factor: factor
          ) do
            delayed.when_stop_moving(vehicles_query.group(groups[2]), after: 100.ticks) { @far_group_scaled = true }
          end

        end
      end

      pipeline.select(left:         x[1], top: y[1],
                      right:        x[1] + unscaled_squad_size, bottom: y[1] + unscaled_squad_size,
                      vehicle_type: coord_to_unit_type[[x[1], y[1]]])
      pipeline.assign(group: groups[1])
      pipeline.move(x: middle_group_x_offset, y: middle_group_y_offset, max_speed: lowest_ground_speed) do
        delayed.when_stop_moving(vehicles_query.group(groups[1]), after: 250.ticks) do
          pipeline.select_group(group: groups[1])
          pipeline.scale(
            x:      x[1] + middle_group_x_offset,
            y:      y[1] + middle_group_y_offset,
            factor: factor
          ) do
            delayed.when_stop_moving(vehicles_query.group(groups[1]), after: 100.ticks) { @middle_group_scaled = true }
          end
        end
      end

      pipeline.select(left:         x[0], top: y[0],
                      right:        x[0] + unscaled_squad_size, bottom: y[0] + unscaled_squad_size,
                      vehicle_type: coord_to_unit_type[[x[0], y[0]]])
      pipeline.assign(group: groups[0])
      pipeline.scale(x: x[0], y: y[0], factor: factor)
      delayed.when_stop_moving(vehicles_query.group(groups[0]), after: 100.ticks) { @near_group_scaled = true }

      all_moved = -> { @near_group_scaled && @middle_group_scaled && @far_group_scaled }
      delayed.when_proc(all_moved) do
        pipeline.select_group(group: groups[0])
        pipeline.move_by(point: compaction1_near_group_vector)

        pipeline.select_group(group: groups[2])
        pipeline.move_by(point: compaction1_far_group_vector) do
          delayed.when_stop_moving(vehicles_query, after: 100.ticks) do
            tlx, tly = vehicles_query.lefttop_point.to_a
            u1, *urest = vehicles_query.map(&:type).to_a.uniq

            # move left/near part
            0.upto(4) do |rowcol|
              pipeline.select(
                left:   tlx + compaction2_step_x * rowcol - 1,
                top:    tly + compaction2_step_y * rowcol - 1,
                right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                vehicle_type: u1
              )
              urest.each do |ut|
                pipeline.add_to_selection(
                  left:   tlx + compaction2_step_x * rowcol - 1,
                  top:    tly + compaction2_step_y * rowcol - 1,
                  right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                  bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                  vehicle_type: ut
                )
              end

              pipeline.move(x: compaction2_dx_enum.next, y: compaction2_dy_enum.next)
            end

            # move right/far part
            5.upto(9) do |rowcol|
              pipeline.select(
                left:   tlx + compaction2_step_x * rowcol - 1,
                top:    tly + compaction2_step_y * rowcol - 1,
                right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                vehicle_type: u1
              )
              urest.each do |ut|
                pipeline.add_to_selection(
                  left:   tlx + compaction2_step_x * rowcol - 1,
                  top:    tly + compaction2_step_y * rowcol - 1,
                  right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                  bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                  vehicle_type: ut
                )
              end

              pipeline.move(x: -compaction2_dx_enum.next, y: -compaction2_dy_enum.next)
            end

            # last one, will wait for this movement to finish
            rowcol = 10
            pipeline.select(
              left:   tlx + compaction2_step_x * rowcol - 1,
              top:    tly + compaction2_step_y * rowcol - 1,
              right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
              bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
              vehicle_type: u1
            )

            urest.each do |ut|
              pipeline.add_to_selection(
                left:   tlx + compaction2_step_x * rowcol - 1,
                top:    tly + compaction2_step_y * rowcol - 1,
                right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                vehicle_type: ut
              )
            end

            pipeline.move(x: -compaction2_dx_enum.next, y: -compaction2_dy_enum.next) do
              delayed.when_stop_moving(vehicles_query, after: 200.ticks) do
                cp = vehicles_query.center_point
                pipeline.select_group(group: sandwich_group)
                pipeline.rotate(x: cp.x, y: cp.y, angle_degrees: angle_degrees, max_speed: lowest_ground_speed) do
                  delayed.when_stop_moving(vehicles_query, after: 100.ticks) do
                    pipeline.select_group(group: sandwich_group)
                    pipeline.scale(x: cp.x, y: cp.y, factor: SANDWICH_SCALE_AFTER_BUILD) do
                      delayed.when_stop_moving(vehicles_query, after: 100.ticks) do
                        @ground_sandwich_ready = true
                        @ground_sandwich_squadron           = Squadron.new(units: board.vehicles.mine.group(sandwich_group).to_a, group: sandwich_group)
                        friendly_squadrons << @ground_sandwich_squadron
                      end
                    end
                  end
                end
              end
            end
          end
        end
      end
    end

    def build_sandwich2(type, rowcol, vehicles_query, sandwich_group)
      puts "tick #{world.tick_index}: building #{type} sandwich at col/row #{rowcol}"
      groups                      = [71, 72]
      blocks                      = [18, 92, 166, 240]
      unscaled_squad_size         = 60
      factor                      = 2.78
      scaled_squad_size           = (54 * factor).ceil
      padding                     = 20
      spacing_between_scaled_rows = 16.78

      position_shift_enum = [0].cycle
      if type == :horizontal
        unit_type_to_min_x            = vehicles_query.select { |v| v.y.round == blocks[rowcol] }.group_by(&:type).transform_values { |vals| vals.map(&:x).min.to_i }
        x                             = unit_type_to_min_x.values.sort
        y                             = Array.new(2) { [18, 92, 166][rowcol] }
        coord_to_unit_type            = x.zip(y).each_with_object({}) do |(x, y), memo|
          memo[[x, y]] = unit_type_to_min_x.rassoc(x).first
        end
        puts "coord to unit type: #{coord_to_unit_type}"
        middle_group_x_offset         = padding * factor + 30
        middle_group_y_offset         = 10
        angle_degrees                 = 45

        compaction1_near_group_vector = Point.new((x[1] + middle_group_x_offset) - x[0], 0) / 2
        compaction1_far_group_vector  = compaction1_near_group_vector * -1

        compaction2_selection_width  = 2.0
        compaction2_selection_height = scaled_squad_size + 20
        compaction2_step_x           = spacing_between_scaled_rows
        compaction2_step_y           = 0
        compaction2_dx_enum          = [scaled_squad_size.fdiv(2)].cycle
        compaction2_dy_enum          = position_shift_enum
      else
        unit_type_to_min_y    = vehicles_query.select { |v| v.x.round == blocks[rowcol] }.group_by(&:type).transform_values { |vals| vals.map(&:y).min.to_i }
        x                     = Array.new(2) { [18, 92, 166][rowcol] }
        y                     = unit_type_to_min_y.values.sort
        coord_to_unit_type            = x.zip(y).each_with_object({}) do |(x, y), memo|
          memo[[x, y]] = unit_type_to_min_y.rassoc(y).first
        end
        puts "coord to unit type: #{coord_to_unit_type}"
        middle_group_x_offset = 5
        middle_group_y_offset = padding * factor + 30
        angle_degrees         = -45

        compaction1_near_group_vector = Point.new(0, (y[1] + middle_group_y_offset) - y[0]) / 2
        compaction1_far_group_vector  = compaction1_near_group_vector * -1

        compaction2_selection_width  = scaled_squad_size + 20
        compaction2_selection_height = 2.0
        compaction2_step_x           = 0
        compaction2_step_y           = spacing_between_scaled_rows
        compaction2_dx_enum          = position_shift_enum
        compaction2_dy_enum          = [scaled_squad_size.fdiv(2)].cycle
      end


      pipeline.select(left:         x[1], top: y[1],
                      right:        x[1] + unscaled_squad_size, bottom: y[1] + unscaled_squad_size,
                      vehicle_type: coord_to_unit_type[[x[1], y[1]]])
      pipeline.assign(group: groups[1])
      pipeline.move(x: middle_group_x_offset, y: middle_group_y_offset, max_speed: lowest_ground_speed) do
        delayed.when_stop_moving(vehicles_query.group(groups[1]), after: 50.ticks) do
          pipeline.select_group(group: groups[1])
          pipeline.scale(
            x:      x[1] + middle_group_x_offset,
            y:      y[1] + middle_group_y_offset,
            factor: factor
          ) do
            delayed.when_stop_moving(vehicles_query.group(groups[1]), after: 50.ticks) { @middle_group_scaled = true }
          end
        end
      end

      pipeline.select(left:         x[0], top: y[0],
                      right:        x[0] + unscaled_squad_size, bottom: y[0] + unscaled_squad_size,
                      vehicle_type: coord_to_unit_type[[x[0], y[0]]])
      pipeline.assign(group: groups[0])
      pipeline.scale(x: x[0], y: y[0], factor: factor)
      delayed.when_stop_moving(vehicles_query.group(groups[0]), after: 50.ticks) { @near_group_scaled = true }

      all_moved = -> { @near_group_scaled && @middle_group_scaled }
      delayed.when_proc(all_moved) do
        pipeline.select_group(group: groups[0])
        pipeline.move_by(point: compaction1_near_group_vector)

        pipeline.select_group(group: groups[1])
        pipeline.move_by(point: compaction1_far_group_vector) do
          delayed.when_stop_moving(vehicles_query, after: 50.ticks) do
            tlx, tly = vehicles_query.lefttop_point.to_a
            u1, *urest = vehicles_query.map(&:type).to_a.uniq

            # move left/near part
            0.upto(4) do |rowcol|
              pipeline.select(
                left:   tlx + compaction2_step_x * rowcol - 1,
                top:    tly + compaction2_step_y * rowcol - 1,
                right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                vehicle_type: u1
              )
              urest.each do |ut|
                pipeline.add_to_selection(
                  left:   tlx + compaction2_step_x * rowcol - 1,
                  top:    tly + compaction2_step_y * rowcol - 1,
                  right:  tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                  bottom: tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                  vehicle_type: ut
                )
              end

              pipeline.move(x: compaction2_dx_enum.next, y: compaction2_dy_enum.next)
            end

            # move right/far part
            5.upto(9) do |rowcol|
              pipeline.select(
                left:         tlx + compaction2_step_x * rowcol - 1,
                top:          tly + compaction2_step_y * rowcol - 1,
                right:        tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                bottom:       tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                vehicle_type: u1
              )
              urest.each do |ut|
                pipeline.add_to_selection(
                  left:         tlx + compaction2_step_x * rowcol - 1,
                  top:          tly + compaction2_step_y * rowcol - 1,
                  right:        tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                  bottom:       tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                  vehicle_type: ut
                )
              end

              pipeline.move(x: -compaction2_dx_enum.next, y: -compaction2_dy_enum.next)
            end

            # last one, will wait for this movement to finish
            rowcol = 10
            pipeline.select(
              left:         tlx + compaction2_step_x * rowcol - 1,
              top:          tly + compaction2_step_y * rowcol - 1,
              right:        tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
              bottom:       tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
              vehicle_type: u1
            )
            urest.each do |ut|
              pipeline.add_to_selection(
                left:         tlx + compaction2_step_x * rowcol - 1,
                top:          tly + compaction2_step_y * rowcol - 1,
                right:        tlx + compaction2_step_x * rowcol - 1 + compaction2_selection_width,
                bottom:       tly + compaction2_step_y * rowcol - 1 + compaction2_selection_height,
                vehicle_type: ut
              )
            end
            pipeline.move(x: -compaction2_dx_enum.next, y: -compaction2_dy_enum.next) do
              delayed.when_stop_moving(vehicles_query, after: 100.ticks) do
                cp = vehicles_query.center_point
                pipeline.select_group(group: sandwich_group)
                pipeline.rotate(x: cp.x, y: cp.y, angle_degrees: angle_degrees, max_speed: lowest_ground_speed) do
                  delayed.when_stop_moving(vehicles_query, after: 100.ticks) do
                    pipeline.select_group(group: sandwich_group)
                    pipeline.scale(x: cp.x, y: cp.y, factor: SANDWICH_SCALE_AFTER_BUILD) do
                      delayed.when_stop_moving(vehicles_query, after: 100.ticks) do
                        @aerial_sandwich_ready = true
                        @aerial_sandwich_squadron = Squadron.new(units: board.vehicles.mine.group(sandwich_group).to_a, group: sandwich_group)
                        friendly_squadrons << @aerial_sandwich_squadron
                      end
                    end
                  end
                end
              end
            end

          end
        end
      end
    end

    def move_aerial_to_ground
      return if @moving_aerial_to_ground
      @moving_aerial_to_ground = true
      squadron = @aerial_sandwich_squadron

      loc = squadron.location
      dest = @ground_sandwich_squadron.location
      pipeline.select_group(group: squadron.group)
      pipeline.move_by(point: dest - loc, max_speed: squadron.min_speed) do
        delayed.when_stop_moving(squadron.units) do
          @formation_ready = true
          @moving_aerial_to_ground = false
        end
      end
    end

    def execute_movements(movements)
      movements.each do |pos, direction|
        case direction
        when Array
          execute_rotation(pos, *direction)
        when Hash
          fail 'moving sequences no longer supported'
        else
          execute_movement(pos, direction)
        end

      end
    end

    def execute_rotation(pos, center_pos, angle_degrees)
      center_points_x = center_points_y = [18 + 27, 92 + 27, 166 + 27]
      # center_points_x = center_points_y = [18, 92, 166]
      center_point = Point.new(center_points_x[pos % 3], center_points_y[pos / 3])

      pivot_point_diffs = { up: [0, -74], down: [0, 74], right: [74, 0], left: [-74, 0] }
      center_point.x    += pivot_point_diffs[center_pos][0]
      center_point.y    += pivot_point_diffs[center_pos][1]

      puts "rotating #{angle_degrees} degrees against #{center_point}"
      pipeline.select_group(group: 90 + pos)
      pipeline.rotate(x: center_point.x, y: center_point.y, angle_degrees: angle_degrees)
    end

    def execute_movement(pos, direction)
      movement_vectors = {
        up: [0, -74],
        down: [0, 74],
        right: [74, 0],
        left: [-74, 0],
        downright: [74, 74],
        downleft: [-74, 74],
      }
      pipeline.select_group(group: 90 + pos)
      x, y = movement_vectors[direction]

      pipeline.move(x: x, y: y, max_speed: lowest_ground_speed)
    end

    def base_potential_map
      width_in_cells      = world.width / 32
      height_in_cells     = world.height / 32
      @base_potential_map ||= create_map do |x, y|
        effects = [0.0]
        effects.push(map_edge_formula(x)) if x <= MAP_EDGE_EFFECT_RADIUS
        effects.push(map_edge_formula(width_in_cells - x - 1)) if (width_in_cells - x - 1) < MAP_EDGE_EFFECT_RADIUS
        effects.push(map_edge_formula(y)) if y <= MAP_EDGE_EFFECT_RADIUS
        effects.push(map_edge_formula(height_in_cells - y - 1)) if (height_in_cells - y - 1) < MAP_EDGE_EFFECT_RADIUS

        effects.max_by(&:abs)
        # WEATHER_COEFFICIENTS[world.weather_by_cell_x_y[y][x]]   # TODO: use these
      end

    end

    def choose_destination_cell_for(squadron)
      emitters             = field_emitters_for(squadron)
      sq_location          = squadron.location
      field_value_to_cells = all_cells.each_with_object({}) do |(x, y), result|
        location = Point.from_cell(x, y)
        next unless location.distance_to_point(sq_location) < SQUADRON_MOVE_RADIUS
        combined_field_value = emitters.select { |e| e.within_range_of(location) }.reduce(0) do |memo, emitter|
          memo + emitter.value_at_point(location, squadron: squadron)
        end

        field_value         = (combined_field_value + base_potential_map[x][y]).round(2)
        result[field_value] ||= []
        result[field_value] << location
      end

      chosen_cell = field_value_to_cells.max_by(&:first).last.sample

      squadron.destination = chosen_cell

      chosen_cell
    end

    def all_cells
      @all_cells ||= begin
        cell_range = (0...32).to_a
        cell_range.product(cell_range)
      end
    end

    def field_emitters_for(squadron)
      artificial_field_emitters_for(squadron) +
        enemy_cell_emitters_for(squadron) +
        friendly_squadron_emitters(except: squadron)
    end

    def enemy_cell_emitters_for(squadron)
      enemies_by_cell.map do |(x, y), enemy_count|
        # we're much stronger, go in and finish them
        # if squadron.aerial? && squadron.total_health.fdiv(enemies_at_cell(x, y).map(&:durability).reduce(:+)) >= 2.0
        #   LinearFalloffEmitter.new(
        #     location:         Point.from_cell(x, y),
        #     effect_radius:    50,
        #     max_effect_value: 60.0
        #   )
        # else # keep distance
          SafeDistanceEmitter.new(
            location:                      Point.from_cell(x, y),
            max_effect_value:              BASE_ENEMY_CELL_ATTRACTION * enemy_count,
            effect_radius:                 ENEMY_CELL_EFFECT_RADIUS,
            preferable_distance:           PREFERABLE_DISTANCE_FROM_ENEMY_CELL,
            preferable_distance_variation: PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION
          )
        # end
      end
    end

    def artificial_field_emitters_for(squadron)
      [].tap do |emitters|
        emitters.push(LinearFalloffEmitter.new(
          location:         Point.new(world.width - 100, world.height - 100),
          effect_radius:    world.width,
          max_effect_value: 100.0
        )) if world.tick_index.between?(0, 2000)

        emitters.push(LinearFalloffEmitter.new(
          location:         board.vehicles.mine.arrvs.center_point,
          effect_radius:    world.width,
          max_effect_value: 200.0
        )) if squadron.low_health? && board.vehicles.mine.arrvs.count > 10
      end
    end

    def clusters
      @clusters ||= []
    end

    def map_edge_formula(cur_distance, safe_distance: MAP_EDGE_EFFECT_RADIUS)
      linear_decay(cur_distance, BASE_MAP_EDGE_ATTRACTION, safe_distance)
    end

    def linear_decay(cur_distance, base_value, effect_radius)
      if cur_distance.between?(0, effect_radius)
        base_value * (1.0 - (cur_distance.fdiv(effect_radius)))
      else
        0.0
      end
    end

    SQUADRON_MOVE_RADIUS = 2.5.cells

    MAP_EDGE_EFFECT_RADIUS   = 3.0
    BASE_MAP_EDGE_ATTRACTION = -1.5

    ENEMY_CELL_EFFECT_RADIUS                      = 850.meters
    BASE_ENEMY_CELL_ATTRACTION                    = 1.0
    PREFERABLE_DISTANCE_FROM_ENEMY_CELL           = 90.meters
    PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION = 10.meters

    FRIENDLY_SQUAD_EFFECT_RADIUS   = 3.5.cells
    BASE_FRIENDLY_SQUAD_ATTRACTION = -25.5


    def friendly_squadron_emitters(except: nil)
      []
    end

    def friendly_squadrons
      @friendly_squadrons ||= []
    end

    def enemies_at_cell(x, y)
      board.vehicles.not_mine.at_cell(x, y)
    end

    def update_enemies_by_cell
      @enemies_by_cell = count_units_by_cell(board.vehicles.not_mine)
    end

    def count_units_by_cell(units)
      units.each_with_object(Hash.new(0)) do |v, memo|
        memo[[v.x.to_i / 32, v.y.to_i / 32]] += 1
      end
    end

    def enemy_units
      board.vehicles.not_mine
    end

    def lowest_ground_speed
      @lowest_ground_speed ||= [$game.tank_speed, $game.ifv_speed, $game.arrv_speed].min * 0.6
    end


    WEATHER_COEFFICIENTS = {
      0 => 0,
      1 => -1, # clouds
      2 => -3, # rain
    }

    def aerial_starting_positions_to_movements
      @calculator ||= AerialMovementCalculator.new
    end

    class AerialMovementCalculator
      def [](positions)
        (x1, y1), (x2, y2) = positions.map{|pos| [pos % 3, pos / 3]}

        p positions
        if x1 == x2 && (y1 - y2).abs == 1
          [:vertical, turns = 0, row = x1]
        elsif y1 == y2 && (x1 - x2).abs == 1
          [:horizontal, turns = 0, row = y1]
        elsif y1 == y2 && x2 - x1 == 2
          [
            :horizontal, turns = 1, row = y2,
            [[positions[0], :right]]
          ]
        elsif y1 == y2 && x1 - x2 == 2
          [
            :horizontal, turns = 1, row = y2,
            [[positions[1], :right]]
          ]
        elsif x1 == x2 && y2 - y1 == 2
          [
            :vertical, turns = 1, row = x1,
            [[positions[0], :down]]
          ]
        elsif x1 == x2 && y1 - y2 == 2
          [
            :vertical, turns = 1, row = x1,
            [[positions[1], :down]]
          ]
        elsif y2 - y1 == 1 && (x2 - x1).abs == 1
          [
            :horizontal, turns = 1, row = y2,
            [[positions[0], :down]]
          ]
        elsif y2 - y1 == 1 && x2 - x1 == 2
          [
            :horizontal, turns = 1, row = y2,
            [[positions[0], :down], [positions[1], :left]]
          ]
        elsif y2 - y1 == 1 && x1 - x2 == 2
          [
            :horizontal, turns = 1, row = y2,
            [[positions[0], :down], [positions[1], :right]]
          ]
        elsif y2 - y1 == 2 && x2 - x1 == 2
          [
            :horizontal, turns = 1, row = y2 - 1,
            [[positions[1], :up], [positions[0], :downright]]
          ]
        elsif y2 - y1 == 2 && x1 - x2 == 2
          [
            :horizontal, turns = 1, row = y2 - 1,
            [[positions[1], :up], [positions[0], :downleft]]
          ]
        elsif y2 - y1 == 2 && x2 - x1 == 1
          [
            :vertical, turns = 1, col = x2 - 1,
            [[positions[1], :left], [positions[0], :down]]
          ]
        elsif y2 - y1 == 2 && x1 - x2 == 1
          [
            :vertical, turns = 1, col = y2 - 1,
            [[positions[1], :right], [positions[0], :down]]
          ]
        else
          fail "unknown positions: #{positions}"
        end
      end
    end

    def starting_positions_to_movements
      {
        [0, 1, 2] => [
          :horizontal, turns = 0, row = 0
        ],
        [0, 1, 3] => [
          :horizontal, turns = 1.5, row = 1,
          [[1, rotate = [:down, 90]], [0, rotate = [:down, 90]]]
        ],
        [0, 1, 4] => [
          :horizontal, turns = 2, row = 0,
          [[4, rotate = [:up, -90]]]
        ],
        [0, 1, 5] => [
          :horizontal, turns = 1, row = 0,
          [[5, :up]]
        ],
        [0, 1, 6] => [
          :horizontal, turns = 2, row = 1,
          [[1, rotate = [:down, 90]], [0, rotate = [:down, 90]], [6, :up]]
        ],
        [0, 1, 7] => [
          :horizontal, turns = 2, row = 1,
          [[0, rotate = [:right, -90]]]
        ],
        [0, 1, 8] => [
          :horizontal, turns = 1, row = 1,
          [[0, :down], [1, :down], [8, :up]]
        ],
        [0, 2, 3] => [
          :horizontal, turns = 2, row = 0,
          [[3, rotate = [:up, -90]]]
        ],
        [0, 2, 4] => [
          :horizontal, turns = 1, row = 0,
          [[4, :up]],
        ],
        [0, 2, 5] => [
          :horizontal, turns = 2, row = 0,
          [[5, rotate = [:up, 90]]]
        ],
        [0, 2, 6] => [
          :horizontal, turns = 2, row = 1,
          [[0, :down], [2, :down], [6, rotate = [:up, -90]]]
        ],
        [0, 2, 7] => [
          :horizontal, turns = 1, row = 1,
          [[0, :down], [7, :up], [2, :down]],
        ],
        [0, 2, 8] => [
          :horizontal, turns = 2, row = 1,
          [[2, :down], [0, :down], [8, rotate = [:up, 90]]]
        ],
        [0, 3, 4] => [
          :horizontal, turns = 2, row = 0,
          [[4, rotate = [:up, -90]], [3, rotate = [:up, -90]]]
        ],
        [0, 3, 5] => [
          :horizontal, turns = 2, row = 1,
          [[0, rotate = [:down, 90]]]
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
          [[0, :right], [6, rotate = [:right, 90]]]
        ],
        [0, 6, 8] => [
          :horizontal, turns = 2, row = 1,
          [[0, rotate = [:down, 90]], [6, :up], [8, :up]]
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
          [[4, rotate = [:up, 90]]]
        ],
        [1, 2, 5] => [
          :horizontal, turns = 2, row = 0,
          [[5, rotate = [:up, -90]]]
        ],
        [1, 2, 6] => [
          :horizontal, turns = 1, row = 1,
          [[6, :up], [1, :down], [2, :down]]
        ],
        [1, 2, 7] => [
          :vertical, turns = 2, col = 1,
          [[2, rotate = [:left, 90]]]
        ],
        [1, 2, 8] => [
          :vertical, turns = 2, col = 2,
          [[1, rotate = [:right, -90]]]
        ],
        [1, 3, 4] => [
          :horizontal, turns = 2, row = 1,
          [[1, rotate = [:down, 90]]]
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
          [[1, rotate = [:down, -90]]]
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
          :vertical, turns = 2, col = 1,
          [[6, rotate = [:right, 90]]]
        ],
        [1, 6, 8] => [
          :horizontal, turns = 1, row = 1,
          [[6, :up], [1, :down], [8, :up]]
        ],
        [1, 7, 8] => [
          :vertical, turns = 2, col = 1,
          [[8, rotate = [:left, -90]]]
        ],
        [2, 3, 4] => [
          :horizontal, turns = 1, row = 1,
          [[2, :down]]
        ],
        [2, 3, 5] => [
          :horizontal, turns = 2, row = 1,
          [[2, rotate = [:down, -90]]]
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
          [[2, rotate = [:down, 90]]]
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
          :horizontal, turns = 2, row = 1,
          [[8, :up], [6, :up], [2, rotate = [:down, -90]]]
        ],
        [2, 7, 8] => [
          :vertical, turns = 2, col = 2,
          [[7, rotate = [:right, 90]]]
        ],
        [3, 4, 5] => [
          :horizontal, turns = 0, row = 1,
        ],
        [3, 4, 6] => [
          :horizontal, turns = 2, row = 2,
          [[4, rotate = [:down, 90]], [3, rotate = [:down, 90]]]
        ],
        [3, 4, 7] => [
          :horizontal, turns = 2, row = 1,
          [[7, rotate = [:up, -90]]]
        ],
        [3, 4, 8] => [
          :horizontal, turns = 1, row = 1,
          [[8, :up]]
        ],
        [3, 5, 6] => [
          :horizontal, turns = 2, row = 1,
          [[6, rotate = [:up, -90]]]
        ],
        [3, 5, 7] => [
          :horizontal, turns = 1, row = 1,
          [[7, :up]]
        ],
        [3, 5, 8] => [
          :horizontal, turns = 2, row = 1,
          [[8, rotate = [:up, 90]]]
        ],
        [3, 6, 7] => [
          :horizontal, turns = 2, row = 1,
          [[7, rotate = [:up, -90]], [6, rotate = [:up, -90]]]
        ],
        [3, 6, 8] => [
          :horizontal, turns = 2, row = 2,
          [[3, rotate = [:down, 90]]]
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
          [[7, rotate = [:up, 90]]]
        ],
        [4, 5, 8] => [
          :horizontal, turns = 2, row = 1,
          [[8, rotate = [:up, -90]]]
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
          [[4, rotate = [:down, -90]]]
        ],
        [5, 6, 7] => [
          :horizontal, turns = 1, row = 2,
          [[5, :down]]
        ],
        [5, 6, 8] => [
          :horizontal, turns = 2, row = 2,
          [[5, rotate = [:down, -90]]]
        ],
        [5, 7, 8] => [
          :horizontal, turns = 2, row = 2,
          [[5, rotate = [:down, 90]]]
        ],
        [6, 7, 8] => [
          :horizontal, turns = 0, row = 2,
        ]
      }
    end

    def launch_nuke_if_acceptable_enemy_cell_exists?
      return if world.tick_index < 600

      strikable_points = enemies_by_cell.keys.select do |x, y|
        friendly_squadrons.any? { |s| s.see_point?(Point.from_cell(x, y)) }
      end.map { |x, y| Point.from_cell(x, y) }

      nukable_damage_threshold = 700
      strikable_points.sort_by { |strike_point| board.enemy_damage_at_point(strike_point) - board.ally_damage_at_point(strike_point) }.reverse_each do |strike_point|
        next if board.enemy_damage_at_point(strike_point) < nukable_damage_threshold

        highlighters = friendly_squadrons.flat_map(&:units).select do |unit|
          unit.effective_vision_range > unit.distance_to_point(strike_point)
        end.sort_by do |unit|
          striking_distance = unit.effective_vision_range - unit.distance_to_point(strike_point)
          if striking_distance.between?(unit.effective_vision_range * 0.5, unit.effective_vision_range * 0.8)
            striking_distance * 30 # more weight
          else
            striking_distance
          end
        end
        highlighter  = highlighters.last # closest
        # highlighter  = highlighters.first # furthest away

        dist  = highlighter.distance_to_point(strike_point)
        range = highlighter.effective_vision_range
        if dist < range
          puts '-------------'
          puts "MISSILE AWAY!"
          puts '-------------'
          pipeline.priority do |priority_pipeline|
            priority_pipeline.select
            priority_pipeline.stop_movement
            priority_pipeline.tactical_nuke(x: strike_point.x, y: strike_point.y, vehicle_id: highlighter.id)
            scale_out_from_the_nuke(strike_point.x, strike_point.y, priority_pipeline)
          end
          @count_down = 30
          return
        else
          puts "Could have launched the nuke, but out of range of our fighters. (#{dist} vs #{range})"
        end
      end
    end

    def wandering_groups
      [].tap do |groups|
        groups << @aerial_sandwich_squadron if aerial_sandwich_ready?
        groups << @ground_sandwich_squadron if ground_sandwich_ready?
      end
    end

    def aerial_sandwich_ready?
      @aerial_sandwich_ready
    end

    def ground_sandwich_ready?
      @ground_sandwich_ready
    end

    def formation_ready?
      @formation_ready
    end

  end
end
