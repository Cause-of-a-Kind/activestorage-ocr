# frozen_string_literal: true

require "test_helper"

class ActiveStorage::Ocr::ResultTest < ActiveStorage::Ocr::TestCase
  def test_success_returns_true_when_text_present
    result = ActiveStorage::Ocr::Result.new(
      text: "Hello World",
      confidence: 0.95,
      processing_time_ms: 100
    )
    assert result.success?
  end

  def test_success_returns_false_when_text_empty
    result = ActiveStorage::Ocr::Result.new(
      text: "",
      confidence: 0.0,
      processing_time_ms: 100
    )
    refute result.success?
  end

  def test_success_returns_false_when_text_nil
    result = ActiveStorage::Ocr::Result.new(
      text: nil,
      confidence: 0.0,
      processing_time_ms: 100
    )
    refute result.success?
  end

  def test_to_h_returns_hash_representation
    result = ActiveStorage::Ocr::Result.new(
      text: "Test",
      confidence: 0.9,
      processing_time_ms: 50,
      warnings: ["Low quality"],
      engine: "ocrs"
    )

    expected = {
      text: "Test",
      confidence: 0.9,
      processing_time_ms: 50,
      warnings: ["Low quality"],
      engine: "ocrs"
    }
    assert_equal expected, result.to_h
  end

  def test_engine_attribute
    result = ActiveStorage::Ocr::Result.new(
      text: "Test",
      confidence: 0.9,
      processing_time_ms: 50,
      engine: "leptess"
    )

    assert_equal "leptess", result.engine
  end

  def test_engine_defaults_to_nil
    result = ActiveStorage::Ocr::Result.new(
      text: "Test",
      confidence: 0.9,
      processing_time_ms: 50
    )

    assert_nil result.engine
  end

  def test_to_metadata_returns_activestorage_compatible_metadata
    result = ActiveStorage::Ocr::Result.new(
      text: "Extracted text",
      confidence: 0.85,
      processing_time_ms: 200,
      engine: "ocrs"
    )

    metadata = result.to_metadata
    assert_equal "Extracted text", metadata[:ocr_text]
    assert_in_delta 0.85, metadata[:ocr_confidence], 0.01
    assert_equal "ocrs", metadata[:ocr_engine]
    assert metadata[:ocr_processed_at].is_a?(String)
  end
end
