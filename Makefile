.PHONY: dev release install check-assets clean setup build run deploy help all-build fix-libs cargo-clean cargo-release cargo-run-release cargo-run

# Default target - show help
all: help

# Build everything from scratch
all-build: clean setup build release deploy
	@echo "===========================================" 
	@echo "🎉 COMPLETE BUILD PIPELINE FINISHED! 🎉"
	@echo "==========================================="
	@echo ""
	@echo "✅ Standalone executable: target/release/run.sh"
	@echo "✅ Distribution package: dist/gomoku/"
	@echo ""
	@echo "Test with:"
	@echo "  cd target/release && ./run.sh"
	@echo ""
	@echo "Or distribute:"
	@echo "  cd dist/gomoku && ./run.sh"

help:
	@echo "Gomoku Build System - Cross-Platform Support"
	@echo ""
	@echo "Available targets:"
	@echo "  all-build     - Complete build pipeline (clean -> setup -> build -> release -> deploy)"
	@echo "  setup         - Setup dependencies for cross-platform builds"
	@echo "  fix-libs      - Fix pkg-config library naming issues"
	@echo "  build         - Build the project using cross-platform build script"
	@echo "  run           - Quick run with automatic dependency setup"
	@echo "  release       - Build standalone executable with bundled libraries"
	@echo "  deploy        - Create complete distribution package"
	@echo "  dev           - Development build with cargo (requires manual setup)"
	@echo "  install       - Install the application (not yet implemented)"
	@echo "  clean         - Clean cargo build artifacts"
	@echo ""
	@echo "Cargo shortcuts:"
	@echo "  cargo-clean       - Clean everything (cargo clean)"
	@echo "  cargo-release     - Build release build (cargo build --release)"
	@echo "  cargo-run-release - Run the release build (cargo run --release)"
	@echo "  cargo-run         - Run the dev build (cargo run)"
	@echo ""
	@echo "For new users, try: make setup && make run"
	@echo "For standalone distribution: make deploy"
	@echo ""
	@echo "Note: This project requires GStreamer for video functionality."
	@echo "GStreamer will be downloaded automatically during setup."
	@echo "Standalone executable available after 'make release' at target/release/run.sh"

setup:
	@echo "Setting up cross-platform dependencies..."
	./setup_deps.sh
	@echo "Fixing pkg-config library naming issues..."
	@sed -i '' 's/-lglib_2\.0/-lglib-2.0/g' deps/lib/pkgconfig/glib-2.0.pc 2>/dev/null || true
	@sed -i '' 's/-lgio_2\.0/-lgio-2.0/g' deps/lib/pkgconfig/gio-2.0.pc 2>/dev/null || true
	@sed -i '' 's/-lgobject_2\.0/-lgobject-2.0/g' deps/lib/pkgconfig/gobject-2.0.pc 2>/dev/null || true
	@echo "Dependencies setup complete!"

fix-libs:
	@echo "Fixing pkg-config library naming issues..."
	@sed -i '' 's/-lglib_2\.0/-lglib-2.0/g' deps/lib/pkgconfig/glib-2.0.pc 2>/dev/null || true
	@sed -i '' 's/-lgio_2\.0/-lgio-2.0/g' deps/lib/pkgconfig/gio-2.0.pc 2>/dev/null || true
	@sed -i '' 's/-lgobject_2\.0/-lgobject-2.0/g' deps/lib/pkgconfig/gobject-2.0.pc 2>/dev/null || true
	@echo "Library naming fixes applied!"

build: check-assets
	@echo "Building with cross-platform script..."
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && ./build.sh

run: release
	@echo "Running Gomoku with proper library and plugin paths..."
	DYLD_LIBRARY_PATH=$(PWD)/deps/lib:$$DYLD_LIBRARY_PATH \
	GST_PLUGIN_PATH=$(PWD)/deps/lib:$$GST_PLUGIN_PATH \
	GST_REGISTRY_FORK=no \
	GST_REGISTRY_UPDATE=no \
	./target/release/gomoku

