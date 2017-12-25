require './point'

module StrategyHelpers
  def nuke_incoming?
    return true if $world.opponent_player.next_nuclear_strike_tick_index != -1
    # return true if $world.my_player.next_nuclear_strike_tick_index != -1

    false
  end

  def nuke_coordinates
    Point.new(
      $world.opponent_player.next_nuclear_strike_x,
      $world.opponent_player.next_nuclear_strike_y
    )
  end

  def nuke_ready?
    $world.my_player.remaining_nuclear_strike_cooldown_ticks < 1
  end

  def nuke_almost_ready?(ticks = 50)
    $world.my_player.remaining_nuclear_strike_cooldown_ticks == ticks
  end

  def evading_nuke?
    @evading_nuke
  end

  def start_nuke_evasion
    @evading_nuke = true
  end

  def stop_nuke_evasion
    @evading_nuke = false
  end
end
