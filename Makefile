.PHONY: build-rust build-ruby test-rust test-ruby test clean

# Default target
all: build-rust

# Build Rust server
build-rust:
	cd rust && cargo build --release

# Build Rust in debug mode
build-rust-debug:
	cd rust && cargo build

# Run Rust tests
test-rust:
	cd rust && cargo test

# Run Ruby tests
test-ruby:
	cd ruby && bundle exec rspec

# Run all tests
test: test-rust test-ruby

# Start the OCR server (development)
server:
	cd rust && cargo run -- --host 127.0.0.1 --port 9292

# Clean build artifacts
clean:
	cd rust && cargo clean

# Format Rust code
fmt:
	cd rust && cargo fmt

# Lint Rust code
lint:
	cd rust && cargo clippy

# Cross-compile for all platforms
cross-compile:
	cd rust && cross build --release --target x86_64-unknown-linux-gnu
	cd rust && cross build --release --target aarch64-unknown-linux-gnu
	cd rust && cross build --release --target x86_64-apple-darwin
	cd rust && cross build --release --target aarch64-apple-darwin
