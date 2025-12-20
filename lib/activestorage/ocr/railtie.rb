# frozen_string_literal: true

module ActiveStorage
  module Ocr
    class Railtie < Rails::Railtie
      initializer "activestorage-ocr.configure" do
        ActiveSupport.on_load(:active_storage_blob) do
          ActiveStorage::Blob.analyzer_class = ->(blob) {
            if ActiveStorage::Ocr::Analyzer.accept?(blob)
              ActiveStorage::Ocr::Analyzer
            else
              ActiveStorage::Analyzer
            end
          }
        end
      end

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
        end
      end
    end
  end
end
