.PHONY: build release install uninstall clean fmt check test help

# Default install location
PREFIX ?= $(HOME)/.local
BINARY_NAME := sk1llz

help:
	@echo "sk1llz CLI - Build & Install"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build     Build debug binary"
	@echo "  release   Build optimized release binary"
	@echo "  install   Build and install to $(PREFIX)/bin"
	@echo "  uninstall Remove from $(PREFIX)/bin"
	@echo "  clean     Remove build artifacts"
	@echo "  fmt       Format code"
	@echo "  check     Run cargo check"
	@echo "  test      Run tests"
	@echo "  clippy    Run clippy lints"
	@echo ""
	@echo "Options:"
	@echo "  PREFIX=path  Install prefix (default: ~/.local)"

build:
	cargo build

release:
	cargo build --release

install: release
	@mkdir -p $(PREFIX)/bin
	cp target/release/$(BINARY_NAME) $(PREFIX)/bin/
	@echo "Installed $(BINARY_NAME) to $(PREFIX)/bin/"
	@echo ""
	@echo "Make sure $(PREFIX)/bin is in your PATH:"
	@echo '  export PATH="$(PREFIX)/bin:$$PATH"'

uninstall:
	rm -f $(PREFIX)/bin/$(BINARY_NAME)
	@echo "Removed $(BINARY_NAME) from $(PREFIX)/bin/"

clean:
	cargo clean

fmt:
	cargo fmt

check:
	cargo check

test:
	cargo test

clippy:
	cargo clippy -- -D warnings
