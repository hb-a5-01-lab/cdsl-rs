# CDSL Build and Installation Makefile

.DEFAULT_GOAL := all

.PHONY: all check run config format fmt-check test clean rebuild \
        install uninstall help

# Project configuration
BINARY         := cdsl-rs
BUILD_BINARY   := target/release/$(BINARY)
EXAMPLE_CONFIG := example-config.xml

# Installation paths
INSTALL_DIR := $(HOME)/.local/bin
CONFIG_DIR  := $(HOME)/.config/cdsl
CONFIG_FILE := $(CONFIG_DIR)/config.xml

# Default target
all: check build config

# Display available commands
help:
	@echo "CDSL - Build and Installation"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all        Build release binary and setup configuration"
	@echo "  build      Build release binary"
	@echo "  run        Build and run CDSL"
	@echo "  config     Create default configuration if missing"
	@echo "  format     Format Rust source files"
	@echo "  fmt-check  Check Rust formatting without modifying files"
	@echo "  test       Run all tests"
	@echo "  clean      Remove build artifacts"
	@echo "  rebuild    Clean and rebuild the project"
	@echo "  install    Install binary to ~/.local/bin"
	@echo "  uninstall  Remove installed binary"
	@echo "  check      Verify required development tools"

# Verify required development tools
check:
	@command -v cargo >/dev/null 2>&1 || { \
		echo "Error: Cargo is not installed."; \
		echo "Install Rust: https://rustup.rs"; \
		exit 1; \
	}
	@command -v git >/dev/null 2>&1 || { \
		echo "Error: Git is not installed."; \
		echo "Install: sudo apt install git"; \
		exit 1; \
	}

# Build release binary
build: check
	@echo "Building CDSL..."
	@cargo build --release --bin $(BINARY)
	@echo "Build complete: $(BUILD_BINARY)"

# Build and run
run: build
	@echo ""
	@$(BUILD_BINARY)

# Set up default configuration
config:
	@echo "Setting up CDSL configuration..."
	@if [ ! -f "$(EXAMPLE_CONFIG)" ]; then \
		echo "Error: $(EXAMPLE_CONFIG) not found."; \
		exit 1; \
	fi
	@mkdir -p "$(CONFIG_DIR)"
	@if [ ! -f "$(CONFIG_FILE)" ]; then \
		cp "$(EXAMPLE_CONFIG)" "$(CONFIG_FILE)"; \
		echo "Created $(CONFIG_FILE)"; \
	else \
		echo "$(CONFIG_FILE) already exists; leaving it unchanged."; \
	fi

# Format Rust source files
format:
	@cargo fmt

# Check Rust formatting
fmt-check:
	@cargo fmt -- --check

# Run tests
test: check
	@cargo test

# Remove build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Clean and rebuild
rebuild: clean build

# Install binary
install: build
	@echo "Installing CDSL to $(INSTALL_DIR)..."
	@mkdir -p "$(INSTALL_DIR)"
	@install -m 755 "$(BUILD_BINARY)" "$(INSTALL_DIR)/$(BINARY)"
	@echo "Installed: $(INSTALL_DIR)/$(BINARY)"

# Uninstall binary
uninstall:
	@echo "Removing CDSL..."
	@rm -f "$(INSTALL_DIR)/$(BINARY)"
	@echo "Uninstalled CDSL."
