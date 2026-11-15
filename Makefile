# CDSL Build and Installation Makefile

.PHONY: all clean rebuild check config install uninstall test format

# Installation and configuration paths
INSTALL_DIR    := $(HOME)/.local/bin
CONFIG_DIR     := $(HOME)/.config/cdsl
CONFIG_FILE    := $(CONFIG_DIR)/config.xml
EXAMPLE_CONFIG := example-config.xml
BINARY         := cdsl-rs
BUILD_BINARY   := target/release/$(BINARY)

# Default target: build the release binary and ensure the configuration exists
all: $(BINARY) config

# Verify that required development tools (cargo, git) are available
check:
	@command -v cargo >/dev/null 2>&1 || { \
		echo "Error: Cargo is not installed."; \
		echo "Install Rust with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; \
		exit 1; \
	}

	@command -v git >/dev/null 2>&1 || { \
		echo "Error: Git is not installed."; \
		echo "Install it with: sudo apt update && sudo apt install git"; \
		exit 1; \
	}

# Build the release binary using Cargo
$(BINARY): check
	@echo "Building CDSL..."
	cargo build --release --bin $(BINARY)
	cp $(BUILD_BINARY) $(BINARY)
	@echo "Build complete: $(BINARY)"

# Set up the default configuration file if not already present
config:
	@echo "Setting up CDSL configuration..."
	@mkdir -p $(CONFIG_DIR)
	@if [ ! -f $(CONFIG_FILE) ]; then \
		cp $(EXAMPLE_CONFIG) $(CONFIG_FILE); \
		echo "Created $(CONFIG_FILE)"; \
	else \
		echo "$(CONFIG_FILE) already exists; leaving it unchanged."; \
	fi

# Format all Rust source files
format:
	@echo "Formatting Rust files..."
	cargo fmt

# Run all unit and integration tests
test:
	@echo "Running tests..."
	cargo test

# Remove build artifacts and compiled binaries
clean:
	@echo "Cleaning build files..."
	cargo clean
	rm -f $(BINARY)

# Rebuild the project from scratch
rebuild: clean
	$(MAKE)

# Install the binary to the local bin directory (~/.local/bin)
install: $(BINARY)
	@echo "Installing CDSL to $(INSTALL_DIR)..."
	@mkdir -p $(INSTALL_DIR)
	@install -m 755 $(BINARY) $(INSTALL_DIR)/$(BINARY)
	@echo "Installed $(INSTALL_DIR)/$(BINARY)"

# Remove the installed binary from the local bin directory
uninstall:
	@echo "Removing CDSL from $(INSTALL_DIR)..."
	@rm -f $(INSTALL_DIR)/$(BINARY)
