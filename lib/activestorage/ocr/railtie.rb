# frozen_string_literal: true

module ActiveStorage
  module Ocr
    # Rails integration via Railtie.
    #
    # Automatically registers the OCR analyzer with Active Storage and
    # provides rake tasks for managing the OCR server.
    #
    # == Rake Tasks
    #
    # * +activestorage_ocr:install+ - Download and install the server binary
    # * +activestorage_ocr:start+ - Start the OCR server
    # * +activestorage_ocr:health+ - Check if the server is responding
    # * +activestorage_ocr:info+ - Show binary and platform information
    #
    class Railtie < Rails::Railtie
      # Registers the OCR analyzer with Active Storage.
      #
      # We use an initializer that runs after Active Storage's engine is loaded
      # to prepend our analyzer to the configuration's analyzers list.
      # This ensures our analyzer runs before the default image analyzers.
      initializer "activestorage-ocr.add_analyzer", after: "active_storage.configs" do |app|
        # Prepend to the config's analyzers list, which ActiveStorage::Engine
        # later copies to ActiveStorage.analyzers in its after_initialize
        app.config.active_storage.analyzers.prepend(ActiveStorage::Ocr::Analyzer)
      end

      # Defines rake tasks for server management.
      rake_tasks do
        namespace :activestorage_ocr do
          desc "Install the OCR server binary (optional: path=./bin/dist)"
          task :install do
            path = ENV["path"]
            ActiveStorage::Ocr::Binary.install!(path: path)
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
            uri = URI.parse(config.server_url)
            host = uri.host || "127.0.0.1"
            port = uri.port || 9292

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
