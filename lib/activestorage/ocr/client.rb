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
    #   # Use a specific OCR engine
    #   result = client.extract_text(document.file, engine: :leptess)
    #
    #   # Compare results from both engines
    #   comparison = client.compare(document.file)
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
      # * +engine+ - OCR engine to use (:ocrs or :leptess). Defaults to configured engine.
      #
      # ==== Returns
      #
      # A Result object with extracted text and metadata.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
      def extract_text(blob, engine: nil)
        blob.open do |file|
          extract_text_from_file(file, blob.content_type, blob.filename.to_s, engine: engine)
        end
      end

      # Extracts text from a file path.
      #
      # ==== Parameters
      #
      # * +path+ - Path to the file
      # * +content_type+ - MIME type (auto-detected if not provided)
      # * +filename+ - Filename to send (defaults to basename of path)
      # * +engine+ - OCR engine to use (:ocrs or :leptess). Defaults to configured engine.
      #
      # ==== Returns
      #
      # A Result object with extracted text and metadata.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
      def extract_text_from_path(path, content_type: nil, filename: nil, engine: nil)
        content_type ||= Marcel::MimeType.for(Pathname.new(path))
        filename ||= File.basename(path)

        File.open(path, "rb") do |file|
          extract_text_from_file(file, content_type, filename, engine: engine)
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
      # * +engine+ - OCR engine to use (:ocrs or :leptess). Defaults to configured engine.
      #
      # ==== Returns
      #
      # A Result object with extracted text and metadata.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
      def extract_text_from_file(file, content_type, filename, engine: nil)
        target_engine = engine || @config.engine
        endpoint = ocr_endpoint_for(target_engine)

        response = connection.post(endpoint) do |req|
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

      # Compares OCR results from both engines.
      #
      # Runs OCR on the same file using both ocrs and leptess engines,
      # allowing you to compare accuracy and performance.
      #
      # ==== Parameters
      #
      # * +blob+ - An ActiveStorage::Blob instance
      #
      # ==== Returns
      #
      # A Hash with :ocrs and :leptess keys, each containing a Result object.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
      def compare(blob)
        ocrs_result = extract_text(blob, engine: :ocrs)
        leptess_result = extract_text(blob, engine: :leptess)

        {
          ocrs: ocrs_result,
          leptess: leptess_result
        }
      end

      # Compares OCR results from both engines using a file path.
      #
      # ==== Parameters
      #
      # * +path+ - Path to the file
      # * +content_type+ - MIME type (auto-detected if not provided)
      # * +filename+ - Filename to send (defaults to basename of path)
      #
      # ==== Returns
      #
      # A Hash with :ocrs and :leptess keys, each containing a Result object.
      def compare_from_path(path, content_type: nil, filename: nil)
        content_type ||= Marcel::MimeType.for(Pathname.new(path))
        filename ||= File.basename(path)

        ocrs_result = extract_text_from_path(path, content_type: content_type, filename: filename, engine: :ocrs)
        leptess_result = extract_text_from_path(path, content_type: content_type, filename: filename, engine: :leptess)

        {
          ocrs: ocrs_result,
          leptess: leptess_result
        }
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

      # Returns the OCR endpoint path for the given engine.
      #
      # ==== Parameters
      #
      # * +engine+ - Engine name (:ocrs or :leptess)
      #
      # ==== Returns
      #
      # The endpoint path string (e.g., "/ocr" or "/ocr/leptess")
      def ocr_endpoint_for(engine)
        case engine.to_sym
        when :ocrs
          "/ocr"
        when :leptess
          "/ocr/leptess"
        else
          raise ArgumentError, "Unknown engine: #{engine}"
        end
      end

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
          warnings: data[:warnings] || [],
          engine: data[:engine]
        )
      end
    end
  end
end
