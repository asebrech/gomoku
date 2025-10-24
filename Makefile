.PHONY: all dev release clean fclean re run test setup help

# 42 School Required Rules
# Default target - Build the project (42 standard)
all: release
	@echo "✅ Default build complete! (release version)"
	@echo "Run with: cargo run --release"

# Clean object files and cargo artifacts (42 standard)
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean completed!"

# Full clean (42 standard)
fclean: clean
	@echo "Cleaning distribution directory..."
	rm -rf dist
	@echo "✅ Full clean completed!"

# Rebuild everything from scratch (42 standard)
re: fclean all
	@echo "✅ Complete rebuild finished!"

# Help target
help:
	@echo "Gomoku Build System - Simple Cargo Commands"
	@echo ""
	@echo "🚀 First time setup:"
	@echo "  ./setup.sh        - Install all dependencies (GStreamer, etc.)"
	@echo ""
	@echo "Main targets:"
	@echo "  make / make all   - Build release version"
	@echo "  make dev          - Build development version"
	@echo "  make run          - Build and run release version"
	@echo "  make test         - Run all tests"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make fclean       - Full clean (artifacts + dist)"
	@echo "  make re           - Rebuild everything from scratch"
	@echo ""
	@echo "Direct Cargo commands:"
	@echo "  cargo run                    - Build and run (debug)"
	@echo "  cargo run --release          - Build and run (optimized)"
	@echo "  cargo build                  - Build debug version"
	@echo "  cargo build --release        - Build release version"
	@echo "  cargo test                   - Run tests"
	@echo ""
	@echo "💡 If you get GStreamer errors, run: ./setup.sh"

# Main targets - Simple and clear

# Build release version
release:
	@echo "Building release version..."
	cargo build --release
	@echo "✅ Release build complete!"

# Build development version
dev:
	@echo "Building development version..."
	cargo build
	@echo "✅ Development build complete!"

# Build and run release version
run: release
	@echo "Running Gomoku (release version)..."
	cargo run --release

# Run all tests
test:
	@echo "Running all tests..."
	cargo test
	@echo "✅ All tests completed!"

# Setup development environment
setup:
	@echo "Setting up development environment..."
	./setup.sh
	@echo "✅ Setup completed!"
