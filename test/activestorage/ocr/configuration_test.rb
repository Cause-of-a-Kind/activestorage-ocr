# frozen_string_literal: true

require "test_helper"

class ActiveStorage::Ocr::ConfigurationTest < ActiveStorage::Ocr::TestCase
  def test_server_url_defaults_to_localhost
    config = ActiveStorage::Ocr::Configuration.new
    assert_equal "http://localhost:9292", config.server_url
  end

  def test_server_url_can_be_configured
    config = ActiveStorage::Ocr::Configuration.new
    config.server_url = "http://ocr.example.com:8080"
    assert_equal "http://ocr.example.com:8080", config.server_url
  end

  def test_timeout_defaults_to_30_seconds
    config = ActiveStorage::Ocr::Configuration.new
    assert_equal 30, config.timeout
  end

  def test_content_types_includes_common_image_formats
    config = ActiveStorage::Ocr::Configuration.new
    assert_includes config.content_types, "image/png"
    assert_includes config.content_types, "image/jpeg"
    assert_includes config.content_types, "image/gif"
  end

  def test_content_types_includes_pdf
    config = ActiveStorage::Ocr::Configuration.new
    assert_includes config.content_types, "application/pdf"
  end

  def test_accept_content_type_returns_true_for_supported_types
    config = ActiveStorage::Ocr::Configuration.new
    assert config.accept_content_type?("image/png")
    assert config.accept_content_type?("application/pdf")
  end

  def test_accept_content_type_returns_false_for_unsupported_types
    config = ActiveStorage::Ocr::Configuration.new
    refute config.accept_content_type?("video/mp4")
  end
end
