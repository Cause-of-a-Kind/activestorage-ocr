# frozen_string_literal: true

module ActiveStorage
  module Ocr
    class Configuration
      attr_accessor :server_url, :timeout, :open_timeout, :content_types

      def initialize
        @server_url = ENV.fetch("ACTIVESTORAGE_OCR_SERVER_URL", "http://localhost:9292")
        @timeout = ENV.fetch("ACTIVESTORAGE_OCR_TIMEOUT", 30).to_i
        @open_timeout = ENV.fetch("ACTIVESTORAGE_OCR_OPEN_TIMEOUT", 5).to_i
        @content_types = default_content_types
      end

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

      def accept_content_type?(content_type)
        content_types.include?(content_type)
      end
    end
  end
end
