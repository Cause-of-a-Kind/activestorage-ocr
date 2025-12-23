# frozen_string_literal: true

module ActiveStorage
  module Ocr
    # Configuration options for the OCR module.
    #
    # Settings can be configured via environment variables or the configure block:
    #
    #   ActiveStorage::Ocr.configure do |config|
    #     config.server_url = "http://localhost:9292"
    #     config.timeout = 60
    #   end
    #
    # == Environment Variables
    #
    # * +ACTIVESTORAGE_OCR_SERVER_URL+ - OCR server URL (default: http://localhost:9292)
    # * +ACTIVESTORAGE_OCR_TIMEOUT+ - Request timeout in seconds (default: 30)
    # * +ACTIVESTORAGE_OCR_OPEN_TIMEOUT+ - Connection timeout in seconds (default: 5)
    #
    class Configuration
      # The URL of the OCR server.
      attr_accessor :server_url

      # Request timeout in seconds.
      attr_accessor :timeout

      # Connection open timeout in seconds.
      attr_accessor :open_timeout

      # Array of MIME types that the analyzer will process.
      attr_accessor :content_types

      # Creates a new Configuration with default values.
      #
      # Defaults are read from environment variables if set.
      def initialize
        @server_url = ENV.fetch("ACTIVESTORAGE_OCR_SERVER_URL", "http://localhost:9292")
        @timeout = ENV.fetch("ACTIVESTORAGE_OCR_TIMEOUT", 30).to_i
        @open_timeout = ENV.fetch("ACTIVESTORAGE_OCR_OPEN_TIMEOUT", 5).to_i
        @content_types = default_content_types
      end

      # Returns the default list of supported content types.
      #
      # Includes common image formats and PDF.
      def default_content_types
        %w[
          image/png
          image/jpeg
          image/gif
          image/bmp
          image/webp
          image/tiff
          application/pdf
        ]
      end

      # Checks if the given content type is supported.
      #
      # ==== Parameters
      #
      # * +content_type+ - A MIME type string (e.g., "image/png")
      #
      # ==== Returns
      #
      # +true+ if the content type is in the supported list, +false+ otherwise.
      def accept_content_type?(content_type)
        content_types.include?(content_type)
      end
    end
  end
end
