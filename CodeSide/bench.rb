#! /usr/bin/env ruby

require 'ruby-progressbar'
num_attempts = (ARGV[0] || 100).to_i
baseline_binary = ARGV[1]

if num_attempts.nil? || baseline_binary.nil?
  puts "Usage: ./bench.rb <num_attempts> <target_binary>"
  puts "For example:"
  puts "\t./bench.rb 100 aicup2019-8.1"
  exit(1)
end
puts "running #{num_attempts} attempts"

local_runner_dir = ENV['LOCAL_RUNNER_DIR'] || '/Users/sergio/projects/aicup/2019/aicup2019-macos'
strategy_dir = ENV['STRATEGY_DIR'] || '/Users/sergio/projects/aicup/2019/rust'
config_file = 'config-bench.json'


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
    server_cmd = "./aicup2019 --config #{config_file} --batch-mode --save-results results/result#{'%04d' % x}.json 2>/dev/null &"
    system(server_cmd)
    sleep(0.5)
  end

  Dir.chdir(strategy_dir) do
    client_cmd = "./target/release/#{baseline_binary} 127.0.0.1 31001 >/dev/null 2>&1 &"
    system(client_cmd)
    sleep(0.5)

    client_cmd = "./target/release/aicup2019 127.0.0.1 31000 >/dev/null 2>&1"
    system(client_cmd)
  end
  progressbar.increment
end

results_dir = "#{local_runner_dir}/results"
require 'json'
tally = {
  wins: 0,
  losses: 0,
  draws: 0,
}
seeds = []

Dir.glob(File.join(results_dir, '*.json')).each do |filename|
  res = JSON.parse(File.read(filename))
  p1_score, p2_score = res['results']
  if p1_score > p2_score
    tally[:wins] += 1
  elsif p1_score < p2_score
    tally[:losses] += 1
    seeds << res['seed']
  else
    tally[:draws] += 1
  end
end
puts tally

puts "seeds of failed games:"
puts seeds