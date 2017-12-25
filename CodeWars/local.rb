require './my_strategy'
require './remote_process_client'

class Runner
  def initialize
    @remote_process_client = RemoteProcessClient::new('127.0.0.1', 31002)
    @token = '0000000000000000'
  end

  def run
    begin
      @remote_process_client.write_token_message(@token)
      @remote_process_client.write_protocol_version_message
      @remote_process_client.read_team_size_message
      game = @remote_process_client.read_game_context_message

      strategy = MyStrategy::new

      until (player_context = @remote_process_client.read_player_context_message).nil?
        player = player_context.player
        break if player.nil?

        move = Move::new
        strategy.move(player, player_context.world, game, move)

        @remote_process_client.write_move_message(move)
      end
    ensure
      @remote_process_client.close
    end
  end
end

Runner.new.run
