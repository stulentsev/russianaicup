module Schedulable
  def self.included(base)
    base.send(:include, InstanceMethods)
    base.extend(ClassMethods)
  end

  module InstanceMethods
    def run_scheduled_events(tick_index)
      schedule = self.class.instance_variable_get(:@__scheduled_events).to_h
      schedule.each do |period, scheduled_logic|
        if tick_index % period == 0
          instance_exec(tick_index, &scheduled_logic)
        end
      end
    end
  end

  module ClassMethods
    def every(number_of_ticks, &block)
      @__scheduled_events                  ||= {}
      @__scheduled_events[number_of_ticks] = block
    end
  end
end

# auxiliary stuff

module WithTicks
  def ticks
    self
  end

  alias_method :tick, :ticks
end

Integer.send(:include, WithTicks) if defined?(Integer)
Fixnum.send(:include, WithTicks) if defined?(Fixnum)
