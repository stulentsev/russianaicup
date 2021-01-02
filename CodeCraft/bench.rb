#! /usr/bin/env ruby

require 'bundler'
Bundler.require
require 'erb'

stage, num_attempts, *baseline_versions = ARGV
num_attempts = num_attempts.to_i

if stage.nil? || num_attempts.nil? || baseline_versions.nil? || baseline_versions.empty?
  puts "Usage: ./bench.rb <stage> <num_attempts> <target_version> <target_version> <target_version>"
  puts "For example:"
  puts "\t./bench.rb Round1 100 v2 v2 v1"
  exit(1)
end
puts "running #{num_attempts} attempts"

local_runner_dir = ENV['LOCAL_RUNNER_DIR'] || '/Users/sergio/projects/aicup/2020/aicup2020-macos'
strategy_dir = ENV['STRATEGY_DIR'] || '/Users/sergio/projects/aicup/2019/rust'


system "cd #{local_runner_dir}; rm results/*"
build_ok = system "cd #{strategy_dir}; cargo build --release"
unless build_ok
  puts "Build failed, not running batch games"
  exit(1)
end

system `cp target/release/aicup2020 target/release/aicup2020-current-for-bench`

progressbar = ProgressBar.create(
  total: num_attempts,
  title: "Running batch simulation",
  format: '%t: %B [%c/%C]',
)

num_players = {
  'Round1' => 4,
  'Round2' => 4,
  'Finals' => 2,
}.fetch(stage)

num_workers = 5
num_workers.times.map do |worker_id|
  # generate config file
  template = File.read("localrunner-configs/tcp-vs-tcp-batch.json.erb")
  base_port = 32000 + worker_id * 4
  config_file = "/home/sergio/raic/localrunner-configs/tcp-vs-tcp-batch-#{worker_id}.json"

  File.write(config_file, ERB.new(template).result(binding))

  Process.fork do
    tasks_per_worker = num_attempts.fdiv(num_workers).ceil
    tasks_per_worker.times do |x|
      file_num = worker_id * tasks_per_worker + x
      Dir.chdir(local_runner_dir) do
        server_cmd = "./aicup2020 --config #{config_file} --batch-mode --save-results results/result#{'%04d' % (file_num)}.json 2>/dev/null &"
        system(server_cmd)
        sleep(0.5)
      end

      Dir.chdir(strategy_dir) do
        if num_players == 4
          client_cmd = "./bin/aicup2020-#{baseline_versions[1]} 127.0.0.1 #{base_port + 1} >/dev/null 2>&1 &"
          system(client_cmd)

          client_cmd = "./bin/aicup2020-#{baseline_versions[2]} 127.0.0.1 #{base_port + 2} >/dev/null 2>&1 &"
          system(client_cmd)
        end

        client_cmd = "./bin/aicup2020-#{baseline_versions[0]} 127.0.0.1 #{base_port + 3} >/dev/null 2>&1 &"
        system(client_cmd)

        client_cmd = "./bin/aicup2020-current-for-bench 127.0.0.1 #{base_port + 0}"
        system(client_cmd)
      end
      progressbar.increment
    end

  end
end

Process.waitall

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
    seeds << "place #{rank} - #{res['seed']}"
  end
end
puts tally

puts "seeds of failed games:"
puts seeds