check-assets:
	@if [ ! -f assets/backgrounds/ingame-background/*.png ] && [ -f assets/backgrounds/ingame-background/ingame-background.zip ]; then \
		cd assets/backgrounds/ingame-background && unzip -o ingame-background.zip; \
	fi
	@if [ ! -f assets/backgrounds/dolphin/*.png ] && [ -f assets/backgrounds/dolphin/dolphin.zip ]; then \
		cd assets/backgrounds/dolphin && unzip -o dolphin.zip; \
	fi

dev: check-assets
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && cargo run

release: check-assets
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && cargo build --release
	@echo "Setting up library structure for standalone execution..."
	@mkdir -p target/release/lib
	@echo "Copying all dynamic libraries..."
	@cp deps/lib/*.dylib target/release/lib/ 2>/dev/null || true
	@echo "Removing duplicate symbolic links to prevent conflicts..."
	@cd target/release/lib && find . -type l -name "*.dylib" -delete
	@echo "Creating standalone run script..."
	@echo "#!/bin/bash" > target/release/run.sh
	@echo "cd \"\$$(dirname \"\$$0\")\"" >> target/release/run.sh
	@echo "export DYLD_LIBRARY_PATH=\$$(pwd)/lib:\$$DYLD_LIBRARY_PATH" >> target/release/run.sh
	@echo "export GST_PLUGIN_PATH=\$$(pwd)/lib:\$$GST_PLUGIN_PATH" >> target/release/run.sh
	@echo "export GST_REGISTRY_FORK=no" >> target/release/run.sh
	@echo "export GST_REGISTRY_UPDATE=no" >> target/release/run.sh
	@echo "./gomoku \"\$$@\"" >> target/release/run.sh
	@chmod +x target/release/run.sh
	@echo "Standalone executable ready at target/release/gomoku"
	@echo "Run with: cd target/release && ./run.sh"

install:
	@echo "Install target not yet implemented"

clean:
	cargo clean
	rm -rf dist
	@echo "Cargo build artifacts and distribution cleaned"

deploy: release
	@echo "Creating deployment package..."
	@rm -rf dist/gomoku
	@mkdir -p dist/gomoku
	@cp target/release/gomoku dist/gomoku/
	@cp -r target/release/lib dist/gomoku/
	@cp -r assets dist/gomoku/
	@cp -r config dist/gomoku/
	@echo "#!/bin/bash" > dist/gomoku/run.sh
	@echo "cd \"\$$(dirname \"\$$0\")\"" >> dist/gomoku/run.sh
	@echo "export DYLD_LIBRARY_PATH=\$$(pwd)/lib:\$$DYLD_LIBRARY_PATH" >> dist/gomoku/run.sh
	@echo "export GST_PLUGIN_PATH=\$$(pwd)/lib:\$$GST_PLUGIN_PATH" >> dist/gomoku/run.sh
	@echo "export GST_REGISTRY_FORK=no" >> dist/gomoku/run.sh
	@echo "export GST_REGISTRY_UPDATE=no" >> dist/gomoku/run.sh
	@echo "./gomoku \"\$$@\"" >> dist/gomoku/run.sh
	@chmod +x dist/gomoku/run.sh
	@echo "Complete distribution created in dist/gomoku/"
	@echo "Run with: cd dist/gomoku && ./run.sh"

# Cargo shortcuts
cargo-clean:
	@echo "Cleaning everything with cargo clean..."
	cargo clean
	@echo "Cargo clean complete!"

cargo-release: check-assets
	@echo "Building release build with cargo..."
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && cargo build --release
	@echo "Release build complete!"

cargo-run-release: check-assets
	@echo "Running release build with cargo..."
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && \
	DYLD_LIBRARY_PATH=$(PWD)/deps/lib:$$DYLD_LIBRARY_PATH \
	GST_PLUGIN_PATH=$(PWD)/deps/lib:$$GST_PLUGIN_PATH \
	GST_REGISTRY_FORK=no \
	GST_REGISTRY_UPDATE=no \
	cargo run --release

cargo-run: check-assets
	@echo "Running dev build with cargo..."
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && \
	DYLD_LIBRARY_PATH=$(PWD)/deps/lib:$$DYLD_LIBRARY_PATH \
	GST_PLUGIN_PATH=$(PWD)/deps/lib:$$GST_PLUGIN_PATH \
	GST_REGISTRY_FORK=no \
	GST_REGISTRY_UPDATE=no \
	cargo run
