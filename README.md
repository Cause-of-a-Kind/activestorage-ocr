# activestorage-ocr

[![CI](https://github.com/Cause-of-a-Kind/activestorage-ocr/actions/workflows/ci.yml/badge.svg)](https://github.com/Cause-of-a-Kind/activestorage-ocr/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

OCR for Rails Active Storage attachments, powered by Rust.

## Overview

`activestorage-ocr` provides optical character recognition (OCR) for files stored with Active Storage. It uses a high-performance Rust server with your choice of OCR engine, eliminating the need for third-party OCR services.

**Key Features:**
- **Two OCR Engines** - Choose the right tool for the job:
  - **ocrs** (default) - Pure Rust, no dependencies, fast (~150ms)
  - **leptess** - Tesseract-based, better for noisy/messy images like phone photos
- **Self-contained** - Pre-built binaries with no system dependencies
- **Per-request engine selection** - Use different engines for different files
- **Automatic** - OCR runs automatically when files are uploaded via Active Storage

**Supported Formats:**
- Images: PNG, JPEG, TIFF, WebP, GIF, BMP
- Documents: PDF (both embedded text and scanned/image PDFs)

**Architecture:** Separate process with HTTP/JSON communication (inspired by AnyCable)
- **Rust server** handles CPU-intensive OCR processing
- **Ruby gem** provides seamless Rails integration
- Simple HTTP/JSON protocol for easy debugging

### Choosing an Engine

| Engine | Best For | Speed | Accuracy on Clean Images | Accuracy on Messy Images |
|--------|----------|-------|--------------------------|--------------------------|
| `ocrs` | Scanned documents, clean images | Fast (~150ms) | Excellent | Good |
| `leptess` | Phone photos, noisy images, skewed text | Moderate (~300ms) | Excellent | Excellent |

## Requirements

- Ruby 3.2+
- Rails 7.0+ with Active Storage
- Rust (for building from source) or pre-built binaries from releases

## Installation

**1. Add to your Gemfile:**

```ruby
gem "activestorage-ocr"
```

**2. Install the gem and OCR server binary:**

```bash
bundle install
bin/rails activestorage_ocr:install
```

**3. Add the OCR server to your `Procfile.dev`:**

```procfile
web: bin/rails server
ocr: activestorage-ocr-server --host 127.0.0.1 --port 9292
```

Now when you run `bin/dev`, the OCR server starts automatically alongside Rails.

> **Note:** If you don't have a `Procfile.dev`, create one. Rails 7+ apps typically use `bin/dev` with [foreman](https://github.com/ddollar/foreman) or [overmind](https://github.com/DarthSim/overmind) to manage multiple processes.

## Usage

Once installed, OCR happens **automatically** when you upload images or PDFs through Active Storage. The extracted text is stored in the blob's metadata.

### Accessing OCR Results

```ruby
# After uploading a file
document.file.analyze  # Triggers OCR (usually happens automatically)

# Access the results from metadata
document.file.metadata["ocr_text"]         # => "Extracted text..."
document.file.metadata["ocr_confidence"]   # => 0.85
document.file.metadata["ocr_processed_at"] # => "2024-12-24T12:00:00Z"
```

### Helper Methods (Optional)

Add convenience methods to your model:

```ruby
class Document < ApplicationRecord
  has_one_attached :file

  def ocr_text
    file.metadata["ocr_text"]
  end

  def ocr_confidence
    file.metadata["ocr_confidence"]
  end

  def ocr_processed?
    file.metadata["ocr_processed_at"].present?
  end
end
```

### Using the Client Directly

You can also use the client directly for more control:

```ruby
client = ActiveStorage::Ocr::Client.new

# Check server health
client.healthy?  # => true

# Extract text from a file path
result = client.extract_text_from_path("/path/to/image.png")
result.text        # => "Extracted text..."
result.confidence  # => 0.95

# Extract text from an Active Storage attachment
result = client.extract_text(document.file)
```

## Configuration

```ruby
# config/initializers/activestorage_ocr.rb
ActiveStorage::Ocr.configure do |config|
  config.server_url = ENV.fetch("ACTIVESTORAGE_OCR_SERVER_URL", "http://127.0.0.1:9292")
  config.timeout = 60        # Request timeout in seconds
  config.open_timeout = 10   # Connection timeout in seconds
end
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ACTIVESTORAGE_OCR_SERVER_URL` | `http://127.0.0.1:9292` | Full URL to the OCR server |

## Production Deployment

For production deployments, the OCR server binary needs to be installed in your app's `bin/` directory (not the gem directory) so it can be referenced from your Procfile.

### Setup for Production

**1. Run the install generator:**

```bash
rails generate activestorage_ocr:install
```

This creates:
- `bin/activestorage-ocr-server` - A wrapper script that runs the OCR server
- `bin/dist/activestorage-ocr-server` - The actual binary (gitignored)

**2. Add to your Procfile:**

```procfile
web: bundle exec puma -C config/puma.rb
ocr: bin/activestorage-ocr-server --host 127.0.0.1 --port 9292
```

### Docker Deployment

In your `Dockerfile`, run the generator during the build to install the binary:

```dockerfile
# Install gems
RUN bundle install

# Install OCR server binary to bin/dist/
RUN bundle exec rails activestorage_ocr:install path=./bin/dist

# Alternatively, run the full generator:
# RUN bundle exec rails generate activestorage_ocr:install
```

Use foreman to manage both processes:

```dockerfile
# Procfile
CMD ["bundle", "exec", "foreman", "start"]
```

### Fly.io Deployment

**fly.toml configuration:**

```toml
app = "your-app-name"
primary_region = "iad"

[deploy]
  # Note: Don't use release_command for SQLite with volumes
  # Migrations run in docker-entrypoint instead

[env]
  RAILS_ENV = "production"
  ACTIVESTORAGE_OCR_SERVER_URL = "http://127.0.0.1:9292"

[http_service]
  internal_port = 8080
  force_https = true

[[mounts]]
  source = "data"
  destination = "/rails/storage"

[[vm]]
  memory = "1024mb"
  cpu_kind = "shared"
  cpus = 2
```

**Procfile for Fly.io:**

```procfile
web: bundle exec puma -C config/puma.rb
ocr: bin/activestorage-ocr-server --host 127.0.0.1 --port 9292
```

**Important notes for Fly.io:**
- Use `foreman` as the entrypoint to run both processes
- The OCR server binds to `127.0.0.1` (internal only)
- Set `ACTIVESTORAGE_OCR_SERVER_URL` env var to `http://127.0.0.1:9292`
- For SQLite with volumes, run migrations in `docker-entrypoint` not `release_command`

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ACTIVESTORAGE_OCR_SERVER_URL` | `http://127.0.0.1:9292` | URL where the OCR server is running |
| `ACTIVESTORAGE_OCR_TIMEOUT` | `30` | Request timeout in seconds |
| `ACTIVESTORAGE_OCR_OPEN_TIMEOUT` | `5` | Connection timeout in seconds |

### Troubleshooting

**Binary not found:**
```
Error: bin/activestorage-ocr-server: No such file or directory
```
Solution: Run `rails generate activestorage_ocr:install` or `rails activestorage_ocr:install path=./bin/dist`

**Connection refused:**
```
Faraday::ConnectionFailed: Connection refused
```
Solution: Ensure the OCR server is running and `ACTIVESTORAGE_OCR_SERVER_URL` is correctly configured.

**Timeout errors:**
```
Faraday::TimeoutError
```
Solution: Increase timeout values in the initializer or reduce image/PDF sizes.

**Health check:**
```bash
# Verify the OCR server is responding
curl http://localhost:9292/health

# Or via rake task
bin/rails activestorage_ocr:health
```

## Rake Tasks

```bash
# Install the OCR server binary for your platform
bin/rails activestorage_ocr:install

# Start the OCR server (for manual testing)
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
