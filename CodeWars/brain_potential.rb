require './brain_base'

module Brains
  class Potential < Base
    ENEMY_CELL_EFFECT_RADIUS                      = 350.meters
    BASE_ENEMY_CELL_ATTRACTION                    = 1.0
    PREFERABLE_DISTANCE_FROM_ENEMY_CELL           = 100.meters
    PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION = 10.meters

    def initialize_brain
      @didnt_move_turns = 0
    end

    def handle_move
      unless scheduled_actions.empty?
        sa = scheduled_actions.shift
        case sa.action_type
        when :shrink
          shrink
        when :rotate_left
          rotate(:left)
        when :rotate_right
          rotate(:right)
        else
          # do nothing
        end

        wait_for(sa.cooldown)
        return
      end

      if @didnt_move_turns > 100 # apparently we're stuck. Switch to no-op mode to not waste cpu
        deactivate_brain
        return
      end

      loc = squadron.location

      dest_point = choose_destination_cell_for(squadron, loc)
      diff_point = dest_point - loc
      if diff_point.x.to_i == 0 && diff_point.y.to_i == 0
        @didnt_move_turns += 1
      else
        move_by(diff_point)
        @didnt_move_turns = 0
      end
      wait_for(time_until_next_move_calculation)
    end

    private

    def time_until_next_move_calculation
      200.ticks
    end

    def move_by(diff_point)
      pipeline.select_group(group: squadron.group)
      pipeline.move_by(point: diff_point, max_speed: preferred_speed)
    end

    def preferred_speed
      squadron.min_speed
    end

    def shrink
      cp = squadron.location
      pipeline.select_group(group: squadron.group)
      pipeline.scale(x: cp.x, y: cp.y, factor: 0.1)
    end

    def rotate(direction)
      angle = direction == :left ? -60 : 60

      cp = squadron.location
      pipeline.select_group(group: squadron.group)
      pipeline.rotate(x: cp.x, y: cp.y, angle_degrees: angle)
    end

    def choose_destination_cell_for(squadron, loc)
      emitters             = field_emitters_for(squadron)
      field_value_to_cells = all_cells.each_with_object({}) do |(x, y), result|
        location = Point.from_cell(x, y)
        next unless location.distance_to_point(loc) < squadron_move_radius * 3
        combined_field_value = emitters.select { |e| e.within_range_of(location) }.reduce(0) do |memo, emitter|
          memo + emitter.value_at_point(location, squadron: squadron)
        end

        field_value         = (combined_field_value + base_potential_for_cell(x, y)).round(2)
        result[field_value] ||= []
        result[field_value] << location
      end

      chosen_cell = field_value_to_cells.max_by(&:first).last.sample

      if $localhost && AppSettings.rewind
        all_field_value_to_cells = all_cells.each_with_object({}) do |(x, y), result|
          location = Point.from_cell(x, y)
          combined_field_value = emitters.select { |e| e.within_range_of(location) }.reduce(0) do |memo, emitter|
            memo + emitter.value_at_point(location, squadron: squadron)
          end

          field_value         = (combined_field_value + base_potential_for_cell(x, y)).round(2)
          result[field_value] ||= []
          result[field_value] << location
        end

        show_field_values(loc, chosen_cell, all_field_value_to_cells)
        show_squadron_units
        rew.message("tick #{$world.tick_index}\n")
        rew.message("Squadron of #{VehicleType.name(squadron.unit_type)} (#{squadron.id}), using #{self.class.name} (#{id}) didnt_move: #{@didnt_move_turns}, cooldown: #{@turn_cooldown}")
      end

      squadron.destination = chosen_cell

      chosen_cell
    end

    def base_potential_for_cell(cx, cy)
      strategy.base_potential_map[cx][cy]
    end

    def squadron_move_radius
      1.0.cells
    end

    def show_squadron_units
      squadron.units.select(&:alive?).each do |unit|
        rew.circle(unit.x, unit.y, 3, Color.yellow, 5)
      end
    end

    def maybe_dump_fields(field_value_to_cells)
      dump_interval = 20
      @dump_timeout ||= dump_interval

      if @dump_timeout == 0
        GnuplotDumper.dump(field_value_to_cells)
        @dump_timeout = dump_interval
      else
        @dump_timeout -= 1
      end
    end

    def field_emitters_for(squadron)
      emitters = artificial_field_emitters_for(squadron) +
        enemy_cell_emitters_for(squadron) +
        friendly_squadron_emitters_for(squadron)

      emitters.compact
    end

    def enemy_cell_emitters_for(squadron)
      []
    end

    def artificial_field_emitters_for(squadron)
      []
    end

    def friendly_squadron_emitters_for(squadron)
      []
    end

    def relative_enemy_category(my_unit_type, enemy_unit_type)
      ignores = VehicleType.ignores?(my_unit_type, enemy_unit_type)
      strong = VehicleType.strong_against?(my_unit_type, enemy_unit_type)
      weak = VehicleType.weak_against?(my_unit_type, enemy_unit_type)

      return :ignore if ignores
      return :attack if strong
      return :avoid if weak

      :attack
    end

  end
end
