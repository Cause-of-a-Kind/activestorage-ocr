# activestorage-ocr

[![CI](https://github.com/Cause-of-a-Kind/activestorage-ocr/actions/workflows/ci.yml/badge.svg)](https://github.com/Cause-of-a-Kind/activestorage-ocr/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

OCR for Rails Active Storage attachments, powered by Rust and [ocrs](https://github.com/robertknight/ocrs).

## Overview

`activestorage-ocr` provides optical character recognition (OCR) for files stored with Active Storage. It uses a high-performance Rust server with the pure-Rust `ocrs` OCR engine, eliminating the need for third-party OCR services or system-level dependencies.

**Key Features:**
- **Pure Rust** - No Tesseract or system dependencies required
- **Self-contained** - Models download automatically on first run (~50MB)
- **Fast** - Processes images in ~150ms
- **HTTP/JSON API** - Easy to debug and integrate

**Supported Formats:**
- Images: PNG, JPEG, TIFF, WebP, GIF, BMP
- Documents: PDF (both embedded text and scanned/image PDFs)

**Architecture:** Separate process with HTTP/JSON communication (inspired by AnyCable)
- **Rust server** handles CPU-intensive OCR processing
- **Ruby gem** provides seamless Rails integration
- Simple HTTP/JSON protocol for easy debugging

## Requirements

- Ruby 3.2+
- Rails 7.0+ with Active Storage
- Rust (for building from source) or pre-built binaries from releases

## Installation

Add to your Gemfile:

```ruby
gem "activestorage-ocr"
```

Then install the OCR server binary:

```bash
bundle install
bin/rails activestorage_ocr:install
```

## Quick Start

1. **Start the OCR server:**

   ```bash
   bin/rails activestorage_ocr:start
   ```

2. **Use the client in your Rails app:**

   ```ruby
   # In rails console or your code
   client = ActiveStorage::Ocr::Client.new

   # Check server health
   client.healthy?  # => true

   # Extract text from a file
   result = client.extract_text_from_path("/path/to/image.png", content_type: "image/png")
   result.text        # => "Extracted text..."
   result.confidence  # => 0.95

   # Extract text from an Active Storage attachment
   result = client.extract_text(document.file)
   ```

## Configuration

```ruby
# config/initializers/activestorage_ocr.rb
ActiveStorage::Ocr.configure do |config|
  config.server_host = "127.0.0.1"
  config.server_port = 9292
  config.timeout = 60
end
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ACTIVESTORAGE_OCR_HOST` | `127.0.0.1` | Server host |
| `ACTIVESTORAGE_OCR_PORT` | `9292` | Server port |

## Rake Tasks

```bash
# Install the OCR server binary for your platform
bin/rails activestorage_ocr:install

# Start the OCR server
bin/rails activestorage_ocr:start

# Check server health
bin/rails activestorage_ocr:health

# Show binary info (platform, path, version)
bin/rails activestorage_ocr:info
```

## API Endpoints

The Rust server exposes these HTTP endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/info` | GET | Server info and supported formats |
| `/ocr` | POST | Extract text from uploaded file |

### Example with curl

```bash
# Health check
curl http://localhost:9292/health

# OCR an image
curl -X POST http://localhost:9292/ocr \
  -F "file=@document.png;type=image/png"
```

## Development

### Building from source

```bash
# Build the Rust server
cd rust
cargo build --release

# The binary will be at rust/target/release/activestorage-ocr-server
```

### Running tests

```bash
# Ruby unit tests
bundle exec rake test

# Rust tests
cd rust && cargo test

# Integration tests (requires server running)
cd rust && ./target/release/activestorage-ocr-server &
cd test/sandbox && RAILS_ENV=test bin/rails test
```

### Code style

```bash
# Format Rust code
cd rust && cargo fmt

# Check for Rust warnings
cd rust && cargo clippy
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT License - see [LICENSE](LICENSE) file.
