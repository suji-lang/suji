# Makefile for nnlang project

.PHONY: build release test verify_spec verify_examples clean lint help

# Default target
all: build

# Build the project
build:
	@echo "Building nnlang..."
	cargo build

# Build release version
release:
	@echo "Building nnlang release..."
	cargo build --release

# Run all tests
test:
	@echo "Running tests..."
	cargo test

# Verify spec tests using shell script
verify_spec: release
	@echo "Verifying spec tests..."
	./scripts/verify_spec.sh

# Verify example programs using shell script
verify_examples: release
	@echo "Verifying example programs..."
	./scripts/verify_examples.sh

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Run clippy linter and format check
lint:
	@echo "Running clippy linter..."
	cargo clippy --all-targets
	@echo "Checking code formatting..."
	cargo fmt --check

# Show help
help:
	@echo "Available targets:"
	@echo "  build       - Build the project"
	@echo "  release     - Build the project in release mode"
	@echo "  test        - Run all tests"
	@echo "  verify_spec - Run spec tests using shell script (requires release build)"
	@echo "  verify_examples - Run example programs to verify they work (requires release build)"
	@echo "  clean       - Clean build artifacts"
	@echo "  lint        - Run clippy linter and format check"
	@echo "  help        - Show this help message"
