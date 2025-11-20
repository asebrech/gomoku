#!/bin/bash
# Source this file to set up the development environment
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$HOME/.local/share/pnpm:$PATH"
export PNPM_HOME="$HOME/.local/share/pnpm"
echo "Environment set up. You can now use rustc, wasm-pack, node, and pnpm."
