require './brain_potential'
require './gnuplot_dumper'

module Brains
  class Stalker < Potential
    def initialize_brain
      @didnt_move_turns = 0
    end

    private

    def field_emitters_for(squadron)
      emitters = artificial_field_emitters_for(squadron) +
        enemy_cell_emitters_for(squadron) +
        friendly_squadron_emitters_for(squadron)

      emitters.compact
    end

    def enemy_cell_emitters_for(squadron)
      [].tap do |emitters|
        clusters.map do |cluster|
          # TODO: detect type more reliably
          sample_unit = cluster.units.detect(&:alive?)
          next unless sample_unit

          enemy_unit_type = sample_unit.type
          category = relative_enemy_category(squadron.unit_type, enemy_unit_type)
          target_location = cluster.center_point

          case category
          when :attack
            # long-range attractor
            emitters.push(
              ExponentialFalloffEmitter.new(
                location:         target_location,
                max_effect_value: 1000,
                exponent:         50,
                effect_radius:    1024
              )
            )
          when :avoid
            emitters.push(
              SafeDistanceEmitter.new(
                location:                      target_location,
                max_effect_value:              BASE_ENEMY_CELL_ATTRACTION * cluster.size,
                effect_radius:                 ENEMY_CELL_EFFECT_RADIUS,
                preferable_distance:           PREFERABLE_DISTANCE_FROM_ENEMY_CELL + 50,
                preferable_distance_variation: PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION
              )
            )
          else
            # ignore, no emitter
          end
        end
      end
    end

    # def enemy_cell_emitters_for(squadron)
    #   [].tap do |emitters|
    #     enemies_by_cell.each do |(x, y), enemy_count|
    #       enemies = enemies_at_cell(x, y)
    #       next if enemies.empty?
    #
    #       if squadron.easy_prey?(enemies)
    #         # puts "Detected easy prey(type=#{enemies.count_by(&:type)} for #{squadron.unit_type}"
    #         emitters.push(
    #           ExponentialFalloffEmitter.new(
    #             location:         Point.from_cell(x, y),
    #             max_effect_value: 1000,
    #             effect_radius:    250,
    #             exponent:         50
    #           )
    #         )
    #       else # keep distance
    #         # puts "Cell with #{enemy_count} units, position: #{Point.from_cell(x, y)}, projected position: #{Point.from_cell(x, y) + aggregate_direction}"
    #         emitters.push(
    #           SafeDistanceEmitter.new(
    #             location:                      Point.from_cell(x, y),
    #             max_effect_value:              BASE_ENEMY_CELL_ATTRACTION * enemy_count,
    #             effect_radius:                 ENEMY_CELL_EFFECT_RADIUS,
    #             preferable_distance:           PREFERABLE_DISTANCE_FROM_ENEMY_CELL,
    #             preferable_distance_variation: PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION
    #           )
    #         )
    #       end
    #     end
    #   end
    # end

    def artificial_field_emitters_for(squadron)
      [].tap do |emitters|
        if should_retreat_and_heal?
          healer_squadron = friendly_squadrons.detect{|s| s.alive? && s.arrv?}

          if healer_squadron.nil?
            @healer_squadron_dead = true
            squadron.healing = false
          else
            squadron.healing = true
            emitters.push(LinearFalloffEmitter.new(
              location:         healer_squadron.location,
              effect_radius:    world.width,
              max_effect_value: 15000.0
            ))
          end

        end

        if squadron.ground?
          emitters.push(*facility_emitters_for(squadron))
        end

      end
    end

    def facility_emitters_for(squadron)
      emitters = []

      $world.facilities.not_mine.each do |facility|
        emitters.push(
          ExponentialFalloffEmitter.new(
            location:         facility.location,
            max_effect_value: 3000,
            exponent:         50,
            effect_radius:    1500
          )
        )
      end
      $world.facilities.factories.mine.each do |facility|
        emitters.push(
          ExponentialFalloffEmitter.new(
            location:         facility.location,
            max_effect_value: -3000,
            exponent:         50,
            effect_radius:    40
          )
        )
      end

      emitters
    end

    def friendly_squadron_emitters_for(squadron)
      factor = -3500
      exponent = 50
      effect_radius = 100
      without_self       = friendly_squadrons.reject { |s| s.id == squadron.id }
      mutually_repelling = without_self.select { |s| s.keep_distance_from?(squadron) }
      mutually_repelling.flat_map do |squadron|
        [
          ExponentialFalloffEmitter.new(
            location:         squadron.location,
            max_effect_value: factor,
            exponent:         exponent,
            effect_radius:    effect_radius
          ),

        # ExponentialFalloffEmitter.new(
        #     max_effect_value: -500,
        #     exponent:         25,
        #     bias:             0,
        #     effect_radius:    200,
        #     location:         squadron.location
        #   ),
        ].tap do |result|
          if squadron.destination
            result.push(
              ExponentialFalloffEmitter.new(
                max_effect_value: factor,
                exponent:         exponent,
                bias:             0,
                effect_radius:    effect_radius,
                location:         squadron.location
              )
            )
          end
        end
      end
    end

    private

    def should_retreat_and_heal?
      return false if @healer_squadron_dead
      return true if squadron.healing

      squadron.aerial? &&
        squadron.low_health? &&
        friendly_squadrons.any? { |s| s.alive? && s.arrv? && s.size > 10 }
    end

  end
end
