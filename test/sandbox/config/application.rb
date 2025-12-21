# frozen_string_literal: true

require_relative "boot"
require "rails"
require "active_model/railtie"
require "active_record/railtie"
require "active_storage/engine"
require "action_controller/railtie"
require "active_job/railtie"

Bundler.require(*Rails.groups)
require "activestorage-ocr"

module Sandbox
  class Application < Rails::Application
    config.load_defaults 7.1
    config.eager_load = false
    config.active_storage.service = :local
  end
end
