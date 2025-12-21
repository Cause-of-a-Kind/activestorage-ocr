# frozen_string_literal: true

require "fileutils"
require "net/http"
require "uri"
require "rubygems/package"
require "zlib"

module ActiveStorage
  module Ocr
    class Binary
      GITHUB_REPO = "Cause-of-a-Kind/activestorage-ocr"
      BINARY_NAME = "activestorage-ocr-server"

      class << self
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

        def binary_path
          @binary_path ||= File.join(install_dir, BINARY_NAME)
        end

        def install_dir
          @install_dir ||= begin
            dir = File.join(gem_root, "bin")
            FileUtils.mkdir_p(dir)
            dir
          end
        end

        def gem_root
          @gem_root ||= File.expand_path("../../../..", __FILE__)
        end

        def installed?
          File.executable?(binary_path)
        end

        def version
          ActiveStorage::Ocr::VERSION
        end

        def download_url
          tag = "v#{version}"
          filename = "activestorage-ocr-server-#{platform}.tar.gz"
          "https://github.com/#{GITHUB_REPO}/releases/download/#{tag}/#{filename}"
        end

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

        def extract_binary(tarball_data)
          # Extract from gzipped tarball
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
