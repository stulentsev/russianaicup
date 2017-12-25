module NukeEvasion
  GROUP_NUKE_TARGET         = 99
  NUKE_EVASION_SCALE_FACTOR = 2.0

  def check_for_nuke
    if nuke_incoming?
      emergency_nuke_evasion_maneuver! unless evading_nuke?
    else
      if evading_nuke?
        restore_formation_after_evasive_maneuver
        stop_nuke_evasion
      end
    end
  end

  def emergency_nuke_evasion_maneuver!
    start_nuke_evasion
    pipeline.priority do |priority_pipeline|
      nx = world.opponent_player.next_nuclear_strike_x
      ny = world.opponent_player.next_nuclear_strike_y

      scale_out_from_the_nuke(nx, ny, priority_pipeline)
      @count_down           = $game.tactical_nuclear_strike_delay + 10
      @current_center_point = Point.new(nx, ny)
    end
  end

  def scale_out_from_the_nuke(nx, ny, priority_pipeline)
    nuke_radius      = $game.tactical_nuclear_strike_radius
    selection_radius = nuke_radius * 1.1

    priority_pipeline.select(left:  nx - selection_radius, top: ny - selection_radius,
                             right: nx + selection_radius, bottom: ny + selection_radius)
    priority_pipeline.assign(group: GROUP_NUKE_TARGET)
    priority_pipeline.scale(x: nx, y: ny, factor: NUKE_EVASION_SCALE_FACTOR)
  end

  def restore_formation_after_evasive_maneuver
    pipeline.priority do |priority_pipeline|
      priority_pipeline.select_group(group: GROUP_NUKE_TARGET)
      priority_pipeline.scale(x: @current_center_point.x, y: @current_center_point.y, factor: 1.0 / NUKE_EVASION_SCALE_FACTOR)
      priority_pipeline.disband(group: GROUP_NUKE_TARGET)
      @current_center_point = nil
      @count_down           = 30
    end
  end

end
