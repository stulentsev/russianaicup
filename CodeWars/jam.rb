require 'socket'

class Jam
  attr_reader :socket

  def initialize
    @socket = TCPSocket.new('localhost', 13579)
  end

  def begin_post
    socket.puts 'begin post'
  end

  def end_post
    socket.puts 'end post'
  end

  def circle(x, y, r, color)
    socket.puts("circle #{x} #{y} #{r} #{color}")
  end

  def fill_circle(x, y, r, color)
    socket.puts("fill_circle #{x} #{y} #{r} #{color}")
  end

  def rect(x0, y0, x1, y1, color)
    socket.puts("rect #{x0} #{y0} #{x1} #{y1} #{color}")
  end

  def fill_rect(x0, y0, x1, y1, color)
    socket.puts("fill_rect #{x0} #{y0} #{x1} #{y1} #{color}")
  end

  def line(x0, y0, x1, y1, color)
    socket.puts("line #{x0} #{y0} #{x1} #{y1} #{color}")
  end

  def text(x0, y0, msg, color)
    socket.puts("text #{x0} #{y0} #{msg} #{color}")
  end

  def arc(x, y, r, start_angle_rad, arc_angle_rad, color)
    socket.puts("arc #{x} #{y} #{r} #{start_angle_rad} #{arc_angle_rad} #{color}")
  end

  def fill_arc(x, y, r, start_angle_rad, arc_angle_rad, color)
    socket.puts("fill_arc #{x} #{y} #{r} #{start_angle_rad} #{arc_angle_rad} #{color}")
  end


end
