#! /usr/bin/env ruby
require 'bundler'
Bundler.require

require 'net/http'
require 'uri'

if ARGV.empty?
  puts "Usage: ./charts.rb https://russianaicup.ru/game/view/<ID>"
  exit(1)
end

uri = URI(ARGV[0])
content = Net::HTTP.get(uri)
doc = Nokogiri::HTML(content)

game_id = doc.xpath('//span[@class="run-player"]/@data-gameid')
token = doc.xpath('//span[@class="run-player"]/@data-token')

log_file_url = "https://russianaicup.ru/boombox/data/games/#{token}"
puts "Downloading #{log_file_url}"
output_file = "repeat_logs/#{game_id}.log"
bytes_written = File.write(output_file, Net::HTTP.get(URI(log_file_url)))
puts "Saved #{bytes_written} bytes to #{output_file}"

