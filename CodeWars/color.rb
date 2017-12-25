class Color

  attr_reader :r, :g, :b, :a

  def initialize(r, g, b, a = 0)
    @r = r.to_i
    @g = g.to_i
    @b = b.to_i
    @a = a.to_i
  end

  def self.random
    Color.new(
      rand(255),
      rand(255),
      rand(255),
      rand(255)
    )
  end

  def opacity(new_a)
    Color.new(r, g, b, new_a)
  end

  def saturation(s)
    h, _, v = to_hsv

    Color.new(*Color.from_hsv(h, s, v).to_rgb)
  end

  # as described in https://docs.oracle.com/javase/7/docs/api/java/awt/Color.html#getRGB()
  def to_i
    (a << 24) + (r << 16) + (g << 8) + b
  end

  def to_s
    [r, g, b].map { |c| c.fdiv(255) }.join(' ')
  end

  def to_rgb
    [r, g, b]
  end

  def to_hsv
    r1 = r.fdiv(255)
    g1 = g.fdiv(255)
    b1 = b.fdiv(255)
    min, max   = [r1, g1, b1].minmax
    delta = max - min

    v     = max

    if max != 0.0
      s = delta.fdiv(max)
    else
      s = 0.0
    end

    if delta == 0.0
      h = 0.0
    else
      if r1 == max
        h = (g1 - b1).fdiv(delta)
      elsif g1 == max
        h = 2 + (b1 - r1).fdiv(delta)
      elsif b1 == max
        h = 4 + (r1 - g1).fdiv(delta)
      end

      h *= 60.0

      if h < 0
        h += 360.0
      end
    end

    [h, s, v]
    # [h.fdiv(360), s.fdiv(100), v.fdiv(100)]
  end

  # @param h (Float) hue, from 0 to 360
  # @param s (Float) saturation, from 0 to 1
  # @param v (Float) value, from 0 to 1
  def self.from_hsv(h, s, v)
    c = v * s
    x = c * (1 - (((h / 60) % 2) - 1).abs)
    m = v - c

    r1, g1, b1 = case h
                 when 0...60
                   [c, x, 0]
                 when 60...120
                   [x, c, 0]
                 when 120...180
                   [0, c, x]
                 when 180...240
                   [0, x, c]
                 when 240...300
                   [x, 0, c]
                 else
                   [c, 0, x]
                 end

    Color.new(
      (r1 + m) * 255,
      (g1 + m) * 255,
      (b1 + m) * 255
    )

    # i = (h * 6).floor
    # f = h * 6 - i
    # p = v * (1 - s)
    # q = v * (1 - f * s)
    # t = v * (1 - (1 - f) * s)
    #
    # r, g, b = case i % 6
    #           when 0
    #             [v, t, p]
    #           when 1
    #             [q, v, p]
    #           when 2
    #             [p, v, t]
    #           when 3
    #             [p, q, v]
    #           when 4
    #             [t, p, v]
    #           else
    #             [v, p, q]
    #           end
    #
    # Color.new(r * 255, g * 255, b * 255)
  end

  def self.red
    Color.new(0xff, 0, 0)
  end

  def self.green
    Color.new(0, 0xff, 0)
  end

  def self.blue
    Color.new(0, 0, 0xff)
  end

  def self.yellow
    Color.new(0xff, 0xff, 0)
  end

  def self.white
    Color.new(0xff, 0xff, 0xff)
  end

  def self.black
    Color.new(0, 0, 0)
  end

  def self.orange
    Color.new(242, 162, 0)
  end

  def self.blueish
    Color.new(0, 162, 242)
  end

end
