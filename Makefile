.PHONY: help build test run clean fmt clippy doc install release check-all

help:
	@echo "Available commands:"
	@echo "  build       - Build the project in debug mode"
	@echo "  test        - Run all tests"
	@echo "  run         - Run the application with --help"
	@echo "  clean       - Clean build artifacts"
	@echo "  fmt         - Format code using cargo fmt"
	@echo "  clippy      - Run clippy linter"
	@echo "  doc         - Generate documentation"
	@echo "  install     - Install the binary locally"
	@echo "  release     - Build optimized release binary"
	@echo "  check-all   - Run fmt, clippy, test, and doc"
	@echo "  help        - Show this help message"

build:
	cargo build

test:
	cargo test

test-specific TEST_NAME:
	cargo test $(TEST_NAME)

run:
	cargo fmt
	cargo run $(args)

clean:
	cargo clean

fmt:
	cargo fmt

clippy:
	cargo clippy

doc:
	cargo doc --open --no-deps

publish:
	cargo fmt
	cargo test
	cargo publish
