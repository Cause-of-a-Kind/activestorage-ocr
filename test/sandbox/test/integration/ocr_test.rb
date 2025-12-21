# frozen_string_literal: true

require "test_helper"

class OcrIntegrationTest < ActiveSupport::TestCase
  # NOTE: These tests require the Rust OCR server to be running
  # Start it with: cd rust && cargo run --release

  def setup
    @client = ActiveStorage::Ocr::Client.new
  end

  test "server is healthy" do
    skip "OCR server not running" unless @client.healthy?
    assert @client.healthy?
  end

  test "extracts text from PNG image" do
    skip "OCR server not running" unless @client.healthy?

    result = @client.extract_text_from_path(
      fixture_file("sample_text.png"),
      content_type: "image/png"
    )

    assert result.success?
    assert_includes result.text, "Hello"
    assert_includes result.text, "World"
    assert result.confidence > 0
  end

  test "extracts text from attached file" do
    skip "OCR server not running" unless @client.healthy?

    doc = Document.create!(name: "test")
    doc.file.attach(
      io: File.open(fixture_file("sample_text.png")),
      filename: "sample_text.png",
      content_type: "image/png"
    )

    result = @client.extract_text(doc.file)

    assert result.success?
    assert_includes result.text, "Hello"
  end

  test "extracts text from PDF" do
    skip "OCR server not running" unless @client.healthy?

    result = @client.extract_text_from_path(
      fixture_file("sample_text.pdf"),
      content_type: "application/pdf"
    )

    assert result.success?
    assert_includes result.text, "Hello"
    assert result.confidence > 0
  end
end
