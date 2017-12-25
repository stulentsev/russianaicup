class CommandPipeline
  attr_accessor :world

  def select(vehicle_type: nil, left: 0, top: 0, right: world.width, bottom: world.height, &block)
    push(
      action:       ActionType::CLEAR_AND_SELECT,
      vehicle_type: vehicle_type,
      left:         left,
      top:          top,
      right:        right,
      bottom:       bottom,
      &block
    )
  end

  def add_to_selection(vehicle_type: nil, left: 0, top: 0, right: world.width, bottom: world.height, &block)
    push(
      action:       ActionType::ADD_TO_SELECTION,
      vehicle_type: vehicle_type,
      left:         left,
      top:          top,
      right:        right,
      bottom:       bottom,
      &block
    )
  end

  def move(x: 0, y: 0, max_speed: nil, &block)
    push(action: ActionType::MOVE, x: x, y: y, max_speed: max_speed, &block)
  end

  def move_by(x: nil, y: nil, point: nil, max_speed: nil, &block)
    if point
      move(x: point.x, y: point.y, max_speed: max_speed, &block)
    else
      move(x: x, y: y, max_speed: max_speed, &block)
    end
  end

  def setup_vehicle_production(facility_id:, vehicle_type:)
    push(
      action:       ActionType::SETUP_VEHICLE_PRODUCTION,
      facility_id:  facility_id,
      vehicle_type: vehicle_type
    )
  end

  def stop_movement
    move(x: 0, y: 0)
  end

  def assign(group:, &block)
    push(action: ActionType::ASSIGN, group: group, &block)
  end

  def dismiss(group:, &block)
    push(action: ActionType::DISMISS, group: group, &block)
  end

  def disband(group:, &block)
    push(action: ActionType::DISBAND, group: group, &block)
  end

  def select_group(group:, &block)
    push(action: ActionType::CLEAR_AND_SELECT, group: group, &block)
  end

  def compact(x:, y:, &block)
    scale(x: x, y: y, factor: 0.4, &block)
  end

  def scale(x:, y:, factor:, max_speed: nil, &block)
    push(action: ActionType::SCALE, factor: factor, x: x, y: y, max_speed: max_speed, &block)
  end

  def rotate(x:, y:, angle_degrees:, max_speed: nil, &block)
    angle_rad = angle_degrees.fdiv(180) * Math::PI
    push(action: ActionType::ROTATE, x: x, y: y, angle: angle_rad, max_speed: max_speed, &block)
  end

  def tactical_nuke(x:, y:, vehicle_id:)
    push(
      action:     ActionType::TACTICAL_NUCLEAR_STRIKE,
      x:          x,
      y:          y,
      vehicle_id: vehicle_id,
      )
  end

  def priority(&block)
    buffer       = CommandPipeline.new
    buffer.world = world
    block.call(buffer)

    buffer.send(:commands).reverse_each { |cmd, block| unshift(cmd, &block) }
  end

  def unshift(cmd, &block)
    commands.unshift([without_nil_values(cmd), block])
  end

  def push(cmd, &block)
    commands.push([without_nil_values(cmd), block])
  end

  def pop
    commands.shift
  end

  def has_commands?
    !commands.empty?
  end

  def run(move)
    cmd, callback = pop

    return unless cmd
    puts "tick \##{world.tick_index}: executing #{cmd} (cooldown: #{$world.my_player.remaining_action_cooldown_ticks})"

    cmd.each do |field, value|
      move.public_send("#{field}=", value)
    end
    mark_execution_in_command_history

    callback.call if callback
  end

  def soft_limit_reached?
    return false if command_history.empty?
    return false if $world.tick_index < 300

    command_history.first > world.tick_index - 60 && command_history.size == soft_limit
  end

  def mark_execution_in_command_history
    command_history.push(world.tick_index)
    command_history.shift if command_history.size > soft_limit
  end

  def soft_limit
    nuke_evasion_sequence_size = 3
    limit_per_60_ticks         = 12 + $world.facilities.control_centers.mine.count
    limit_per_60_ticks - nuke_evasion_sequence_size
  end

  def command_history
    @command_history ||= []
  end

  def without_nil_values(cmd)
    cmd.delete_if { |_k, v| v.nil? }
  end

  def commands
    @commands ||= []
  end
end
