# Contributing to activestorage-ocr

Thank you for your interest in contributing!

## Development Setup

### Prerequisites

- Ruby 3.2+
- Rust (latest stable)
- Bundler

### Getting Started

1. Fork and clone the repository

2. Install Ruby dependencies:
   ```bash
   bundle install
   ```

3. Build the Rust OCR server:
   ```bash
   cd rust && cargo build --release
   ```

4. Run tests:
   ```bash
   # Ruby unit tests (no server needed)
   bundle exec rake test

   # Rust tests
   cd rust && cargo test

   # Integration tests (requires server running)
   cd rust && ./target/release/activestorage-ocr-server &
   cd test/sandbox && RAILS_ENV=test bin/rails test
   ```

## Code Style

- **Ruby**: Follow standard Ruby style conventions
- **Rust**: Run `cargo fmt` before committing

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes with clear commit messages
3. Ensure all tests pass (both Ruby and Rust)
4. Update documentation if needed
5. Submit a PR with a clear description of the changes

## Reporting Issues

When reporting issues, please include:

- Ruby and Rust versions
- Operating system and architecture
- Steps to reproduce the issue
- Relevant error messages or logs
- Sample files if applicable (ensure they don't contain sensitive data)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
