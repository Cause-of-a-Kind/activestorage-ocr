# frozen_string_literal: true

module ActiveStorage
  module Ocr
    class Analyzer < ActiveStorage::Analyzer
      def self.accept?(blob)
        ActiveStorage::Ocr.configuration.accept_content_type?(blob.content_type)
      end

      def metadata
        result = extract_text
        return {} unless result&.success?

        result.to_metadata
      rescue Error => e
        Rails.logger.error("[ActiveStorage::Ocr] OCR failed: #{e.message}") if defined?(Rails)
        {}
      end

      private

      def extract_text
        Client.new.extract_text(blob)
      end
    end
  end
end
