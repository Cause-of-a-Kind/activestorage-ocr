# frozen_string_literal: true

module ActiveStorage
  module Ocr
    class Client
      def initialize(config: ActiveStorage::Ocr.configuration)
        @config = config
      end

      # Extract text from an ActiveStorage blob
      def extract_text(blob)
        blob.open do |file|
          extract_text_from_file(file, blob.content_type, blob.filename.to_s)
        end
      end

      # Extract text from a file path
      def extract_text_from_path(path, content_type: nil, filename: nil)
        content_type ||= Marcel::MimeType.for(Pathname.new(path))
        filename ||= File.basename(path)

        File.open(path, "rb") do |file|
          extract_text_from_file(file, content_type, filename)
        end
      end

      # Extract text from an IO object
      def extract_text_from_file(file, content_type, filename)
        response = connection.post("/ocr") do |req|
          req.body = {
            file: Faraday::Multipart::FilePart.new(
              file,
              content_type,
              filename
            )
          }
        end

        handle_response(response)
      rescue Faraday::ConnectionFailed, Faraday::TimeoutError => e
        raise ConnectionError, "Failed to connect to OCR server: #{e.message}"
      end

      # Check if the OCR server is healthy
      def healthy?
        response = connection.get("/health")
        response.success? && JSON.parse(response.body)["status"] == "ok"
      rescue StandardError
        false
      end

      # Get server info
      def server_info
        response = connection.get("/info")
        JSON.parse(response.body, symbolize_names: true)
      rescue Faraday::ConnectionFailed, Faraday::TimeoutError => e
        raise ConnectionError, "Failed to connect to OCR server: #{e.message}"
      end

      private

      def connection
        @connection ||= Faraday.new(url: @config.server_url) do |f|
          f.request :multipart
          f.options.timeout = @config.timeout
          f.options.open_timeout = @config.open_timeout
          f.adapter Faraday.default_adapter
        end
      end

      def handle_response(response)
        unless response.success?
          error_body = JSON.parse(response.body) rescue {}
          raise ServerError, error_body["error"] || "OCR server returned #{response.status}"
        end

        data = JSON.parse(response.body, symbolize_names: true)

        Result.new(
          text: data[:text],
          confidence: data[:confidence],
          processing_time_ms: data[:processing_time_ms],
          warnings: data[:warnings] || []
        )
      end
    end
  end
end
