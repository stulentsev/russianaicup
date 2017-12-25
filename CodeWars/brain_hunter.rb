require './point'
require './brain_potential'

module Brains
  class Hunter < Potential

    def enemy_cell_emitters_for(squadron)
      [].tap do |emitters|
        clusters.map do |cluster|
          if cluster.sandwich?
            enemy_emitters_for_sandwich_clusters(cluster, emitters)
          else
            enemy_emitters_for_mono_clusters(cluster, emitters)
          end
        end
      end
    end

    def enemy_emitters_for_sandwich_clusters(cluster, emitters)
      target_location = cluster.location

      if should_retreat_and_heal?
        emitters.push(
          ExponentialFalloffEmitter.new(
            location:         target_location,
            max_effect_value: -3000,
            exponent:         15,
            effect_radius:    1024
          )
        )
      else
        emitters.push(
          SafeDistanceEmitter.new(
            location:                      target_location,
            max_effect_value:              BASE_ENEMY_CELL_ATTRACTION * cluster.size,
            effect_radius:                 900.meters,
            preferable_distance:           130.meters,
            preferable_distance_variation: PREFERABLE_DISTANCE_FROM_ENEMY_CELL_VARIATION
          )
        )
      end

    end

    def enemy_emitters_for_mono_clusters(cluster, emitters)
      # detect type more reliably
      sample_unit = cluster.units.detect(&:alive?)
      return unless sample_unit

      enemy_unit_type = sample_unit.type
      category        = relative_enemy_category(squadron.unit_type, enemy_unit_type)
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
        # strong short-range repeller
        emitters.push(
          ExponentialFalloffEmitter.new(
            location:         target_location,
            max_effect_value: -3000,
            exponent:         15,
            effect_radius:    1024
          )
        )
      else
        # ignore, no emitter
      end

    end

    def base_potential_for_cell(cx, cy)
      result = 0
      result += strategy.base_potential_map[cx][cy] # base map value

      VehicleType.all.each do |enemy_unit_type|
        enemy_count = enemies_by_type_by_cell[enemy_unit_type][[cx, cy]]
        next unless enemy_count.to_i > 0
        category = relative_enemy_category(squadron.unit_type, enemy_unit_type)

        result += case category
                  when :attack
                    BASE_ENEMY_CELL_ATTRACTION * enemy_count
                  when :avoid
                    -1 * BASE_ENEMY_CELL_ATTRACTION * enemy_count
                  else
                    0
                  end
      end

      result
    end

    def friendly_squadron_emitters_for(squadron)
      [].tap do |emitters|

        without_self       = friendly_squadrons.reject { |s| s.id == squadron.id }
        mutually_repelling = without_self.select { |s| s.keep_distance_from?(squadron) }
        mutually_repelling.each do |squadron|
          emitters.push(
            ExponentialFalloffEmitter.new(
              max_effect_value: -500,
              exponent:         25,
              bias:             0,
              effect_radius:    200,
              location:         squadron.location
            )
          )
        end
      end
    end

    def artificial_field_emitters_for(squadron)
      [].tap do |emitters|
        if should_retreat_and_heal?
          healer_squadron = friendly_squadrons.detect { |s| s.alive? && s.arrv? }

          if healer_squadron.nil?
            @healer_squadron_dead = true
            squadron.healing      = false
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


    def speed
      squadron.speed
    end

    def squadron_move_radius
      1.0.cells
    end

    def time_until_next_move_calculation
      50.ticks
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
