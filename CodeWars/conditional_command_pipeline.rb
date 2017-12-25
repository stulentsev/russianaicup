class ConditionalCommandPipeline
  attr_reader :board

  def initialize(board:)
    @board = board
  end

  def handle_tick
    delayed_handlers.each do |handler|
      handler.can_signal? and handler.signal
    end

    delayed_handlers.reject!(&:signaled?)
  end

  # evaluate custom block/proc
  def when_proc(logic, &block)
    delayed_handlers.push(
      CustomProc.new(logic, &block)
    )
  end

  def create_new
    self.class.new(board: board)
  end

  def when_all(waiters, &block)
    delayed_handlers.push(
      WaitForMultiple.new(waiters: waiters, &block)
    )
  end

  def after(n_ticks, &block)
    delayed_handlers.push(
      Timer.new(
        board: board,
        after: n_ticks,
        &block
      )
    )
  end

  def when_stop_moving(vehicles, after: 5, &block)
    delayed_handlers.push(
      WaitForStandStill.new(
        board:    board,
        vehicles: vehicles,
        after:    after,
        &block
      )
    )
  end

  def delayed_handlers
    @delayed_handlers ||= []
  end

  private

  class BaseHandler
    attr_reader :callback

    def initialize(&block)
      @callback = block
    end

    def can_signal?
      !signaled? && condition_matches?
    end

    def signal
      callback.call
    ensure
      # `say signal`
      @signaled = true
    end

    def signaled?
      @signaled
    end

    def condition_matches?
      fail NotImplementedError
    end
  end

  class WaitForMultiple < BaseHandler
    attr_reader :waiters

    def initialize(waiters:, &block)
      super(&block)
      @waiters = waiters
    end

    def condition_matches?
      waiters.all?(&:condition_matches?)
    end
  end

  class CustomProc < BaseHandler
    attr_reader :logic

    def initialize(logic, &block)
      super(&block)
      @logic = logic
    end

    def condition_matches?
      logic.call
    end
  end

  class Timer < BaseHandler
    attr_reader :board, :after

    def initialize(board:, after: nil, &block)
      super(&block)

      @board              = board
      @after              = after
      @current_tick_index = $world.tick_index
    end

    def condition_matches?
      return true if after.nil?
      # puts "timer set at #{@current_tick_index}, wait for #{after}, current world tick: #{$world.tick_index}"
      @current_tick_index + after < $world.tick_index
    end
  end

  class WaitForStandStill < Timer
    attr_reader :vehicles

    def initialize(vehicles:, **kwargs, &block)
      super(**kwargs, &block)
      @vehicles = vehicles.to_a
      vehicles.each(&:increment_movement_watcher_count)
    end

    def signal
      super
      vehicles.each(&:decrement_movement_watcher_count)
    end

    def condition_matches?
      return false unless super
      # puts '.'
      vehicles.all? do |v|
        v.last_moved_at.nil? || v.last_moved_at < $world.tick_index - 1
      end
    end
  end
end
