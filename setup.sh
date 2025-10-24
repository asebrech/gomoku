#!/bin/bash

# Gomoku Setup Script
# This script installs all necessary dependencies to build and run the Gomoku project on macOS

set -e  # Exit on any error

echo "🎮 Setting up Gomoku development environment..."
echo ""

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew is not installed!"
    echo "Please install Homebrew first: https://brew.sh/"
    echo "Run: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    exit 1
fi

echo "✅ Homebrew found"

# Check if Rust/Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed!"
    echo "Please install Rust first: https://rustup.rs/"
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust/Cargo found"

# Update Homebrew
echo "📦 Updating Homebrew..."
brew update

# Install GStreamer and related dependencies
echo "🎬 Installing GStreamer..."
brew install gstreamer

echo "📚 Installing GStreamer development libraries..."
brew install gst-plugins-base

echo "🔧 Installing GStreamer good plugins..."
brew install gst-plugins-good

echo "🔧 Installing GStreamer bad plugins..."
brew install gst-plugins-bad

echo "🔧 Installing GStreamer ugly plugins..."
brew install gst-plugins-ugly

echo "🔧 Installing additional GStreamer plugins..."
brew install gst-libav

# Install pkg-config (needed for GStreamer detection)
echo "🔧 Installing pkg-config..."
brew install pkg-config

# Install additional system dependencies that might be needed
echo "🔧 Installing additional dependencies..."
brew install cairo
brew install pango
brew install gdk-pixbuf
brew install gtk+3

# Verify GStreamer installation
echo ""
echo "🔍 Verifying GStreamer installation..."
if pkg-config --exists gstreamer-1.0; then
    echo "✅ GStreamer 1.0 found: $(pkg-config --modversion gstreamer-1.0)"
else
    echo "❌ GStreamer 1.0 not found in pkg-config"
    exit 1
fi

if pkg-config --exists gstreamer-app-1.0; then
    echo "✅ GStreamer App found: $(pkg-config --modversion gstreamer-app-1.0)"
else
    echo "❌ GStreamer App not found in pkg-config"
    exit 1
fi

if pkg-config --exists gstreamer-video-1.0; then
    echo "✅ GStreamer Video found: $(pkg-config --modversion gstreamer-video-1.0)"
else
    echo "❌ GStreamer Video not found in pkg-config"
    exit 1
fi

# Set up environment variables persistently
echo ""
echo "🔧 Setting up environment variables..."

# Get Homebrew prefix (different on Intel vs Apple Silicon Macs)
HOMEBREW_PREFIX=$(brew --prefix)
echo "📍 Homebrew prefix: $HOMEBREW_PREFIX"

# Determine which shell profile to update
SHELL_PROFILE=""
if [[ "$SHELL" == */zsh ]]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [[ "$SHELL" == */bash ]]; then
    SHELL_PROFILE="$HOME/.bash_profile"
else
    SHELL_PROFILE="$HOME/.profile"
fi

echo "📝 Updating shell profile: $SHELL_PROFILE"

# Create the export line
EXPORT_LINE="export PKG_CONFIG_PATH=\"$HOMEBREW_PREFIX/lib/pkgconfig:\$PKG_CONFIG_PATH\""

# Check if the export is already in the profile
if grep -q "PKG_CONFIG_PATH.*$HOMEBREW_PREFIX/lib/pkgconfig" "$SHELL_PROFILE" 2>/dev/null; then
    echo "✅ PKG_CONFIG_PATH already configured in $SHELL_PROFILE"
else
    echo "# Added by Gomoku setup script" >> "$SHELL_PROFILE"
    echo "$EXPORT_LINE" >> "$SHELL_PROFILE"
    echo "✅ PKG_CONFIG_PATH added to $SHELL_PROFILE"
fi

# Set for current session
export PKG_CONFIG_PATH="$HOMEBREW_PREFIX/lib/pkgconfig:$PKG_CONFIG_PATH"
echo "✅ PKG_CONFIG_PATH set for current session"

# Show the current PKG_CONFIG_PATH
echo "📋 Current PKG_CONFIG_PATH: $PKG_CONFIG_PATH"

echo ""
echo "🎉 Setup complete!"
echo ""
echo "📝 IMPORTANT: The PKG_CONFIG_PATH has been added to your shell profile."
echo "   For the current terminal session, it's already set."
echo "   For new terminal sessions, restart your terminal or run:"
echo "   source $SHELL_PROFILE"
echo ""
echo "🚀 Now you can build and run the project:"
echo "   make dev    # Build development version"
echo "   make run    # Build and run release version"
echo "   cargo run   # Run directly with cargo"
echo ""
echo "🧪 To run tests:"
echo "   make test   # Run all tests"
echo ""
echo "📖 For more build options:"
echo "   make help   # Show all available targets"
