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
    # == Binary Variants
    #
    # * :ocrs - Pure Rust OCR engine only (~15MB, no system dependencies)
    # * :leptess - Tesseract OCR engine only (~50-80MB, no system dependencies)
    # * :all - Both engines included (~80-100MB)
    #
    # == Usage
    #
    #   # Install the default (ocrs) binary for the current platform
    #   ActiveStorage::Ocr::Binary.install!
    #
    #   # Install the leptess variant
    #   ActiveStorage::Ocr::Binary.install!(variant: :leptess)
    #
    #   # Install all-engines variant
    #   ActiveStorage::Ocr::Binary.install!(variant: :all)
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

      # Available binary variants with their download suffix and descriptions.
      VARIANTS = {
        ocrs: {
          suffix: "",
          description: "Pure Rust OCR engine (fast, no system dependencies)"
        },
        leptess: {
          suffix: "-leptess",
          description: "Tesseract OCR engine (better for messy images)"
        },
        all: {
          suffix: "-all",
          description: "All OCR engines included"
        }
      }.freeze

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

        # Returns the download URL for the current platform and variant.
        #
        # ==== Parameters
        #
        # * +variant+ - The binary variant (:ocrs, :leptess, or :all)
        #
        # ==== Returns
        #
        # GitHub releases URL for the platform-specific tarball.
        def download_url(variant: :ocrs)
          variant = variant.to_sym
          validate_variant!(variant)
          tag = "v#{version}"
          suffix = VARIANTS[variant][:suffix]
          filename = "activestorage-ocr-server#{suffix}-#{platform}.tar.gz"
          "https://github.com/#{GITHUB_REPO}/releases/download/#{tag}/#{filename}"
        end

        # Lists available binary variants.
        #
        # ==== Returns
        #
        # Array of variant names.
        def available_variants
          VARIANTS.keys
        end

        # Returns info about a specific variant.
        #
        # ==== Parameters
        #
        # * +variant+ - The variant name (:ocrs, :leptess, or :all)
        #
        # ==== Returns
        #
        # Hash with :suffix and :description keys.
        def variant_info(variant)
          validate_variant!(variant)
          VARIANTS[variant]
        end

        # Downloads and installs the binary.
        #
        # Downloads from GitHub releases and extracts to the specified directory.
        #
        # ==== Parameters
        #
        # * +force+ - If true, reinstalls even if already installed
        # * +path+ - Custom installation directory (defaults to gem's bin directory)
        # * +variant+ - The binary variant to install (:ocrs, :leptess, or :all)
        #
        # ==== Returns
        #
        # Path to the installed binary.
        #
        # ==== Raises
        #
        # RuntimeError if the download fails.
        # ArgumentError if an invalid variant is specified.
        def install!(force: false, path: nil, variant: :ocrs)
          validate_variant!(variant)
          target_dir = path || install_dir
          target_path = File.join(target_dir, BINARY_NAME)

          if !force && File.executable?(target_path)
            puts "Binary already installed at #{target_path}"
            return target_path
          end

          FileUtils.mkdir_p(target_dir)

          variant_desc = VARIANTS[variant][:description]
          puts "Downloading activestorage-ocr-server (#{variant_desc}) for #{platform}..."

          url = download_url(variant: variant)
          uri = URI(url)
          response = fetch_with_redirects(uri)

          unless response.is_a?(Net::HTTPSuccess)
            feature_flag = variant == :ocrs ? "engine-ocrs" : (variant == :leptess ? "engine-leptess" : "all-engines")
            raise "Failed to download binary: #{response.code} #{response.message}\n" \
                  "URL: #{url}\n" \
                  "You may need to build from source: cd rust && cargo build --release --features #{feature_flag}"
          end

          extract_binary(response.body, target_path)
          puts "Installed to #{target_path}"
          target_path
        end

        private

        # Validates the variant parameter.
        #
        # ==== Raises
        #
        # ArgumentError if the variant is not valid.
        def validate_variant!(variant)
          variant = variant.to_sym
          unless VARIANTS.key?(variant)
            raise ArgumentError, "Invalid variant: #{variant}. Valid variants: #{VARIANTS.keys.join(', ')}"
          end
        end

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
        def extract_binary(tarball_data, target_path)
          gz = Zlib::GzipReader.new(StringIO.new(tarball_data))
          tar = Gem::Package::TarReader.new(gz)

          tar.each do |entry|
            next unless entry.file? && entry.full_name == BINARY_NAME

            File.open(target_path, "wb") do |f|
              f.write(entry.read)
            end
            File.chmod(0o755, target_path)
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
