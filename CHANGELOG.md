# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-12-21

### Added

- Initial release
- Rust OCR server using pure-Rust `ocrs` library (no system dependencies)
- Ruby gem with HTTP client for Rails Active Storage integration
- Support for PNG, JPEG, TIFF, WebP, and PDF formats
- Automatic text extraction from PDFs (both embedded text and scanned/image PDFs)
- GitHub Actions for CI and cross-platform binary builds (Linux/macOS, x86_64/ARM64)
- Sandbox Rails app for integration testing
- Rake tasks for server management (`activestorage_ocr:install`, `activestorage_ocr:start`, `activestorage_ocr:health`)
