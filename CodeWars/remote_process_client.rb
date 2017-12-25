require 'socket'
require './model/action_type'
require './model/circular_unit'
require './model/game'
require './model/move'
require './model/player'
require './model/player_context'
require './model/unit'
require './model/world'

# noinspection RubyTooManyMethodsInspection
class RemoteProcessClient
  LITTLE_ENDIAN_BYTE_ORDER = true
  BYTE_ORDER_FORMAT_STRING = LITTLE_ENDIAN_BYTE_ORDER ? '<' : '>'

  BYTE_FORMAT_STRING = 'c'
  INT_FORMAT_STRING = 'l' + BYTE_ORDER_FORMAT_STRING
  LONG_FORMAT_STRING = 'q' + BYTE_ORDER_FORMAT_STRING
  DOUBLE_FORMAT_STRING = LITTLE_ENDIAN_BYTE_ORDER ? 'E' : 'G'

  INTEGER_SIZE_BYTES = 4
  LONG_SIZE_BYTES = 8
  DOUBLE_SIZE_BYTES = 8

  EMPTY_BYTE_ARRAY = ''

  def initialize(host, port)
    @socket = TCPSocket::new(host, port)

    @previous_player_by_id = {}
    @previous_facility_by_id = {}
  end

  def write_token_message(token)
    write_enum(MessageType::AUTHENTICATION_TOKEN)
    write_string(token)
  end

  def write_protocol_version_message
    write_enum(MessageType::PROTOCOL_VERSION)
    write_int(3)
  end

  def read_team_size_message
    ensure_message_type(read_enum(MessageType), MessageType::TEAM_SIZE)
    read_int
  end

  def read_game_context_message
    ensure_message_type(read_enum(MessageType), MessageType::GAME_CONTEXT)
    read_game
  end

  def read_player_context_message
    message_type = read_enum(MessageType)
    if message_type == MessageType::GAME_OVER
      return nil
    end

    ensure_message_type(message_type, MessageType::PLAYER_CONTEXT)
    read_player_context
  end

  def write_move_message(move)
    write_enum(MessageType::MOVE)
    write_move(move)
  end

  def read_facility
    flag = read_signed_byte
    return nil if flag == 0
    return @previous_facility_by_id[read_long] if flag == 127

    facility = Facility::new(read_long, read_enum(FacilityType), read_long, read_double, read_double, read_double,
                             read_enum(VehicleType), read_int)
    @previous_facility_by_id[facility.id] = facility
  end

  def write_facility(facility)
    if facility.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_long(facility.id)
      write_enum(facility.type)
      write_long(facility.owner_player_id)
      write_double(facility.left)
      write_double(facility.top)
      write_double(facility.capture_points)
      write_enum(facility.vehicle_type)
      write_int(facility.production_progress)
    end
  end

  def read_facilities
    facility_count = read_int
    return @previous_facilities if facility_count < 0

    facilities = []
    facility_count.times {|_| facilities.push(read_facility)}
    @previous_facilities = facilities
  end

  def write_facilities(facilities)
    if facilities.nil?
      write_int(-1)
    else
      write_int(facilities.length)
      facilities.each {|facility| write_facility(facility)}
    end
  end

  def read_game
    return nil unless read_boolean

    game = read_bytes(565).unpack('q<l<E2cl9<E19l<E4l7<E4l7<E2l3<E2l<E4l7<E4l6<E4l2<E2l<')

    Game::new(game[0], game[1], game[2], game[3], game[4] != 0, game[5], game[6], game[7], game[8], game[9], game[10],
              game[11], game[12], game[13], game[14], game[15], game[16], game[17], game[18], game[19], game[20],
              game[21], game[22], game[23], game[24], game[25], game[26], game[27], game[28], game[29], game[30],
              game[31], game[32], game[33], game[34], game[35], game[36], game[37], game[38], game[39], game[40],
              game[41], game[42], game[43], game[44], game[45], game[46], game[47], game[48], game[49], game[50],
              game[51], game[52], game[53], game[54], game[55], game[56], game[57], game[58], game[59], game[60],
              game[61], game[62], game[63], game[64], game[65], game[66], game[67], game[68], game[69], game[70],
              game[71], game[72], game[73], game[74], game[75], game[76], game[77], game[78], game[79], game[80],
              game[81], game[82], game[83], game[84], game[85], game[86], game[87], game[88], game[89], game[90],
              game[91], game[92], game[93])
  end

  def write_game(game)
    if game.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_long(game.random_seed)
      write_int(game.tick_count)
      write_double(game.world_width)
      write_double(game.world_height)
      write_boolean(game.fog_of_war_enabled)
      write_int(game.victory_score)
      write_int(game.facility_capture_score)
      write_int(game.vehicle_elimination_score)
      write_int(game.action_detection_interval)
      write_int(game.base_action_count)
      write_int(game.additional_action_count_per_control_center)
      write_int(game.max_unit_group)
      write_int(game.terrain_weather_map_column_count)
      write_int(game.terrain_weather_map_row_count)
      write_double(game.plain_terrain_vision_factor)
      write_double(game.plain_terrain_stealth_factor)
      write_double(game.plain_terrain_speed_factor)
      write_double(game.swamp_terrain_vision_factor)
      write_double(game.swamp_terrain_stealth_factor)
      write_double(game.swamp_terrain_speed_factor)
      write_double(game.forest_terrain_vision_factor)
      write_double(game.forest_terrain_stealth_factor)
      write_double(game.forest_terrain_speed_factor)
      write_double(game.clear_weather_vision_factor)
      write_double(game.clear_weather_stealth_factor)
      write_double(game.clear_weather_speed_factor)
      write_double(game.cloud_weather_vision_factor)
      write_double(game.cloud_weather_stealth_factor)
      write_double(game.cloud_weather_speed_factor)
      write_double(game.rain_weather_vision_factor)
      write_double(game.rain_weather_stealth_factor)
      write_double(game.rain_weather_speed_factor)
      write_double(game.vehicle_radius)
      write_int(game.tank_durability)
      write_double(game.tank_speed)
      write_double(game.tank_vision_range)
      write_double(game.tank_ground_attack_range)
      write_double(game.tank_aerial_attack_range)
      write_int(game.tank_ground_damage)
      write_int(game.tank_aerial_damage)
      write_int(game.tank_ground_defence)
      write_int(game.tank_aerial_defence)
      write_int(game.tank_attack_cooldown_ticks)
      write_int(game.tank_production_cost)
      write_int(game.ifv_durability)
      write_double(game.ifv_speed)
      write_double(game.ifv_vision_range)
      write_double(game.ifv_ground_attack_range)
      write_double(game.ifv_aerial_attack_range)
      write_int(game.ifv_ground_damage)
      write_int(game.ifv_aerial_damage)
      write_int(game.ifv_ground_defence)
      write_int(game.ifv_aerial_defence)
      write_int(game.ifv_attack_cooldown_ticks)
      write_int(game.ifv_production_cost)
      write_int(game.arrv_durability)
      write_double(game.arrv_speed)
      write_double(game.arrv_vision_range)
      write_int(game.arrv_ground_defence)
      write_int(game.arrv_aerial_defence)
      write_int(game.arrv_production_cost)
      write_double(game.arrv_repair_range)
      write_double(game.arrv_repair_speed)
      write_int(game.helicopter_durability)
      write_double(game.helicopter_speed)
      write_double(game.helicopter_vision_range)
      write_double(game.helicopter_ground_attack_range)
      write_double(game.helicopter_aerial_attack_range)
      write_int(game.helicopter_ground_damage)
      write_int(game.helicopter_aerial_damage)
      write_int(game.helicopter_ground_defence)
      write_int(game.helicopter_aerial_defence)
      write_int(game.helicopter_attack_cooldown_ticks)
      write_int(game.helicopter_production_cost)
      write_int(game.fighter_durability)
      write_double(game.fighter_speed)
      write_double(game.fighter_vision_range)
      write_double(game.fighter_ground_attack_range)
      write_double(game.fighter_aerial_attack_range)
      write_int(game.fighter_ground_damage)
      write_int(game.fighter_aerial_damage)
      write_int(game.fighter_ground_defence)
      write_int(game.fighter_aerial_defence)
      write_int(game.fighter_attack_cooldown_ticks)
      write_int(game.fighter_production_cost)
      write_double(game.max_facility_capture_points)
      write_double(game.facility_capture_points_per_vehicle_per_tick)
      write_double(game.facility_width)
      write_double(game.facility_height)
      write_int(game.base_tactical_nuclear_strike_cooldown)
      write_int(game.tactical_nuclear_strike_cooldown_decrease_per_control_center)
      write_double(game.max_tactical_nuclear_strike_damage)
      write_double(game.tactical_nuclear_strike_radius)
      write_int(game.tactical_nuclear_strike_delay)
    end
  end

  def read_games
    game_count = read_int
    return nil if game_count < 0

    games = []
    game_count.times {|_| games.push(read_game)}
    games
  end

  def write_games(games)
    if games.nil?
      write_int(-1)
    else
      write_int(games.length)
      games.each {|game| write_game(game)}
    end
  end

  def write_move(move)
    if move.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_enum(move.action)
      write_int(move.group)
      write_double(move.left)
      write_double(move.top)
      write_double(move.right)
      write_double(move.bottom)
      write_double(move.x)
      write_double(move.y)
      write_double(move.angle)
      write_double(move.factor)
      write_double(move.max_speed)
      write_double(move.max_angular_speed)
      write_enum(move.vehicle_type)
      write_long(move.facility_id)
      write_long(move.vehicle_id)
    end
  end

  def write_moves(moves)
    if moves.nil?
      write_int(-1)
    else
      write_int(moves.length)
      moves.each {|move| write_move(move)}
    end
  end

  def read_player
    flag = read_signed_byte
    return nil if flag == 0
    return @previous_player_by_id[read_long] if flag == 127

    player = read_bytes(50).unpack('q<c2l3<q<l<E2')

    player = Player::new(player[0], player[1] != 0, player[2] != 0, player[3], player[4], player[5], player[6],
                         player[7], player[8], player[9])
    @previous_player_by_id[player.id] = player
  end

  def write_player(player)
    if player.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_long(player.id)
      write_boolean(player.me)
      write_boolean(player.strategy_crashed)
      write_int(player.score)
      write_int(player.remaining_action_cooldown_ticks)
      write_int(player.remaining_nuclear_strike_cooldown_ticks)
      write_long(player.next_nuclear_strike_vehicle_id)
      write_int(player.next_nuclear_strike_tick_index)
      write_double(player.next_nuclear_strike_x)
      write_double(player.next_nuclear_strike_y)
    end
  end

  def read_players
    player_count = read_int
    return @previous_players if player_count < 0

    players = []
    player_count.times {|_| players.push(read_player)}
    @previous_players = players
  end

  def write_players(players)
    if players.nil?
      write_int(-1)
    else
      write_int(players.length)
      players.each {|player| write_player(player)}
    end
  end

  def read_player_context
    return nil unless read_boolean

    PlayerContext::new(read_player, read_world)
  end

  def write_player_context(player_context)
    if player_context.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_player(player_context.player)
      write_world(player_context.world)
    end
  end

  def read_player_contexts
    player_context_count = read_int
    return nil if player_context_count < 0

    player_contexts = []
    player_context_count.times {|_| player_contexts.push(read_player_context)}
    player_contexts
  end

  def write_player_contexts(player_contexts)
    if player_contexts.nil?
      write_int(-1)
    else
      write_int(player_contexts.length)
      player_contexts.each {|player_context| write_player_context(player_context)}
    end
  end

  def read_vehicle
    return nil unless read_boolean

    vehicle = read_bytes(128).unpack('q<E3q<l2<E7l6<')

    Vehicle::new(vehicle[0], vehicle[1], vehicle[2], vehicle[3], vehicle[4], vehicle[5], vehicle[6], vehicle[7],
                 vehicle[8], vehicle[9], vehicle[10], vehicle[11], vehicle[12], vehicle[13], vehicle[14], vehicle[15],
                 vehicle[16], vehicle[17], vehicle[18], vehicle[19], read_enum(VehicleType), read_boolean, read_boolean,
                 read_ints)
  end

  def write_vehicle(vehicle)
    if vehicle.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_long(vehicle.id)
      write_double(vehicle.x)
      write_double(vehicle.y)
      write_double(vehicle.radius)
      write_long(vehicle.player_id)
      write_int(vehicle.durability)
      write_int(vehicle.max_durability)
      write_double(vehicle.max_speed)
      write_double(vehicle.vision_range)
      write_double(vehicle.squared_vision_range)
      write_double(vehicle.ground_attack_range)
      write_double(vehicle.squared_ground_attack_range)
      write_double(vehicle.aerial_attack_range)
      write_double(vehicle.squared_aerial_attack_range)
      write_int(vehicle.ground_damage)
      write_int(vehicle.aerial_damage)
      write_int(vehicle.ground_defence)
      write_int(vehicle.aerial_defence)
      write_int(vehicle.attack_cooldown_ticks)
      write_int(vehicle.remaining_attack_cooldown_ticks)
      write_enum(vehicle.type)
      write_boolean(vehicle.aerial)
      write_boolean(vehicle.selected)
      write_ints(vehicle.groups)
    end
  end

  def read_vehicles
    vehicle_count = read_int
    return nil if vehicle_count < 0

    vehicles = []
    vehicle_count.times {|_| vehicles.push(read_vehicle)}
    vehicles
  end

  def write_vehicles(vehicles)
    if vehicles.nil?
      write_int(-1)
    else
      write_int(vehicles.length)
      vehicles.each {|vehicle| write_vehicle(vehicle)}
    end
  end

  def read_vehicle_update
    return nil unless read_boolean

    vehicle_update = read_bytes(33).unpack('q<E2l2<c')

    VehicleUpdate::new(vehicle_update[0], vehicle_update[1], vehicle_update[2], vehicle_update[3], vehicle_update[4],
                       vehicle_update[5] != 0, read_ints)
  end

  def write_vehicle_update(vehicle_update)
    if vehicle_update.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_long(vehicle_update.id)
      write_double(vehicle_update.x)
      write_double(vehicle_update.y)
      write_int(vehicle_update.durability)
      write_int(vehicle_update.remaining_attack_cooldown_ticks)
      write_boolean(vehicle_update.selected)
      write_ints(vehicle_update.groups)
    end
  end

  def read_vehicle_updates
    vehicle_update_count = read_int
    return nil if vehicle_update_count < 0

    vehicle_updates = []
    vehicle_update_count.times {|_| vehicle_updates.push(read_vehicle_update)}
    vehicle_updates
  end

  def write_vehicle_updates(vehicle_updates)
    if vehicle_updates.nil?
      write_int(-1)
    else
      write_int(vehicle_updates.length)
      vehicle_updates.each {|vehicle_update| write_vehicle_update(vehicle_update)}
    end
  end

  def read_world
    return nil unless read_boolean

    world = read_bytes(24).unpack('l2<E2')

    World::new(world[0], world[1], world[2], world[3], read_players, read_vehicles, read_vehicle_updates,
               read_terrain_by_cell_x_y, read_weather_by_cell_x_y, read_facilities)
  end

  def write_world(world)
    if world.nil?
      write_boolean(false)
    else
      write_boolean(true)

      write_int(world.tick_index)
      write_int(world.tick_count)
      write_double(world.width)
      write_double(world.height)
      write_players(world.players)
      write_vehicles(world.new_vehicles)
      write_vehicle_updates(world.vehicle_updates)
      write_enums_2d(world.terrain_by_cell_x_y)
      write_enums_2d(world.weather_by_cell_x_y)
      write_facilities(world.facilities)
    end
  end

  def read_worlds
    world_count = read_int
    return nil if world_count < 0

    worlds = []
    world_count.times {|_| worlds.push(read_world)}
    worlds
  end

  def write_worlds(worlds)
    if worlds.nil?
      write_int(-1)
    else
      write_int(worlds.length)
      worlds.each {|world| write_world(world)}
    end
  end

  def read_terrain_by_cell_x_y
    @terrain_by_cell_x_y = read_enums_2d(TerrainType) if @terrain_by_cell_x_y.nil?
    @terrain_by_cell_x_y
  end

  def read_weather_by_cell_x_y
    @weather_by_cell_x_y = read_enums_2d(WeatherType) if @weather_by_cell_x_y.nil?
    @weather_by_cell_x_y
  end

  def ensure_message_type(actual_type, expected_type)
    if actual_type != expected_type
      raise ArgumentError, "Received wrong message [actual=#{actual_type}, expected=#{expected_type}]."
    end
  end

  def read_byte_array(nullable)
    count = read_int

    if count <= 0
      return nullable && count < 0 ? null : EMPTY_BYTE_ARRAY
    end

    read_bytes(count)
  end

  def write_byte_array(array)
    if array.nil?
      write_int(-1)
    else
      write_int(array.length)
      write_bytes(array)
    end
  end

  def read_enum(enum_class)
    value = read_bytes(1).unpack(BYTE_FORMAT_STRING)[0]
    value >= 0 && value < enum_class::COUNT ? value : nil
  end

  def read_enums(enum_class)
    count = read_int
    return nil if count < 0

    enums = []
    count.times {|_| enums.push(read_enum(enum_class))}
    enums
  end

  def read_enums_2d(enum_class)
    count = read_int
    return nil if count < 0

    enums_2d = []
    count.times {|_| enums_2d.push(read_enums(enum_class))}
    enums_2d
  end

  def write_enum(value)
    write_bytes([value.nil? ? -1 : value].pack(BYTE_FORMAT_STRING))
  end

  def write_enums(enums)
    if enums.nil?
      write_int(-1)
    else
      write_int(enums.length)
      enums.each {|enum| write_enum(enum)}
    end
  end

  def write_enums_2d(enums_2d)
    if enums_2d.nil?
      write_enum(-1)
    else
      write_enum(enums_2d.length)
      enums_2d.each {|enums| write_enum(enums)}
    end
  end

  def read_string
    length = read_int
    return nil if length == -1

    read_bytes(length)
  end

  def write_string(value)
    if value.nil?
      write_int(-1)
      return
    end

    write_int(value.length)
    write_bytes(value)
  end

  def read_signed_byte
    read_bytes(1).unpack(BYTE_FORMAT_STRING)[0]
  end

  def read_boolean
    read_bytes(1) != "\0"
  end

  def write_boolean(value)
    write_bytes(value ? "\1" : "\0")
  end

  def read_int
    read_bytes(INTEGER_SIZE_BYTES).unpack(INT_FORMAT_STRING)[0]
  end

  def read_ints
    count = read_int
    return nil if count < 0

    ints = []
    ints.concat(read_bytes(count * INTEGER_SIZE_BYTES).unpack("l#{count}" + BYTE_ORDER_FORMAT_STRING)) if count > 0
    ints
  end

  def read_ints_2d
    count = read_int
    return nil if count < 0

    ints_2d = []
    count.times {|_| ints_2d.push(read_ints)}
    ints_2d
  end

  def write_int(value)
    write_bytes([value].pack(INT_FORMAT_STRING))
  end

  def write_ints(ints)
    if ints.nil?
      write_int(-1)
    else
      write_int(ints.length)
      ints.each {|int| write_int(int)}
    end
  end

  def write_ints_2d(ints_2d)
    if ints_2d.nil?
      write_int(-1)
    else
      write_int(ints_2d.length)
      ints_2d.each {|ints| write_int(ints)}
    end
  end

  def read_long
    read_bytes(LONG_SIZE_BYTES).unpack(LONG_FORMAT_STRING)[0]
  end

  def write_long(value)
    write_bytes([value].pack(LONG_FORMAT_STRING))
  end

  def read_double
    read_bytes(DOUBLE_SIZE_BYTES).unpack(DOUBLE_FORMAT_STRING)[0]
  end

  def write_double(value)
    write_bytes([value].pack(DOUBLE_FORMAT_STRING))
  end

  def read_bytes(byte_count)
    byte_array = ''

    while byte_array.length < byte_count
      chunk = @socket.recv(byte_count - byte_array.length)
      raise IOError, "Can't read #{byte_count} bytes from input stream." if chunk.length == 0
      byte_array << chunk
    end

    byte_array
  end

  def write_bytes(byte_array)
    @socket.write(byte_array)
  end

  def close
    @socket.close
  end

  module MessageType
    UNKNOWN = 0
    GAME_OVER = 1
    AUTHENTICATION_TOKEN = 2
    TEAM_SIZE = 3
    PROTOCOL_VERSION = 4
    GAME_CONTEXT = 5
    PLAYER_CONTEXT = 6
    MOVE = 7
    COUNT = 8
  end
end