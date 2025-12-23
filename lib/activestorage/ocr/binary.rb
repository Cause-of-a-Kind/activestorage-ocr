# frozen_string_literal: true

require "fileutils"
require "net/http"
require "uri"
require "rubygems/package"
require "zlib"

module ActiveStorage
  module Ocr
    # Manages the OCR server binary.
    #
    # Handles downloading pre-built binaries from GitHub releases and
    # detecting the appropriate platform.
    #
    # == Supported Platforms
    #
    # * darwin-x86_64 (macOS Intel)
    # * darwin-aarch64 (macOS Apple Silicon)
    # * linux-x86_64 (Linux x86_64)
    # * linux-aarch64 (Linux ARM64)
    #
    # == Usage
    #
    #   # Install the binary for the current platform
    #   ActiveStorage::Ocr::Binary.install!
    #
    #   # Check if binary is installed
    #   ActiveStorage::Ocr::Binary.installed?  # => true
    #
    #   # Get the path to the binary
    #   ActiveStorage::Ocr::Binary.binary_path
    #
    class Binary
      # GitHub repository for downloading releases.
      GITHUB_REPO = "Cause-of-a-Kind/activestorage-ocr"

      # Name of the server binary.
      BINARY_NAME = "activestorage-ocr-server"

      class << self
        # Detects the current platform.
        #
        # ==== Returns
        #
        # A String like "darwin-x86_64" or "linux-aarch64".
        #
        # ==== Raises
        #
        # RuntimeError if the OS or architecture is unsupported.
        def platform
          os = case RbConfig::CONFIG["host_os"]
               when /darwin/i then "darwin"
               when /linux/i then "linux"
               else
                 raise "Unsupported OS: #{RbConfig::CONFIG['host_os']}"
               end

          arch = case RbConfig::CONFIG["host_cpu"]
                 when /x86_64|amd64/i then "x86_64"
                 when /arm64|aarch64/i then "aarch64"
                 else
                   raise "Unsupported architecture: #{RbConfig::CONFIG['host_cpu']}"
                 end

          "#{os}-#{arch}"
        end

        # Returns the path where the binary is installed.
        #
        # ==== Returns
        #
        # Absolute path to the binary.
        def binary_path
          @binary_path ||= File.join(install_dir, BINARY_NAME)
        end

        # Returns the installation directory.
        #
        # Creates the directory if it doesn't exist.
        def install_dir
          @install_dir ||= begin
            dir = File.join(gem_root, "bin")
            FileUtils.mkdir_p(dir)
            dir
          end
        end

        # Returns the gem's root directory.
        def gem_root
          @gem_root ||= File.expand_path("../../../..", __FILE__)
        end

        # Checks if the binary is installed and executable.
        #
        # ==== Returns
        #
        # +true+ if the binary exists and is executable.
        def installed?
          File.executable?(binary_path)
        end

        # Returns the gem version.
        #
        # Used to determine which release to download.
        def version
          ActiveStorage::Ocr::VERSION
        end

        # Returns the download URL for the current platform.
        #
        # ==== Returns
        #
        # GitHub releases URL for the platform-specific tarball.
        def download_url
          tag = "v#{version}"
          filename = "activestorage-ocr-server-#{platform}.tar.gz"
          "https://github.com/#{GITHUB_REPO}/releases/download/#{tag}/#{filename}"
        end

        # Downloads and installs the binary.
        #
        # Downloads from GitHub releases and extracts to the gem's bin directory.
        #
        # ==== Parameters
        #
        # * +force+ - If true, reinstalls even if already installed
        #
        # ==== Returns
        #
        # Path to the installed binary.
        #
        # ==== Raises
        #
        # RuntimeError if the download fails.
        def install!(force: false)
          return binary_path if installed? && !force

          puts "Downloading activestorage-ocr-server for #{platform}..."

          uri = URI(download_url)
          response = fetch_with_redirects(uri)

          unless response.is_a?(Net::HTTPSuccess)
            raise "Failed to download binary: #{response.code} #{response.message}\n" \
                  "URL: #{download_url}\n" \
                  "You may need to build from source: cd rust && cargo build --release"
          end

          extract_binary(response.body)
          puts "Installed to #{binary_path}"
          binary_path
        end

        private

        # Fetches a URL, following redirects.
        def fetch_with_redirects(uri, limit = 10)
          raise "Too many redirects" if limit == 0

          http = Net::HTTP.new(uri.host, uri.port)
          http.use_ssl = uri.scheme == "https"

          request = Net::HTTP::Get.new(uri)
          response = http.request(request)

          case response
          when Net::HTTPRedirection
            location = URI(response["location"])
            fetch_with_redirects(location, limit - 1)
          else
            response
          end
        end

        # Extracts the binary from a gzipped tarball.
        def extract_binary(tarball_data)
          gz = Zlib::GzipReader.new(StringIO.new(tarball_data))
          tar = Gem::Package::TarReader.new(gz)

          tar.each do |entry|
            next unless entry.file? && entry.full_name == BINARY_NAME

            File.open(binary_path, "wb") do |f|
              f.write(entry.read)
            end
            File.chmod(0o755, binary_path)
            return
          end

          raise "Binary not found in tarball"
        ensure
          gz&.close
        end
      end
    end
  end
end
