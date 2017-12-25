require './model/player'
require './model/world'

class PlayerContext
  # @return [Player, NilClass]
  attr_reader :player

  # @return [World, NilClass]
  attr_reader :world

  # @param [Player, NilClass] player
  # @param [World, NilClass] world
  def initialize(player, world)
    @player = player
    @world = world
  end
end