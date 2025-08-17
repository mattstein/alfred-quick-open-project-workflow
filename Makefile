.PHONY: build clean install workflow package test help

# Default target
all: build

# Build the Rust binary (local development)
build:
	cargo build --release
	cp target/release/search ./search

# Build universal binary for both Intel and Apple Silicon (requires rustup)
build-universal:
	rustup target add aarch64-apple-darwin || true
	rustup target add x86_64-apple-darwin || true
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	lipo -create -output search \
		target/aarch64-apple-darwin/release/search \
		target/x86_64-apple-darwin/release/search

# Clean build artifacts
clean:
	cargo clean
	rm -f search
	rm -f *.alfredworkflow

# Run tests
test:
	cargo test

# Install dependencies (for CI)
install:
	# Rust dependencies are handled by Cargo
	# Add targets for universal binary support
	rustup target add aarch64-apple-darwin
	rustup target add x86_64-apple-darwin

# Create Alfred workflow package
workflow: build prepare-plist
	@echo "Creating Alfred workflow package..."
	zip -r quick-open-project.alfredworkflow \
		search \
		info.plist.tmp \
		icon.png \
		warning.png \
		README.md \
		LICENSE.md \
		CHANGELOG.md
	# Rename to correct info.plist in the zip
	@python3 fix_workflow_zip.py
	rm -f info.plist.tmp
	@echo "✅ Created quick-open-project.alfredworkflow"

# Prepare info.plist with version
prepare-plist:
	$(eval VERSION := $(shell grep '^version' Cargo.toml | cut -d'"' -f2))
	@echo "Setting workflow version to $(VERSION)"
	# Create updated plist
	@python3 update_plist.py $(VERSION)

# Create workflow package for release (current architecture)
package: clean build workflow
	@echo "🎉 Package ready for release!"

# Create universal workflow package for release (both Intel and Apple Silicon)
package-universal: clean build-universal workflow
	@echo "🎉 Universal package ready for release!"

# Show help
help:
	@echo "Available targets:"
	@echo "  build              - Build the Rust binary (current architecture)"
	@echo "  build-universal    - Build universal binary (Intel + Apple Silicon)"
	@echo "  test               - Run all tests"
	@echo "  clean              - Clean build artifacts"
	@echo "  workflow           - Create .alfredworkflow package (auto-sets version)"
	@echo "  package            - Clean build and create workflow package"
	@echo "  package-universal  - Clean build universal and create workflow package"
	@echo "  help               - Show this help message"