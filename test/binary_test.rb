# frozen_string_literal: true

require "test_helper"

class BinaryTest < Minitest::Test
  def test_platform_detection
    platform = ActiveStorage::Ocr::Binary.platform
    assert_match(/^(darwin|linux)-(x86_64|aarch64)$/, platform)
  end

  def test_binary_path
    path = ActiveStorage::Ocr::Binary.binary_path
    assert_includes path, "activestorage-ocr-server"
  end

  def test_install_dir
    dir = ActiveStorage::Ocr::Binary.install_dir
    assert File.directory?(dir)
  end

  def test_version_matches_gem_version
    assert_equal ActiveStorage::Ocr::VERSION, ActiveStorage::Ocr::Binary.version
  end

  def test_download_url_format
    url = ActiveStorage::Ocr::Binary.download_url
    assert_match %r{^https://github.com/.+/releases/download/v.+/activestorage-ocr-server-.+\.tar\.gz$}, url
  end
end
