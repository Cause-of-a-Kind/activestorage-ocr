# frozen_string_literal: true

module ActiveStorage
  module Ocr
    # Represents the result of an OCR operation.
    #
    # Contains the extracted text, confidence score, and processing metadata.
    #
    # == Example
    #
    #   result = client.extract_text(blob)
    #   if result.success?
    #     puts result.text
    #     puts "Confidence: #{result.confidence}"
    #   end
    #
    class Result
      # The extracted text content.
      attr_reader :text

      # Confidence score from 0.0 to 1.0.
      attr_reader :confidence

      # Time taken to process the file in milliseconds.
      attr_reader :processing_time_ms

      # Array of warning messages from the OCR server.
      attr_reader :warnings

      # The OCR engine that processed this result (e.g., "ocrs" or "leptess").
      attr_reader :engine

      # Preprocessing statistics (Hash with :preset, :total_time_ms, :steps).
      # nil if preprocessing was skipped.
      attr_reader :preprocessing

      # Creates a new Result.
      #
      # ==== Parameters
      #
      # * +text+ - The extracted text
      # * +confidence+ - Confidence score (0.0 to 1.0)
      # * +processing_time_ms+ - Processing time in milliseconds
      # * +warnings+ - Array of warning messages (optional)
      # * +engine+ - The OCR engine used (optional)
      # * +preprocessing+ - Preprocessing stats hash (optional)
      def initialize(text:, confidence:, processing_time_ms:, warnings: [], engine: nil, preprocessing: nil)
        @text = text
        @confidence = confidence
        @processing_time_ms = processing_time_ms
        @warnings = warnings
        @engine = engine
        @preprocessing = preprocessing
      end

      # Returns whether OCR successfully extracted text.
      #
      # ==== Returns
      #
      # +true+ if text was extracted, +false+ if text is nil or empty.
      def success?
        !text.nil? && !text.empty?
      end

      # Returns the preprocessing time in milliseconds, or 0 if not preprocessed.
      def preprocessing_time_ms
        preprocessing&.dig(:total_time_ms) || 0
      end

      # Returns the preprocessing preset used, or nil if not preprocessed.
      def preprocessing_preset
        preprocessing&.dig(:preset)
      end

      # Converts the result to a Hash.
      #
      # ==== Returns
      #
      # A Hash with all result attributes.
      def to_h
        {
          text: text,
          confidence: confidence,
          processing_time_ms: processing_time_ms,
          warnings: warnings,
          engine: engine,
          preprocessing: preprocessing
        }
      end

      # Converts the result to Active Storage metadata format.
      #
      # This format is suitable for storing in blob metadata.
      #
      # ==== Returns
      #
      # A Hash with +:ocr_text+, +:ocr_confidence+, +:ocr_engine+, and +:ocr_processed_at+.
      def to_metadata
        {
          ocr_text: text,
          ocr_confidence: confidence,
          ocr_engine: engine,
          ocr_processed_at: Time.now.utc.iso8601
        }
      end
    end
  end
end
