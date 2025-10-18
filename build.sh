#!/bin/bash

# Cross-platform build script for Gomoku
# Handles dependencies and builds for both ARM64 and x86_64

set -e

ARCH=$(uname -m)
OS=$(uname -s)

echo "Building Gomoku for $OS $ARCH..."

# Setup dependencies
if [ ! -d "deps" ]; then
    echo "Setting up dependencies..."
    ./setup_deps.sh
fi

# Set environment variables for pkg-config
export PKG_CONFIG_PATH="$PWD/deps/lib/pkgconfig:$PKG_CONFIG_PATH"
export LD_LIBRARY_PATH="$PWD/deps/lib:$LD_LIBRARY_PATH"

# Disable Wayland to force X11-only builds
export WINIT_UNIX_BACKEND=x11
export WAYLAND_DISPLAY=""
unset WAYLAND_DISPLAY

# macOS specific settings
if [[ "$OS" == "Darwin" ]]; then
    export DYLD_LIBRARY_PATH="$PWD/deps/lib:$DYLD_LIBRARY_PATH"
    # Try Homebrew paths
    if [ -d "/opt/homebrew/lib/pkgconfig" ]; then
        export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
    fi
    if [ -d "/usr/local/lib/pkgconfig" ]; then
        export PKG_CONFIG_PATH="/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH"
    fi
fi

echo "Environment configured:"
echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH"

# Clean previous build
echo "Cleaning previous build..."
cargo clean

# Build with release optimizations
echo "Building Gomoku..."
cargo build --release

echo "Build complete!"
echo "Executable: target/release/gomoku"

# Create portable distribution
echo "Creating portable distribution..."
mkdir -p dist
cp target/release/gomoku dist/

# Copy assets to the correct location for the executable
if [ -d "assets" ]; then
    cp -r assets dist/
    # Also create a symlink in target for development
    if [ ! -L "target/assets" ]; then
        ln -sf ../assets target/assets
    fi
else
    echo "Warning: No assets directory found"
fi

# Copy dependencies
if [ -d "deps/lib" ]; then
    mkdir -p dist/lib
    cp deps/lib/* dist/lib/ 2>/dev/null || true
fi

# Create launcher script
if [[ "$OS" == "Darwin" ]]; then
    cat > dist/run_gomoku.sh << 'EOF'
#!/bin/bash
export DYLD_LIBRARY_PATH="$(dirname "$0")/lib:$DYLD_LIBRARY_PATH"
"$(dirname "$0")/gomoku" "$@"
EOF
else
    cat > dist/run_gomoku.sh << 'EOF'
#!/bin/bash
export LD_LIBRARY_PATH="$(dirname "$0")/lib:$LD_LIBRARY_PATH"
"$(dirname "$0")/gomoku" "$@"
EOF
fi

chmod +x dist/run_gomoku.sh

echo "Portable distribution created in ./dist/"
echo "Run with: ./dist/run_gomoku.sh"
