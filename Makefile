.PHONY: dev release install check-assets clean setup build run help

# Default target - show help
all: help

help:
	@echo "Gomoku Build System - Cross-Platform Support"
	@echo ""
	@echo "Available targets:"
	@echo "  setup     - Setup dependencies for cross-platform builds"
	@echo "  build     - Build the project using cross-platform build script"
	@echo "  run       - Quick run with automatic dependency setup"
	@echo "  dev       - Development build with cargo (requires manual setup)"
	@echo "  release   - Release build with cargo (requires manual setup)"
	@echo "  install   - Install the application (not yet implemented)"
	@echo "  clean     - Clean cargo build artifacts"
	@echo ""
	@echo "For new users, try: make setup && make run"

setup:
	@echo "Setting up cross-platform dependencies..."
	./setup_deps.sh

build: check-assets
	@echo "Building with cross-platform script..."
	./build.sh

run: check-assets
	@echo "Running with cross-platform script..."
	./run.sh

check-assets:
	@if [ ! -f assets/backgrounds/ingame-background/*.png ] && [ -f assets/backgrounds/ingame-background/ingame-background.zip ]; then \
		cd assets/backgrounds/ingame-background && unzip -o ingame-background.zip; \
	fi
	@if [ ! -f assets/backgrounds/dolphin/*.png ] && [ -f assets/backgrounds/dolphin/dolphin.zip ]; then \
		cd assets/backgrounds/dolphin && unzip -o dolphin.zip; \
	fi

dev: check-assets
	cargo run

release: check-assets
	cargo build --release

install:
	@echo "Install target not yet implemented"

clean:
	cargo clean
	@echo "Cargo build artifacts cleaned"
