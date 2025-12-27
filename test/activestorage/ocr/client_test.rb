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

  # Engine selection tests

  def test_extract_text_from_file_uses_default_ocrs_endpoint
    stub_request(:post, "#{@server_url}/ocr")
      .to_return(status: 200, body: @success_response.to_json)

    file = StringIO.new("fake image data")
    @client.extract_text_from_file(file, "image/png", "test.png")

    assert_requested(:post, "#{@server_url}/ocr")
  end

  def test_extract_text_from_file_uses_leptess_endpoint_when_specified
    stub_request(:post, "#{@server_url}/ocr/leptess")
      .to_return(status: 200, body: @success_response.to_json)

    file = StringIO.new("fake image data")
    @client.extract_text_from_file(file, "image/png", "test.png", engine: :leptess)

    assert_requested(:post, "#{@server_url}/ocr/leptess")
  end

  def test_extract_text_from_file_uses_configured_engine
    ActiveStorage::Ocr.configure do |config|
      config.engine = :leptess
    end

    stub_request(:post, "#{@server_url}/ocr/leptess")
      .to_return(status: 200, body: @success_response.to_json)

    client = ActiveStorage::Ocr::Client.new
    file = StringIO.new("fake image data")
    client.extract_text_from_file(file, "image/png", "test.png")

    assert_requested(:post, "#{@server_url}/ocr/leptess")
  end

  def test_extract_text_from_file_per_request_engine_overrides_config
    ActiveStorage::Ocr.configure do |config|
      config.engine = :leptess
    end

    stub_request(:post, "#{@server_url}/ocr")
      .to_return(status: 200, body: @success_response.to_json)

    client = ActiveStorage::Ocr::Client.new
    file = StringIO.new("fake image data")
    client.extract_text_from_file(file, "image/png", "test.png", engine: :ocrs)

    assert_requested(:post, "#{@server_url}/ocr")
  end

  def test_extract_text_from_file_raises_on_invalid_engine
    file = StringIO.new("fake image data")

    assert_raises(ArgumentError) do
      @client.extract_text_from_file(file, "image/png", "test.png", engine: :invalid)
    end
  end

  def test_extract_text_from_file_returns_engine_in_result
    response_with_engine = @success_response.merge(engine: "ocrs")
    stub_request(:post, "#{@server_url}/ocr")
      .to_return(status: 200, body: response_with_engine.to_json)

    file = StringIO.new("fake image data")
    result = @client.extract_text_from_file(file, "image/png", "test.png")

    assert_equal "ocrs", result.engine
  end

  # Compare tests

  def test_compare_from_path_calls_both_engines
    stub_request(:post, "#{@server_url}/ocr")
      .to_return(status: 200, body: @success_response.merge(engine: "ocrs").to_json)
    stub_request(:post, "#{@server_url}/ocr/leptess")
      .to_return(status: 200, body: @success_response.merge(engine: "leptess").to_json)

    # Create a temporary file for the test
    require "tempfile"
    Tempfile.open(["test", ".png"]) do |f|
      f.write("fake image data")
      f.rewind

      comparison = @client.compare_from_path(f.path, content_type: "image/png")

      assert_instance_of Hash, comparison
      assert comparison.key?(:ocrs)
      assert comparison.key?(:leptess)
      assert_equal "ocrs", comparison[:ocrs].engine
      assert_equal "leptess", comparison[:leptess].engine
    end
  end
end
