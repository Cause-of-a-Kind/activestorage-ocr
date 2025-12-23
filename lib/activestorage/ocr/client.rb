# frozen_string_literal: true

module ActiveStorage
  module Ocr
    # HTTP client for communicating with the OCR server.
    #
    # The Client handles all communication with the Rust OCR server,
    # including file uploads and response parsing.
    #
    # == Basic Usage
    #
    #   client = ActiveStorage::Ocr::Client.new
    #
    #   # Extract text from an Active Storage blob
    #   result = client.extract_text(document.file)
    #
    #   # Extract text from a file path
    #   result = client.extract_text_from_path("/path/to/image.png")
    #
    #   # Check server health
    #   client.healthy?  # => true
    #
    class Client
      # Creates a new Client.
      #
      # ==== Parameters
      #
      # * +config+ - Configuration object (defaults to global configuration)
      def initialize(config: ActiveStorage::Ocr.configuration)
        @config = config
      end

      # Extracts text from an Active Storage blob.
      #
      # Opens the blob temporarily and sends it to the OCR server.
      #
      # ==== Parameters
      #
      # * +blob+ - An ActiveStorage::Blob instance
      #
      # ==== Returns
      #
      # A Result object with extracted text and metadata.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
      def extract_text(blob)
        blob.open do |file|
          extract_text_from_file(file, blob.content_type, blob.filename.to_s)
        end
      end

      # Extracts text from a file path.
      #
      # ==== Parameters
      #
      # * +path+ - Path to the file
      # * +content_type+ - MIME type (auto-detected if not provided)
      # * +filename+ - Filename to send (defaults to basename of path)
      #
      # ==== Returns
      #
      # A Result object with extracted text and metadata.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
      def extract_text_from_path(path, content_type: nil, filename: nil)
        content_type ||= Marcel::MimeType.for(Pathname.new(path))
        filename ||= File.basename(path)

        File.open(path, "rb") do |file|
          extract_text_from_file(file, content_type, filename)
        end
      end

      # Extracts text from an IO object.
      #
      # This is the low-level method that performs the actual HTTP request.
      #
      # ==== Parameters
      #
      # * +file+ - An IO object (File, StringIO, etc.)
      # * +content_type+ - MIME type of the file
      # * +filename+ - Filename to send to the server
      #
      # ==== Returns
      #
      # A Result object with extracted text and metadata.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
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

      # Checks if the OCR server is healthy.
      #
      # ==== Returns
      #
      # +true+ if the server responds with status "ok", +false+ otherwise.
      def healthy?
        response = connection.get("/health")
        response.success? && JSON.parse(response.body)["status"] == "ok"
      rescue StandardError
        false
      end

      # Gets information about the OCR server.
      #
      # ==== Returns
      #
      # A Hash with server information including:
      # * +:version+ - Server version
      # * +:supported_formats+ - Array of supported MIME types
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      def server_info
        response = connection.get("/info")
        JSON.parse(response.body, symbolize_names: true)
      rescue Faraday::ConnectionFailed, Faraday::TimeoutError => e
        raise ConnectionError, "Failed to connect to OCR server: #{e.message}"
      end

      private

      # Returns the Faraday connection, creating it if necessary.
      def connection
        @connection ||= Faraday.new(url: @config.server_url) do |f|
          f.request :multipart
          f.options.timeout = @config.timeout
          f.options.open_timeout = @config.open_timeout
          f.adapter Faraday.default_adapter
        end
      end

      # Parses the server response and returns a Result.
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
