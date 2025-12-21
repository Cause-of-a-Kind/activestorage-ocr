# frozen_string_literal: true

module ActiveStorage
  module Ocr
    class Railtie < Rails::Railtie
      initializer "activestorage-ocr.add_analyzer" do
        config.after_initialize do
          # Prepend our analyzer so it runs before other analyzers
          if defined?(ActiveStorage) && ActiveStorage.respond_to?(:analyzers)
            ActiveStorage.analyzers.prepend(ActiveStorage::Ocr::Analyzer)
          end
        end
      end

      # Add rake tasks
      rake_tasks do
        namespace :activestorage_ocr do
          desc "Install the OCR server binary"
          task :install do
            ActiveStorage::Ocr::Binary.install!
          end

          desc "Check OCR server health"
          task health: :environment do
            client = ActiveStorage::Ocr::Client.new
            if client.healthy?
              puts "OCR server is healthy"
              info = client.server_info
              puts "  Version: #{info[:version]}"
              puts "  Supported formats: #{info[:supported_formats].join(', ')}"
            else
              puts "OCR server is not responding"
              exit 1
            end
          end

          desc "Start the OCR server"
          task :start do
            binary = ActiveStorage::Ocr::Binary.binary_path
            unless File.executable?(binary)
              puts "Binary not found. Installing..."
              ActiveStorage::Ocr::Binary.install!
            end

            config = ActiveStorage::Ocr.configuration
            host = config.server_host.gsub(%r{https?://}, "")
            port = config.server_port

            puts "Starting OCR server on #{host}:#{port}..."
            exec(binary, "--host", host, "--port", port.to_s)
          end

          desc "Show binary info"
          task :info do
            puts "Platform: #{ActiveStorage::Ocr::Binary.platform}"
            puts "Binary path: #{ActiveStorage::Ocr::Binary.binary_path}"
            puts "Installed: #{ActiveStorage::Ocr::Binary.installed?}"
            puts "Version: #{ActiveStorage::Ocr::Binary.version}"
          end
        end
      end
    end
  end
end
