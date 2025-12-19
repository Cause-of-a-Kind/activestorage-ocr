# activestorage-ocr

OCR for Rails Active Storage attachments, powered by Rust and [ocrs](https://github.com/robertknight/ocrs).

## Overview

`activestorage-ocr` provides optical character recognition (OCR) for files stored with Active Storage. It uses a high-performance Rust server with the pure-Rust `ocrs` OCR engine, eliminating the need for third-party OCR services or system-level dependencies.

**Key Features:**
- **Pure Rust** - No Tesseract or system dependencies required
- **Self-contained** - Models download automatically on first run (~50MB)
- **Fast** - Processes images in ~150ms
- **HTTP/JSON API** - Easy to debug and integrate

**Architecture:** Separate process with HTTP/JSON communication (inspired by AnyCable)
- **Rust server** handles CPU-intensive OCR processing
- **Ruby gem** provides seamless Rails integration
- Simple HTTP/JSON protocol for easy debugging

## Features

- `has_ocr_attachment :document` - declarative model macro
- Automatic OCR on upload (async via Active Job)
- Manual/on-demand OCR processing
- Self-contained Tesseract (no system dependencies)
- Multiple language support
- Extension hooks for LLM post-processing

## Installation

Add to your Gemfile:

```ruby
gem "activestorage-ocr"
```

Run the installer:

```bash
rails generate activestorage_ocr:install
rails db:migrate
```

## Usage

```ruby
class Document < ApplicationRecord
  has_ocr_attachment :file
end

# Create with file - OCR runs automatically in background
doc = Document.create!(file: uploaded_file)

# Check OCR status
doc.file_ocr_completed?           # => true/false
doc.file_ocr_result.full_text     # => "Extracted text..."
doc.file_ocr_result.confidence    # => 0.95

# Manual OCR
doc.perform_ocr_on_file!          # Sync
doc.perform_ocr_on_file_later     # Async
```

## Configuration

```ruby
# config/initializers/activestorage_ocr.rb
ActiveStorageOcr.configure do |config|
  config.server_host = "127.0.0.1"
  config.server_port = 9292
  config.default_languages = ["eng"]
  config.queue_name = :ocr
end
```

## Development

### Building the Rust server

```bash
cd rust
cargo build --release
```

### Running tests

```bash
# Rust tests
cd rust && cargo test

# Ruby tests
cd ruby && bundle exec rspec
```

## License

MIT License - see LICENSE file.
