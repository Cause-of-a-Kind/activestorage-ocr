# frozen_string_literal: true

module ActiveStorage
  module Ocr
    class Result
      attr_reader :text, :confidence, :processing_time_ms, :warnings

      def initialize(text:, confidence:, processing_time_ms:, warnings: [])
        @text = text
        @confidence = confidence
        @processing_time_ms = processing_time_ms
        @warnings = warnings
      end

      def success?
        !text.nil? && !text.empty?
      end

      def to_h
        {
          text: text,
          confidence: confidence,
          processing_time_ms: processing_time_ms,
          warnings: warnings
        }
      end

      def to_metadata
        {
          ocr_text: text,
          ocr_confidence: confidence,
          ocr_processed_at: Time.now.utc.iso8601
        }
      end
    end
  end
end
