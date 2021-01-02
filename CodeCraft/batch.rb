#! /usr/bin/env ruby

require 'bundler'
Bundler.require

num_attempts = (ARGV[0] || 100).to_i
puts "running #{num_attempts} attempts"

local_runner_dir = ENV['LOCAL_RUNNER_DIR'] || '/home/sergio/Downloads/aicup2020-linux'
strategy_dir = ENV['STRATEGY_DIR'] || '/home/sergio/projects/russianaicup/CodeCraft'
config_file = ENV['BATCH_CONFIG_FILE'] || File.join(strategy_dir, 'config.json')

system "cd #{local_runner_dir}; rm results/*"
build_ok = system "cd #{strategy_dir}; cargo build --release"
unless build_ok
  puts "Build failed, not running batch games"
  exit(1)
end

progressbar = ProgressBar.create(
  total: num_attempts,
  title: "Running batch simulation",
  format: '%t: %B [%c/%C]',
)

num_attempts.times do |x|
  Dir.chdir(local_runner_dir) do
    server_cmd = "./aicup2020 --config #{config_file} --batch-mode --save-results results/result#{'%04d' % x}.json 2>/dev/null &"
    system(server_cmd)
    sleep(0.2)
  end

  Dir.chdir(strategy_dir) do
    client_cmd = "./target/release/aicup2020 127.0.0.1 32000"
    system(client_cmd)
  end

  progressbar.increment
end

results_dir = "#{local_runner_dir}/results"
tally = {
  'place 1' => 0,
  'place 2' => 0,
  'place 3' => 0,
  'place 4' => 0,
}
seeds = []

Dir.glob(File.join(results_dir, '*.json')).each do |filename|
  res = JSON.parse(File.read(filename))
  scores = res['results']
  p1_score, *other_scores = scores
  rank = scores.size - scores.sort.index(p1_score)
  tally["place #{rank}"] += 1
  if rank != 1
    seeds << res['seed']
  end
end
puts tally

puts "seeds of failed games:"
puts seeds
