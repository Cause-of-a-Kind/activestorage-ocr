# frozen_string_literal: true

require "rails/generators"

module ActivestorageOcr
  module Generators
    # Installs the OCR server binary and creates a binstub.
    #
    # This generator creates a bin/activestorage-ocr-server script that
    # automatically downloads the binary if needed and runs it.
    #
    # == Usage
    #
    #   rails generate activestorage_ocr:install
    #
    # == What it does
    #
    # 1. Creates bin/activestorage-ocr-server (wrapper script)
    # 2. Downloads binary to bin/dist/activestorage-ocr-server
    # 3. Adds bin/dist/ to .gitignore
    #
    class InstallGenerator < Rails::Generators::Base
      source_root File.expand_path("templates", __dir__)

      desc "Installs the OCR server binary and creates a binstub"

      def create_binstub
        template "bin/activestorage-ocr-server", "bin/activestorage-ocr-server"
        chmod "bin/activestorage-ocr-server", 0o755
      end

      def update_gitignore
        gitignore_path = Rails.root.join(".gitignore")
        return unless File.exist?(gitignore_path)

        gitignore_content = File.read(gitignore_path)
        return if gitignore_content.include?("/bin/dist")

        append_to_file ".gitignore", "\n# OCR server binary\n/bin/dist/\n"
      end

      def download_binary
        say "Downloading OCR server binary...", :green
        require "activestorage/ocr/binary"
        dist_dir = Rails.root.join("bin", "dist")
        ActiveStorage::Ocr::Binary.install!(path: dist_dir.to_s)
      end

      def show_instructions
        say ""
        say "OCR server installed successfully!", :green
        say ""
        say "Add to your Procfile:", :yellow
        say "  ocr: bin/activestorage-ocr-server --host 127.0.0.1 --port 9292"
        say ""
        say "Configure the server URL in config/initializers/activestorage_ocr.rb:", :yellow
        say "  ActiveStorage::Ocr.configure do |config|"
        say "    config.server_url = ENV.fetch('ACTIVESTORAGE_OCR_SERVER_URL', 'http://127.0.0.1:9292')"
        say "  end"
        say ""
      end
    end
  end
end
