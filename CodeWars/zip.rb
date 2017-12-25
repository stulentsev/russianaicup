files = Dir['*.rb'].grep_v(/remote_client_process.rb/).grep_v(/runner.rb/)

base_name = 'Archive'
existing_archive_count = Dir['Archive*.zip'].length
zip_name = "#{base_name}.#{existing_archive_count + 1}.zip"

`zip #{zip_name} #{files.join(' ')}`
