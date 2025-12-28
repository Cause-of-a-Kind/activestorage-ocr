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

  # Variant tests

  def test_available_variants
    variants = ActiveStorage::Ocr::Binary.available_variants
    assert_includes variants, :ocrs
    assert_includes variants, :leptess
    assert_includes variants, :all
  end

  def test_variant_info_returns_correct_info
    info = ActiveStorage::Ocr::Binary.variant_info(:ocrs)
    assert_equal "", info[:suffix]
    assert info[:description].include?("Rust")

    info = ActiveStorage::Ocr::Binary.variant_info(:leptess)
    assert_equal "-leptess", info[:suffix]
    assert info[:description].include?("Tesseract")

    info = ActiveStorage::Ocr::Binary.variant_info(:all)
    assert_equal "-all", info[:suffix]
  end

  def test_variant_info_raises_on_invalid_variant
    assert_raises(ArgumentError) do
      ActiveStorage::Ocr::Binary.variant_info(:invalid)
    end
  end

  def test_download_url_default_variant
    url = ActiveStorage::Ocr::Binary.download_url
    # Default variant is ocrs, which has no suffix
    refute_includes url, "-leptess"
    refute_includes url, "-all"
  end

  def test_download_url_leptess_variant
    url = ActiveStorage::Ocr::Binary.download_url(variant: :leptess)
    assert_includes url, "-leptess"
  end

  def test_download_url_all_variant
    url = ActiveStorage::Ocr::Binary.download_url(variant: :all)
    assert_includes url, "-all"
  end

  def test_download_url_accepts_string_variant
    url = ActiveStorage::Ocr::Binary.download_url(variant: "leptess")
    assert_includes url, "-leptess"
  end

  def test_download_url_raises_on_invalid_variant
    assert_raises(ArgumentError) do
      ActiveStorage::Ocr::Binary.download_url(variant: :invalid)
    end
  end
end
