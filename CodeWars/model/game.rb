# noinspection RubyInstanceVariableNamingConvention,RubyParameterNamingConvention,RubyTooManyInstanceVariablesInspection
class Game
  # @return [Integer]
  attr_reader :random_seed

  # @return [Integer]
  attr_reader :tick_count

  # @return [Float]
  attr_reader :world_width

  # @return [Float]
  attr_reader :world_height

  # @return [TrueClass, FalseClass]
  attr_reader :fog_of_war_enabled

  # @return [Integer]
  attr_reader :victory_score

  # @return [Integer]
  attr_reader :facility_capture_score

  # @return [Integer]
  attr_reader :vehicle_elimination_score

  # @return [Integer]
  attr_reader :action_detection_interval

  # @return [Integer]
  attr_reader :base_action_count

  # @return [Integer]
  attr_reader :additional_action_count_per_control_center

  # @return [Integer]
  attr_reader :max_unit_group

  # @return [Integer]
  attr_reader :terrain_weather_map_column_count

  # @return [Integer]
  attr_reader :terrain_weather_map_row_count

  # @return [Float]
  attr_reader :plain_terrain_vision_factor

  # @return [Float]
  attr_reader :plain_terrain_stealth_factor

  # @return [Float]
  attr_reader :plain_terrain_speed_factor

  # @return [Float]
  attr_reader :swamp_terrain_vision_factor

  # @return [Float]
  attr_reader :swamp_terrain_stealth_factor

  # @return [Float]
  attr_reader :swamp_terrain_speed_factor

  # @return [Float]
  attr_reader :forest_terrain_vision_factor

  # @return [Float]
  attr_reader :forest_terrain_stealth_factor

  # @return [Float]
  attr_reader :forest_terrain_speed_factor

  # @return [Float]
  attr_reader :clear_weather_vision_factor

  # @return [Float]
  attr_reader :clear_weather_stealth_factor

  # @return [Float]
  attr_reader :clear_weather_speed_factor

  # @return [Float]
  attr_reader :cloud_weather_vision_factor

  # @return [Float]
  attr_reader :cloud_weather_stealth_factor

  # @return [Float]
  attr_reader :cloud_weather_speed_factor

  # @return [Float]
  attr_reader :rain_weather_vision_factor

  # @return [Float]
  attr_reader :rain_weather_stealth_factor

  # @return [Float]
  attr_reader :rain_weather_speed_factor

  # @return [Float]
  attr_reader :vehicle_radius

  # @return [Integer]
  attr_reader :tank_durability

  # @return [Float]
  attr_reader :tank_speed

  # @return [Float]
  attr_reader :tank_vision_range

  # @return [Float]
  attr_reader :tank_ground_attack_range

  # @return [Float]
  attr_reader :tank_aerial_attack_range

  # @return [Integer]
  attr_reader :tank_ground_damage

  # @return [Integer]
  attr_reader :tank_aerial_damage

  # @return [Integer]
  attr_reader :tank_ground_defence

  # @return [Integer]
  attr_reader :tank_aerial_defence

  # @return [Integer]
  attr_reader :tank_attack_cooldown_ticks

  # @return [Integer]
  attr_reader :tank_production_cost

  # @return [Integer]
  attr_reader :ifv_durability

  # @return [Float]
  attr_reader :ifv_speed

  # @return [Float]
  attr_reader :ifv_vision_range

  # @return [Float]
  attr_reader :ifv_ground_attack_range

  # @return [Float]
  attr_reader :ifv_aerial_attack_range

  # @return [Integer]
  attr_reader :ifv_ground_damage

  # @return [Integer]
  attr_reader :ifv_aerial_damage

  # @return [Integer]
  attr_reader :ifv_ground_defence

  # @return [Integer]
  attr_reader :ifv_aerial_defence

  # @return [Integer]
  attr_reader :ifv_attack_cooldown_ticks

  # @return [Integer]
  attr_reader :ifv_production_cost

  # @return [Integer]
  attr_reader :arrv_durability

  # @return [Float]
  attr_reader :arrv_speed

  # @return [Float]
  attr_reader :arrv_vision_range

  # @return [Integer]
  attr_reader :arrv_ground_defence

  # @return [Integer]
  attr_reader :arrv_aerial_defence

  # @return [Integer]
  attr_reader :arrv_production_cost

  # @return [Float]
  attr_reader :arrv_repair_range

  # @return [Float]
  attr_reader :arrv_repair_speed

  # @return [Integer]
  attr_reader :helicopter_durability

  # @return [Float]
  attr_reader :helicopter_speed

  # @return [Float]
  attr_reader :helicopter_vision_range

  # @return [Float]
  attr_reader :helicopter_ground_attack_range

  # @return [Float]
  attr_reader :helicopter_aerial_attack_range

  # @return [Integer]
  attr_reader :helicopter_ground_damage

  # @return [Integer]
  attr_reader :helicopter_aerial_damage

  # @return [Integer]
  attr_reader :helicopter_ground_defence

  # @return [Integer]
  attr_reader :helicopter_aerial_defence

  # @return [Integer]
  attr_reader :helicopter_attack_cooldown_ticks

  # @return [Integer]
  attr_reader :helicopter_production_cost

  # @return [Integer]
  attr_reader :fighter_durability

  # @return [Float]
  attr_reader :fighter_speed

  # @return [Float]
  attr_reader :fighter_vision_range

  # @return [Float]
  attr_reader :fighter_ground_attack_range

  # @return [Float]
  attr_reader :fighter_aerial_attack_range

  # @return [Integer]
  attr_reader :fighter_ground_damage

  # @return [Integer]
  attr_reader :fighter_aerial_damage

  # @return [Integer]
  attr_reader :fighter_ground_defence

  # @return [Integer]
  attr_reader :fighter_aerial_defence

  # @return [Integer]
  attr_reader :fighter_attack_cooldown_ticks

  # @return [Integer]
  attr_reader :fighter_production_cost

  # @return [Float]
  attr_reader :max_facility_capture_points

  # @return [Float]
  attr_reader :facility_capture_points_per_vehicle_per_tick

  # @return [Float]
  attr_reader :facility_width

  # @return [Float]
  attr_reader :facility_height

  # @return [Integer]
  attr_reader :base_tactical_nuclear_strike_cooldown

  # @return [Integer]
  attr_reader :tactical_nuclear_strike_cooldown_decrease_per_control_center

  # @return [Float]
  attr_reader :max_tactical_nuclear_strike_damage

  # @return [Float]
  attr_reader :tactical_nuclear_strike_radius

  # @return [Integer]
  attr_reader :tactical_nuclear_strike_delay

  # @param [Integer] random_seed
  # @param [Integer] tick_count
  # @param [Float] world_width
  # @param [Float] world_height
  # @param [TrueClass, FalseClass] fog_of_war_enabled
  # @param [Integer] victory_score
  # @param [Integer] facility_capture_score
  # @param [Integer] vehicle_elimination_score
  # @param [Integer] action_detection_interval
  # @param [Integer] base_action_count
  # @param [Integer] additional_action_count_per_control_center
  # @param [Integer] max_unit_group
  # @param [Integer] terrain_weather_map_column_count
  # @param [Integer] terrain_weather_map_row_count
  # @param [Float] plain_terrain_vision_factor
  # @param [Float] plain_terrain_stealth_factor
  # @param [Float] plain_terrain_speed_factor
  # @param [Float] swamp_terrain_vision_factor
  # @param [Float] swamp_terrain_stealth_factor
  # @param [Float] swamp_terrain_speed_factor
  # @param [Float] forest_terrain_vision_factor
  # @param [Float] forest_terrain_stealth_factor
  # @param [Float] forest_terrain_speed_factor
  # @param [Float] clear_weather_vision_factor
  # @param [Float] clear_weather_stealth_factor
  # @param [Float] clear_weather_speed_factor
  # @param [Float] cloud_weather_vision_factor
  # @param [Float] cloud_weather_stealth_factor
  # @param [Float] cloud_weather_speed_factor
  # @param [Float] rain_weather_vision_factor
  # @param [Float] rain_weather_stealth_factor
  # @param [Float] rain_weather_speed_factor
  # @param [Float] vehicle_radius
  # @param [Integer] tank_durability
  # @param [Float] tank_speed
  # @param [Float] tank_vision_range
  # @param [Float] tank_ground_attack_range
  # @param [Float] tank_aerial_attack_range
  # @param [Integer] tank_ground_damage
  # @param [Integer] tank_aerial_damage
  # @param [Integer] tank_ground_defence
  # @param [Integer] tank_aerial_defence
  # @param [Integer] tank_attack_cooldown_ticks
  # @param [Integer] tank_production_cost
  # @param [Integer] ifv_durability
  # @param [Float] ifv_speed
  # @param [Float] ifv_vision_range
  # @param [Float] ifv_ground_attack_range
  # @param [Float] ifv_aerial_attack_range
  # @param [Integer] ifv_ground_damage
  # @param [Integer] ifv_aerial_damage
  # @param [Integer] ifv_ground_defence
  # @param [Integer] ifv_aerial_defence
  # @param [Integer] ifv_attack_cooldown_ticks
  # @param [Integer] ifv_production_cost
  # @param [Integer] arrv_durability
  # @param [Float] arrv_speed
  # @param [Float] arrv_vision_range
  # @param [Integer] arrv_ground_defence
  # @param [Integer] arrv_aerial_defence
  # @param [Integer] arrv_production_cost
  # @param [Float] arrv_repair_range
  # @param [Float] arrv_repair_speed
  # @param [Integer] helicopter_durability
  # @param [Float] helicopter_speed
  # @param [Float] helicopter_vision_range
  # @param [Float] helicopter_ground_attack_range
  # @param [Float] helicopter_aerial_attack_range
  # @param [Integer] helicopter_ground_damage
  # @param [Integer] helicopter_aerial_damage
  # @param [Integer] helicopter_ground_defence
  # @param [Integer] helicopter_aerial_defence
  # @param [Integer] helicopter_attack_cooldown_ticks
  # @param [Integer] helicopter_production_cost
  # @param [Integer] fighter_durability
  # @param [Float] fighter_speed
  # @param [Float] fighter_vision_range
  # @param [Float] fighter_ground_attack_range
  # @param [Float] fighter_aerial_attack_range
  # @param [Integer] fighter_ground_damage
  # @param [Integer] fighter_aerial_damage
  # @param [Integer] fighter_ground_defence
  # @param [Integer] fighter_aerial_defence
  # @param [Integer] fighter_attack_cooldown_ticks
  # @param [Integer] fighter_production_cost
  # @param [Float] max_facility_capture_points
  # @param [Float] facility_capture_points_per_vehicle_per_tick
  # @param [Float] facility_width
  # @param [Float] facility_height
  # @param [Integer] base_tactical_nuclear_strike_cooldown
  # @param [Integer] tactical_nuclear_strike_cooldown_decrease_per_control_center
  # @param [Float] max_tactical_nuclear_strike_damage
  # @param [Float] tactical_nuclear_strike_radius
  # @param [Integer] tactical_nuclear_strike_delay
  def initialize(random_seed, tick_count, world_width, world_height, fog_of_war_enabled, victory_score,
                 facility_capture_score, vehicle_elimination_score, action_detection_interval, base_action_count,
                 additional_action_count_per_control_center, max_unit_group, terrain_weather_map_column_count,
                 terrain_weather_map_row_count, plain_terrain_vision_factor, plain_terrain_stealth_factor,
                 plain_terrain_speed_factor, swamp_terrain_vision_factor, swamp_terrain_stealth_factor,
                 swamp_terrain_speed_factor, forest_terrain_vision_factor, forest_terrain_stealth_factor,
                 forest_terrain_speed_factor, clear_weather_vision_factor, clear_weather_stealth_factor,
                 clear_weather_speed_factor, cloud_weather_vision_factor, cloud_weather_stealth_factor,
                 cloud_weather_speed_factor, rain_weather_vision_factor, rain_weather_stealth_factor,
                 rain_weather_speed_factor, vehicle_radius, tank_durability, tank_speed, tank_vision_range,
                 tank_ground_attack_range, tank_aerial_attack_range, tank_ground_damage, tank_aerial_damage,
                 tank_ground_defence, tank_aerial_defence, tank_attack_cooldown_ticks, tank_production_cost,
                 ifv_durability, ifv_speed, ifv_vision_range, ifv_ground_attack_range, ifv_aerial_attack_range,
                 ifv_ground_damage, ifv_aerial_damage, ifv_ground_defence, ifv_aerial_defence,
                 ifv_attack_cooldown_ticks, ifv_production_cost, arrv_durability, arrv_speed, arrv_vision_range,
                 arrv_ground_defence, arrv_aerial_defence, arrv_production_cost, arrv_repair_range, arrv_repair_speed,
                 helicopter_durability, helicopter_speed, helicopter_vision_range, helicopter_ground_attack_range,
                 helicopter_aerial_attack_range, helicopter_ground_damage, helicopter_aerial_damage,
                 helicopter_ground_defence, helicopter_aerial_defence, helicopter_attack_cooldown_ticks,
                 helicopter_production_cost, fighter_durability, fighter_speed, fighter_vision_range,
                 fighter_ground_attack_range, fighter_aerial_attack_range, fighter_ground_damage, fighter_aerial_damage,
                 fighter_ground_defence, fighter_aerial_defence, fighter_attack_cooldown_ticks, fighter_production_cost,
                 max_facility_capture_points, facility_capture_points_per_vehicle_per_tick, facility_width,
                 facility_height, base_tactical_nuclear_strike_cooldown,
                 tactical_nuclear_strike_cooldown_decrease_per_control_center, max_tactical_nuclear_strike_damage,
                 tactical_nuclear_strike_radius, tactical_nuclear_strike_delay)
    @random_seed = random_seed
    @tick_count = tick_count
    @world_width = world_width
    @world_height = world_height
    @fog_of_war_enabled = fog_of_war_enabled
    @victory_score = victory_score
    @facility_capture_score = facility_capture_score
    @vehicle_elimination_score = vehicle_elimination_score
    @action_detection_interval = action_detection_interval
    @base_action_count = base_action_count
    @additional_action_count_per_control_center = additional_action_count_per_control_center
    @max_unit_group = max_unit_group
    @terrain_weather_map_column_count = terrain_weather_map_column_count
    @terrain_weather_map_row_count = terrain_weather_map_row_count
    @plain_terrain_vision_factor = plain_terrain_vision_factor
    @plain_terrain_stealth_factor = plain_terrain_stealth_factor
    @plain_terrain_speed_factor = plain_terrain_speed_factor
    @swamp_terrain_vision_factor = swamp_terrain_vision_factor
    @swamp_terrain_stealth_factor = swamp_terrain_stealth_factor
    @swamp_terrain_speed_factor = swamp_terrain_speed_factor
    @forest_terrain_vision_factor = forest_terrain_vision_factor
    @forest_terrain_stealth_factor = forest_terrain_stealth_factor
    @forest_terrain_speed_factor = forest_terrain_speed_factor
    @clear_weather_vision_factor = clear_weather_vision_factor
    @clear_weather_stealth_factor = clear_weather_stealth_factor
    @clear_weather_speed_factor = clear_weather_speed_factor
    @cloud_weather_vision_factor = cloud_weather_vision_factor
    @cloud_weather_stealth_factor = cloud_weather_stealth_factor
    @cloud_weather_speed_factor = cloud_weather_speed_factor
    @rain_weather_vision_factor = rain_weather_vision_factor
    @rain_weather_stealth_factor = rain_weather_stealth_factor
    @rain_weather_speed_factor = rain_weather_speed_factor
    @vehicle_radius = vehicle_radius
    @tank_durability = tank_durability
    @tank_speed = tank_speed
    @tank_vision_range = tank_vision_range
    @tank_ground_attack_range = tank_ground_attack_range
    @tank_aerial_attack_range = tank_aerial_attack_range
    @tank_ground_damage = tank_ground_damage
    @tank_aerial_damage = tank_aerial_damage
    @tank_ground_defence = tank_ground_defence
    @tank_aerial_defence = tank_aerial_defence
    @tank_attack_cooldown_ticks = tank_attack_cooldown_ticks
    @tank_production_cost = tank_production_cost
    @ifv_durability = ifv_durability
    @ifv_speed = ifv_speed
    @ifv_vision_range = ifv_vision_range
    @ifv_ground_attack_range = ifv_ground_attack_range
    @ifv_aerial_attack_range = ifv_aerial_attack_range
    @ifv_ground_damage = ifv_ground_damage
    @ifv_aerial_damage = ifv_aerial_damage
    @ifv_ground_defence = ifv_ground_defence
    @ifv_aerial_defence = ifv_aerial_defence
    @ifv_attack_cooldown_ticks = ifv_attack_cooldown_ticks
    @ifv_production_cost = ifv_production_cost
    @arrv_durability = arrv_durability
    @arrv_speed = arrv_speed
    @arrv_vision_range = arrv_vision_range
    @arrv_ground_defence = arrv_ground_defence
    @arrv_aerial_defence = arrv_aerial_defence
    @arrv_production_cost = arrv_production_cost
    @arrv_repair_range = arrv_repair_range
    @arrv_repair_speed = arrv_repair_speed
    @helicopter_durability = helicopter_durability
    @helicopter_speed = helicopter_speed
    @helicopter_vision_range = helicopter_vision_range
    @helicopter_ground_attack_range = helicopter_ground_attack_range
    @helicopter_aerial_attack_range = helicopter_aerial_attack_range
    @helicopter_ground_damage = helicopter_ground_damage
    @helicopter_aerial_damage = helicopter_aerial_damage
    @helicopter_ground_defence = helicopter_ground_defence
    @helicopter_aerial_defence = helicopter_aerial_defence
    @helicopter_attack_cooldown_ticks = helicopter_attack_cooldown_ticks
    @helicopter_production_cost = helicopter_production_cost
    @fighter_durability = fighter_durability
    @fighter_speed = fighter_speed
    @fighter_vision_range = fighter_vision_range
    @fighter_ground_attack_range = fighter_ground_attack_range
    @fighter_aerial_attack_range = fighter_aerial_attack_range
    @fighter_ground_damage = fighter_ground_damage
    @fighter_aerial_damage = fighter_aerial_damage
    @fighter_ground_defence = fighter_ground_defence
    @fighter_aerial_defence = fighter_aerial_defence
    @fighter_attack_cooldown_ticks = fighter_attack_cooldown_ticks
    @fighter_production_cost = fighter_production_cost
    @max_facility_capture_points = max_facility_capture_points
    @facility_capture_points_per_vehicle_per_tick = facility_capture_points_per_vehicle_per_tick
    @facility_width = facility_width
    @facility_height = facility_height
    @base_tactical_nuclear_strike_cooldown = base_tactical_nuclear_strike_cooldown
    @tactical_nuclear_strike_cooldown_decrease_per_control_center = tactical_nuclear_strike_cooldown_decrease_per_control_center
    @max_tactical_nuclear_strike_damage = max_tactical_nuclear_strike_damage
    @tactical_nuclear_strike_radius = tactical_nuclear_strike_radius
    @tactical_nuclear_strike_delay = tactical_nuclear_strike_delay
  end
end