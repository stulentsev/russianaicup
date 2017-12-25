module AppSettings
  def self.rewind
    env_var('REWIND')
  end

  def self.jam
    env_var('JAM')
  end

  def self.localhost
    env_var('SOVIET_RUSSIA')
  end

  private

  def self.env_var(var)
    ENV[var].to_i == 1
  end
end
