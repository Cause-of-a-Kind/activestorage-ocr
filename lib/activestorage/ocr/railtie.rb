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
    # * +activestorage_ocr:compare+ - Compare OCR results from different engines
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
          desc "Install the OCR server binary (variant=ocrs|leptess|all, path=./bin/dist)"
          task :install do
            path = ENV["path"]
            variant = (ENV["variant"] || "ocrs").to_sym
            ActiveStorage::Ocr::Binary.install!(path: path, variant: variant)
          end

          desc "Check OCR server health"
          task health: :environment do
            client = ActiveStorage::Ocr::Client.new
            if client.healthy?
              puts "OCR server is healthy"
              info = client.server_info
              puts "  Version: #{info[:version]}"
              puts "  Default engine: #{info[:default_engine]}"
              puts "  Available engines:"
              info[:available_engines].each do |engine|
                puts "    - #{engine[:name]}: #{engine[:description]}"
              end
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
              variant = (ENV["variant"] || "ocrs").to_sym
              ActiveStorage::Ocr::Binary.install!(variant: variant)
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
            puts ""
            puts "Available variants:"
            ActiveStorage::Ocr::Binary::VARIANTS.each do |name, info|
              puts "  #{name}: #{info[:description]}"
            end
          end

          desc "Compare OCR engines on a file (file=/path/to/image.png)"
          task compare: :environment do
            file_path = ENV["file"]
            unless file_path
              puts "Usage: rake activestorage_ocr:compare file=/path/to/image.png"
              exit 1
            end

            unless File.exist?(file_path)
              puts "File not found: #{file_path}"
              exit 1
            end

            client = ActiveStorage::Ocr::Client.new
            puts "Comparing OCR engines on: #{file_path}"
            puts ""

            begin
              comparison = client.compare_from_path(file_path)

              comparison.each do |engine, result|
                puts "#{engine.to_s.upcase}:"
                puts "  Text length: #{result.text.length} characters"
                puts "  Confidence: #{(result.confidence * 100).round(1)}%"
                puts "  Processing time: #{result.processing_time_ms}ms"
                if result.warnings.any?
                  puts "  Warnings: #{result.warnings.join(', ')}"
                end
                puts ""
              end

              # Summary
              faster = comparison.min_by { |_, r| r.processing_time_ms }
              higher_conf = comparison.max_by { |_, r| r.confidence }
              puts "Summary:"
              puts "  Faster engine: #{faster[0]} (#{faster[1].processing_time_ms}ms)"
              puts "  Higher confidence: #{higher_conf[0]} (#{(higher_conf[1].confidence * 100).round(1)}%)"
            rescue ActiveStorage::Ocr::ConnectionError => e
              puts "Error: #{e.message}"
              puts "Make sure the OCR server is running with both engines enabled."
              exit 1
            rescue ActiveStorage::Ocr::ServerError => e
              puts "Server error: #{e.message}"
              puts "This engine may not be available. Check server configuration."
              exit 1
            end
          end
        end
      end
    end
  end
end
