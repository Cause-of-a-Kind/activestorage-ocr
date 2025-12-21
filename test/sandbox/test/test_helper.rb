# frozen_string_literal: true

ENV["RAILS_ENV"] = "test"
require_relative "../config/environment"
require "rails/test_help"
require "minitest/autorun"

class ActiveSupport::TestCase
  fixtures :all

  def fixture_file(filename)
    File.join(File.dirname(__FILE__), "fixtures/files", filename)
  end
end
