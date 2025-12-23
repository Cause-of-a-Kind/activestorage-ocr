# frozen_string_literal: true

module ActiveStorage
  module Ocr
    # Active Storage analyzer for OCR processing.
    #
    # This analyzer integrates with Active Storage's analysis system to
    # automatically extract text from uploaded images and PDFs.
    #
    # When registered with Active Storage, it will automatically process
    # supported file types and store OCR results in the blob's metadata.
    #
    # == Metadata
    #
    # After analysis, blobs will have the following metadata:
    # * +ocr_text+ - The extracted text
    # * +ocr_confidence+ - Confidence score (0.0 to 1.0)
    # * +ocr_processed_at+ - ISO 8601 timestamp
    #
    # == Example
    #
    #   document.file.analyze
    #   document.file.metadata["ocr_text"]  # => "Extracted text..."
    #
    class Analyzer < ActiveStorage::Analyzer
      # Determines if this analyzer can process the blob.
      #
      # ==== Parameters
      #
      # * +blob+ - An ActiveStorage::Blob instance
      #
      # ==== Returns
      #
      # +true+ if the blob's content type is supported.
      def self.accept?(blob)
        ActiveStorage::Ocr.configuration.accept_content_type?(blob.content_type)
      end

      # Extracts OCR metadata from the blob.
      #
      # Called by Active Storage during analysis.
      #
      # ==== Returns
      #
      # A Hash containing OCR results, or an empty Hash if extraction fails.
      def metadata
        result = extract_text
        return {} unless result&.success?

        result.to_metadata
      rescue Error => e
        Rails.logger.error("[ActiveStorage::Ocr] OCR failed: #{e.message}") if defined?(Rails)
        {}
      end

      private

      # Performs the OCR extraction.
      def extract_text
        Client.new.extract_text(blob)
      end
    end
  end
end
