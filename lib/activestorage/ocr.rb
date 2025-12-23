# frozen_string_literal: true

require "faraday"
require "faraday/multipart"
require "json"

require_relative "ocr/version"
require_relative "ocr/configuration"
require_relative "ocr/client"
require_relative "ocr/result"
require_relative "ocr/binary"

if defined?(Rails)
  require_relative "ocr/analyzer"
  require_relative "ocr/railtie"
end

module ActiveStorage
  # OCR support for Rails Active Storage attachments.
  #
  # This module provides optical character recognition (OCR) for files stored
  # with Active Storage using a high-performance Rust server with the pure-Rust
  # +ocrs+ OCR engine.
  #
  # == Configuration
  #
  #   ActiveStorage::Ocr.configure do |config|
  #     config.server_url = "http://localhost:9292"
  #     config.timeout = 30
  #   end
  #
  # == Basic Usage
  #
  #   # Extract text from an Active Storage blob
  #   result = ActiveStorage::Ocr.extract_text(document.file)
  #   result.text        # => "Extracted text..."
  #   result.confidence  # => 0.95
  #
  # == Error Handling
  #
  # All errors inherit from ActiveStorage::Ocr::Error:
  # - ActiveStorage::Ocr::ConnectionError - server unreachable
  # - ActiveStorage::Ocr::ServerError - server returned an error
  #
  module Ocr
    # Base error class for all OCR errors.
    class Error < StandardError; end

    # Raised when the OCR server returns an error response.
    class ServerError < Error; end

    # Raised when the OCR server is unreachable or times out.
    class ConnectionError < Error; end

    class << self
      attr_writer :configuration

      # Returns the current configuration.
      #
      # Creates a new Configuration with defaults if none exists.
      def configuration
        @configuration ||= Configuration.new
      end

      # Configures the OCR module.
      #
      # ==== Example
      #
      #   ActiveStorage::Ocr.configure do |config|
      #     config.server_url = "http://localhost:9292"
      #     config.timeout = 60
      #   end
      def configure
        yield(configuration)
      end

      # Resets configuration to defaults.
      #
      # Useful for testing.
      def reset_configuration!
        @configuration = Configuration.new
      end

      # Extracts text from an Active Storage blob.
      #
      # This is a convenience method that creates a new Client and calls
      # extract_text on it.
      #
      # ==== Parameters
      #
      # * +blob+ - An ActiveStorage::Blob instance
      #
      # ==== Returns
      #
      # A Result object containing extracted text and metadata.
      #
      # ==== Raises
      #
      # * ConnectionError - if the server is unreachable
      # * ServerError - if the server returns an error
      def extract_text(blob)
        Client.new.extract_text(blob)
      end
    end
  end
end
