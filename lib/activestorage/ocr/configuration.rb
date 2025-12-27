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
    #     config.engine = :leptess  # Use Tesseract engine instead of default ocrs
    #   end
    #
    # == Environment Variables
    #
    # * +ACTIVESTORAGE_OCR_SERVER_URL+ - OCR server URL (default: http://localhost:9292)
    # * +ACTIVESTORAGE_OCR_TIMEOUT+ - Request timeout in seconds (default: 30)
    # * +ACTIVESTORAGE_OCR_OPEN_TIMEOUT+ - Connection timeout in seconds (default: 5)
    # * +ACTIVESTORAGE_OCR_ENGINE+ - OCR engine to use: ocrs (default) or leptess
    #
    class Configuration
      # Valid OCR engine names
      VALID_ENGINES = %i[ocrs leptess].freeze

      # The URL of the OCR server.
      attr_accessor :server_url

      # Request timeout in seconds.
      attr_accessor :timeout

      # Connection open timeout in seconds.
      attr_accessor :open_timeout

      # Array of MIME types that the analyzer will process.
      attr_accessor :content_types

      # The OCR engine to use (:ocrs or :leptess).
      # Default is :ocrs (pure Rust, no dependencies).
      # Use :leptess for Tesseract-based OCR (better for messy images).
      attr_reader :engine

      # Creates a new Configuration with default values.
      #
      # Defaults are read from environment variables if set.
      def initialize
        @server_url = ENV.fetch("ACTIVESTORAGE_OCR_SERVER_URL", "http://localhost:9292")
        @timeout = ENV.fetch("ACTIVESTORAGE_OCR_TIMEOUT", 30).to_i
        @open_timeout = ENV.fetch("ACTIVESTORAGE_OCR_OPEN_TIMEOUT", 5).to_i
        @content_types = default_content_types
        self.engine = ENV.fetch("ACTIVESTORAGE_OCR_ENGINE", "ocrs").to_sym
      end

      # Set the OCR engine to use.
      #
      # ==== Parameters
      #
      # * +value+ - Engine name (:ocrs or :leptess)
      #
      # ==== Raises
      #
      # * +ArgumentError+ if an invalid engine name is provided
      def engine=(value)
        value = value.to_sym
        unless VALID_ENGINES.include?(value)
          raise ArgumentError, "Invalid engine: #{value}. Valid engines: #{VALID_ENGINES.join(', ')}"
        end

        @engine = value
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
