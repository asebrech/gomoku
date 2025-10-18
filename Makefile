.PHONY: dev release clean up up-dev fast help setup check-assets all-build deploy

# Default target - show help
all: help

help:
	@echo "Gomoku Build System - Simple Commands"
	@echo ""
	@echo "Main targets:"
	@echo "  make release  - Setup libs + build optimized release version"
	@echo "  make dev      - Setup libs + build development version (fast compile, optimized deps)"
	@echo "  make up       - Build and run the release executable"
	@echo "  make up-dev   - Build and run the development executable (faster startup)"
	@echo "  make fast     - Ultra-fast development mode (no video, instant startup)"
	@echo "  make clean    - Clean everything (cargo + deps)"
	@echo ""
	@echo "Advanced targets:"
	@echo "  setup         - Only setup dependencies"
	@echo "  all-build     - Complete build pipeline with distribution"
	@echo "  deploy        - Create complete distribution package"
	@echo ""
	@echo "For new users: make release && make up"
	@echo "For development: make dev && make up-dev (faster)"
	@echo "For instant testing: make fast (no video, ultra-fast)"
	@echo ""
	@echo "Note: GStreamer dependencies are downloaded automatically."

# Main targets - Simple and clear

# Setup libs + build optimized release version
release: setup
	@echo "Building optimized release version..."
	@echo "Setting devMode to false for release..."
	@sed -i '' 's/"devMode": true/"devMode": false/g' config/config.json
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && cargo build --release
	@echo "✅ Release build complete!"
	@echo "Run with: make up"

# Setup libs + build development version (fast compile, optimized deps)
dev: setup
	@echo "Building development version with optimized dependencies..."
	@echo "Setting devMode to true for development..."
	@sed -i '' 's/"devMode": false/"devMode": true/g' config/config.json
	export PKG_CONFIG_PATH=$(PWD)/deps/lib/pkgconfig:$$PKG_CONFIG_PATH && cargo build
	@echo "✅ Development build complete!"
	@echo "Run with: make up-dev"

# Build and run the executable
up: release check-assets
	@echo "Running Gomoku..."
	@export DYLD_LIBRARY_PATH=$(PWD)/deps/lib:$$DYLD_LIBRARY_PATH && \
	export GST_PLUGIN_PATH=$(PWD)/deps/lib:$$GST_PLUGIN_PATH && \
	export GST_PLUGIN_SYSTEM_PATH="" && \
	export GST_REGISTRY=$(PWD)/deps/lib/registry.bin && \
	export GST_REGISTRY_FORK=no && \
	export GST_REGISTRY_UPDATE=yes && \
	if [ ! -f $(PWD)/deps/lib/registry.bin ]; then \
		rm -f ~/.cache/gstreamer-1.0/registry.*.bin 2>/dev/null || true; \
		echo "Building GStreamer plugin registry..."; \
	fi && \
	./target/release/gomoku

# Build and run the development executable (faster startup)
up-dev: dev check-assets
	@echo "Running Gomoku (development version)..."
	@if [ ! -L target/debug/assets ]; then \
		ln -sf ../../assets target/debug/assets; \
		ln -sf ../../config target/debug/config; \
	fi
	@export DYLD_LIBRARY_PATH=$(PWD)/deps/lib:$$DYLD_LIBRARY_PATH && \
	export GST_PLUGIN_PATH=$(PWD)/deps/lib:$$GST_PLUGIN_PATH && \
	export GST_PLUGIN_SYSTEM_PATH="" && \
	export GST_REGISTRY=$(PWD)/deps/lib/registry.bin && \
	export GST_REGISTRY_FORK=no && \
	export GST_REGISTRY_UPDATE=no && \
	./target/debug/gomoku

# Ultra-fast development mode (no video, instant startup)
fast: dev check-assets
	@echo "Running Gomoku (ultra-fast mode - no video)..."
	@export DYLD_LIBRARY_PATH=$(PWD)/deps/lib:$$DYLD_LIBRARY_PATH && \
	export GST_PLUGIN_PATH="" && \
	export GST_PLUGIN_SYSTEM_PATH="" && \
	export GST_REGISTRY_UPDATE=no && \
	export GST_REGISTRY_FORK=no && \
	./target/debug/gomoku

# Clean everything (cargo + deps)
clean:
	@echo "Cleaning everything..."
	cargo clean
	rm -rf deps
	rm -rf dist
	@echo "✅ Complete cleanup finished!"

# Advanced targets

setup:
	@echo "Setting up cross-platform dependencies..."
	./setup_deps.sh
	@echo "Fixing pkg-config library naming issues..."
	@sed -i '' 's/-lglib_2\.0/-lglib-2.0/g' deps/lib/pkgconfig/glib-2.0.pc 2>/dev/null || true
	@sed -i '' 's/-lgio_2\.0/-lgio-2.0/g' deps/lib/pkgconfig/gio-2.0.pc 2>/dev/null || true
	@sed -i '' 's/-lgobject_2\.0/-lgobject-2.0/g' deps/lib/pkgconfig/gobject-2.0.pc 2>/dev/null || true
	@echo "✅ Dependencies setup complete!"

check-assets:
	@if [ ! -f assets/backgrounds/ingame-background/*.png ] && [ -f assets/backgrounds/ingame-background/ingame-background.zip ]; then \
		cd assets/backgrounds/ingame-background && unzip -o ingame-background.zip; \
	fi
	@if [ ! -f assets/backgrounds/dolphin/*.png ] && [ -f assets/backgrounds/dolphin/dolphin.zip ]; then \
		cd assets/backgrounds/dolphin && unzip -o dolphin.zip; \
	fi

# Build everything from scratch with distribution
all-build: clean setup release deploy
	@echo "===========================================" 
	@echo "🎉 COMPLETE BUILD PIPELINE FINISHED! 🎉"
	@echo "==========================================="
	@echo ""
	@echo "✅ Standalone executable: target/release/run.sh"
	@echo "✅ Distribution package: dist/gomoku/"
	@echo ""
	@echo "Test with: make up"
	@echo "Or distribute: cd dist/gomoku && ./run.sh"

deploy: 
	@if [ ! -f target/release/gomoku ]; then \
		echo "Release build not found, building first..."; \
		$(MAKE) release; \
	fi
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
	@echo "export GST_PLUGIN_SYSTEM_PATH=" >> dist/gomoku/run.sh
	@echo "export GST_REGISTRY=\$$(pwd)/lib/registry.bin" >> dist/gomoku/run.sh
	@echo "export GST_REGISTRY_FORK=no" >> dist/gomoku/run.sh
	@echo "export GST_REGISTRY_UPDATE=yes" >> dist/gomoku/run.sh
	@echo "# Force plugin discovery if registry doesn't exist" >> dist/gomoku/run.sh
	@echo "if [ ! -f \$$(pwd)/lib/registry.bin ]; then" >> dist/gomoku/run.sh
	@echo "    rm -f ~/.cache/gstreamer-1.0/registry.*.bin 2>/dev/null || true" >> dist/gomoku/run.sh
	@echo "    echo 'Building GStreamer plugin registry...'" >> dist/gomoku/run.sh
	@echo "fi" >> dist/gomoku/run.sh
	@echo "./gomoku \"\$$@\"" >> dist/gomoku/run.sh
	@chmod +x dist/gomoku/run.sh
	@echo "✅ Complete distribution created in dist/gomoku/"
	@echo "Run with: cd dist/gomoku && ./run.sh"
