# frozen_string_literal: true

require "test_helper"
require "stringio"

class ActiveStorage::Ocr::ClientTest < ActiveStorage::Ocr::TestCase
  def setup
    super
    @server_url = "http://localhost:9292"
    ActiveStorage::Ocr.configure do |config|
      config.server_url = @server_url
    end
    @client = ActiveStorage::Ocr::Client.new
    @success_response = {
      text: "Hello World",
      confidence: 0.95,
      processing_time_ms: 150,
      warnings: []
    }
  end

  def test_healthy_returns_true_when_server_responds_with_ok
    stub_request(:get, "#{@server_url}/health")
      .to_return(status: 200, body: { status: "ok" }.to_json)

    assert @client.healthy?
  end

  def test_healthy_returns_false_when_server_unreachable
    stub_request(:get, "#{@server_url}/health")
      .to_raise(Faraday::ConnectionFailed)

    refute @client.healthy?
  end

  def test_healthy_returns_false_when_server_returns_error
    stub_request(:get, "#{@server_url}/health")
      .to_return(status: 500)

    refute @client.healthy?
  end

  def test_server_info_returns_server_information
    info = {
      version: "0.1.0",
      supported_formats: ["image/png", "image/jpeg"],
      supported_languages: ["eng"],
      max_file_size_bytes: 10_485_760,
      default_language: "eng"
    }

    stub_request(:get, "#{@server_url}/info")
      .to_return(status: 200, body: info.to_json)

    result = @client.server_info
    assert_equal "0.1.0", result[:version]
    assert_includes result[:supported_formats], "image/png"
  end

  def test_server_info_raises_connection_error_when_unreachable
    stub_request(:get, "#{@server_url}/info")
      .to_raise(Faraday::ConnectionFailed)

    assert_raises(ActiveStorage::Ocr::ConnectionError) do
      @client.server_info
    end
  end

  def test_extract_text_from_file_returns_result_on_success
    stub_request(:post, "#{@server_url}/ocr")
      .to_return(status: 200, body: @success_response.to_json)

    file = StringIO.new("fake image data")
    result = @client.extract_text_from_file(file, "image/png", "test.png")

    assert_instance_of ActiveStorage::Ocr::Result, result
    assert_equal "Hello World", result.text
    assert_in_delta 0.95, result.confidence, 0.01
  end

  def test_extract_text_from_file_raises_server_error_on_failure
    stub_request(:post, "#{@server_url}/ocr")
      .to_return(status: 500, body: { error: "Processing failed" }.to_json)

    file = StringIO.new("fake image data")

    error = assert_raises(ActiveStorage::Ocr::ServerError) do
      @client.extract_text_from_file(file, "image/png", "test.png")
    end
    assert_equal "Processing failed", error.message
  end

  def test_extract_text_from_file_raises_connection_error_when_unreachable
    stub_request(:post, "#{@server_url}/ocr")
      .to_raise(Faraday::ConnectionFailed)

    file = StringIO.new("fake image data")

    assert_raises(ActiveStorage::Ocr::ConnectionError) do
      @client.extract_text_from_file(file, "image/png", "test.png")
    end
  end
end
