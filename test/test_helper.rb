# frozen_string_literal: true

$LOAD_PATH.unshift File.expand_path("../lib", __dir__)
require "activestorage-ocr"
require "minitest/autorun"
require "webmock/minitest"

class ActiveStorage::Ocr::TestCase < Minitest::Test
  def setup
    ActiveStorage::Ocr.reset_configuration!
    WebMock.reset!
  end
end
