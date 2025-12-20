# frozen_string_literal: true

require "faraday"
require "faraday/multipart"
require "json"

require_relative "ocr/version"
require_relative "ocr/configuration"
require_relative "ocr/client"
require_relative "ocr/result"

if defined?(Rails)
  require_relative "ocr/analyzer"
  require_relative "ocr/railtie"
end

module ActiveStorage
  module Ocr
    class Error < StandardError; end
    class ServerError < Error; end
    class ConnectionError < Error; end

    class << self
      attr_writer :configuration

      def configuration
        @configuration ||= Configuration.new
      end

      def configure
        yield(configuration)
      end

      def reset_configuration!
        @configuration = Configuration.new
      end

      # Convenience method to extract text from a blob
      def extract_text(blob)
        Client.new.extract_text(blob)
      end
    end
  end
end
