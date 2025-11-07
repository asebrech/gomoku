#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
echo "[gomoku] Installing prerequisites (root: $ROOT_DIR)"

# Helpers
command_exists() { command -v "$1" >/dev/null 2>&1; }
info() { echo "[info] $*"; }
warn() { echo "[warn] $*"; }

# 1) rustup
if ! command_exists rustup; then
    info "rustup not found — installing rustup (will use default settings)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
else
    info "rustup found"
fi

# read toolchain from rust-toolchain.toml if present, fallback to nightly-2024-08-02
TOOLCHAIN="nightly-2024-08-02"
if [ -f "$ROOT_DIR/rust-toolchain.toml" ]; then
    # naive parse
    parsed=$(grep -E "^channel\s*=\s*\".*\"" "$ROOT_DIR/rust-toolchain.toml" || true)
    if [ -n "$parsed" ]; then
        TOOLCHAIN=$(echo "$parsed" | sed -E 's/.*"([^"]+)".*/\1/')
    fi
fi
info "Ensuring Rust toolchain: $TOOLCHAIN"
rustup toolchain install "$TOOLCHAIN" || true

# 2) wasm target for the toolchain
info "Adding wasm target wasm32-unknown-unknown for $TOOLCHAIN"
rustup target add wasm32-unknown-unknown --toolchain "$TOOLCHAIN" || true

# 3) wasm-pack
if ! command_exists wasm-pack; then
    info "wasm-pack not found — installing wasm-pack"
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
else
    info "wasm-pack found"
fi

# 4) Node and pnpm
if command_exists pnpm; then
    info "pnpm found"
else
    info "pnpm not found — attempting to install Node + pnpm"
    OS="$(uname -s)"
    if command_exists brew; then
        info "Homebrew detected — installing node and pnpm via brew"
        brew install node pnpm || true
    elif [ "$OS" = "Linux" ] && command_exists apt-get; then
        info "apt-get detected — installing nodejs/npm and pnpm via apt/npm (requires sudo)"
        sudo apt-get update
        sudo apt-get install -y nodejs npm || true
        if command_exists npm; then
            sudo npm install -g pnpm || true
        fi
    elif command_exists npm; then
        info "npm detected — installing pnpm globally via npm"
        npm install -g pnpm || true
    else
        warn "Could not auto-install pnpm. Please install Node (>=16) and pnpm manually: https://pnpm.io/installation"
    fi
fi

# 5) Install frontend deps
if [ -d "$ROOT_DIR/frontend" ]; then
    info "Installing frontend dependencies (pnpm install)"
    (cd "$ROOT_DIR/frontend" && pnpm install)
else
    warn "No frontend directory found at $ROOT_DIR/frontend"
fi

info "Prerequisites installation complete. Summary:" 
info " - rustup: $(command_exists rustup && echo 'installed' || echo 'missing')"
info " - rust toolchain: $TOOLCHAIN"
info " - wasm target wasm32-unknown-unknown: installed (if rustup succeeded)"
info " - wasm-pack: $(command_exists wasm-pack && echo 'installed' || echo 'missing')"
info " - pnpm: $(command_exists pnpm && echo 'installed' || echo 'missing')"

echo "Done. You can now run 'make wasm' and 'cd frontend && pnpm run dev' or './run.sh' for the native app."
