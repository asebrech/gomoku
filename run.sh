#!/bin/bash

# Quick run script for Gomoku with all environment variables set
cd "$(dirname "$0")"

# Set up dependencies if needed
if [ ! -d "deps" ]; then
    echo "Setting up dependencies..."
    ./setup_deps.sh
fi

# Create assets symlink if needed
if [ ! -L "target/assets" ] && [ -d "assets" ]; then
    mkdir -p target
    ln -sf ../assets target/assets
fi

# Set environment variables
export PKG_CONFIG_PATH="$PWD/deps/lib/pkgconfig:$PKG_CONFIG_PATH"
export LD_LIBRARY_PATH="$PWD/deps/lib:$LD_LIBRARY_PATH"
export WINIT_UNIX_BACKEND=x11

echo "Starting Gomoku..."
cargo run --release
