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
    
    # Source the cargo environment
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    export PATH="$HOME/.cargo/bin:$PATH"
    
    # Verify rustup installation
    if ! command_exists rustup; then
        warn "rustup installation may have failed"
    else
        info "rustup successfully installed"
    fi
else
    info "rustup found"
fi

# Ensure cargo/rust tools are in PATH for this session
export PATH="$HOME/.cargo/bin:$PATH"

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
    
    # Update PATH to include wasm-pack (usually installed to ~/.cargo/bin)
    export PATH="$HOME/.cargo/bin:$PATH"
    
    # Verify installation
    if ! command_exists wasm-pack; then
        warn "wasm-pack installation may have failed or is not in PATH"
        info "Trying alternative installation method via cargo"
        if command_exists cargo; then
            cargo install wasm-pack
        else
            warn "cargo not found - please ensure Rust is properly installed"
        fi
    else
        info "wasm-pack successfully installed"
    fi
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
        info "apt-get detected — installing Node.js via NodeSource without sudo"
        # Use NodeSource binary distributions that don't require sudo
        NODE_VERSION="20"
        NODE_DISTRO="linux-x64"
        NODE_URL="https://nodejs.org/dist/latest-v${NODE_VERSION}.x/node-v${NODE_VERSION}.*-${NODE_DISTRO}.tar.xz"
        
        # Get the latest Node.js version URL
        LATEST_NODE=$(curl -s https://nodejs.org/dist/latest-v${NODE_VERSION}.x/ | grep -o "node-v${NODE_VERSION}\.[0-9]*\.[0-9]*-${NODE_DISTRO}\.tar\.xz" | head -1)
        if [ -n "$LATEST_NODE" ]; then
            NODE_URL="https://nodejs.org/dist/latest-v${NODE_VERSION}.x/$LATEST_NODE"
            INSTALL_DIR="$HOME/.local"
            mkdir -p "$INSTALL_DIR"
            
            info "Downloading and installing Node.js to $INSTALL_DIR"
            cd /tmp
            curl -fsSL "$NODE_URL" | tar -xJ
            NODE_DIR=$(echo "$LATEST_NODE" | sed 's/\.tar\.xz//')
            cp -r "$NODE_DIR"/* "$INSTALL_DIR/"
            
            # Add to PATH if not already there
            if [[ ":$PATH:" != *":$INSTALL_DIR/bin:"* ]]; then
                export PATH="$INSTALL_DIR/bin:$PATH"
                echo "export PATH=\"$INSTALL_DIR/bin:\$PATH\"" >> "$HOME/.bashrc"
                info "Added $INSTALL_DIR/bin to PATH in ~/.bashrc"
            fi
            
            # Install pnpm using corepack (comes with Node.js 16+)
            if command_exists corepack; then
                info "Installing pnpm via corepack"
                corepack enable
                corepack prepare pnpm@latest --activate
            elif command_exists npm; then
                info "Installing pnpm via npm"
                npm install -g pnpm
            fi
        else
            warn "Could not determine latest Node.js version. Trying alternative installation method..."
            # Fallback to n (Node.js version manager)
            if ! command_exists n; then
                info "Installing n (Node.js version manager)"
                curl -fsSL https://raw.githubusercontent.com/tj/n/master/bin/n | bash -s lts
                export PATH="$HOME/n/bin:$PATH"
            fi
            n lts
            npm install -g pnpm || true
        fi
    elif command_exists npm; then
        info "npm detected — installing pnpm without sudo"
        # Try to install pnpm globally without sudo first
        if npm install -g pnpm 2>/dev/null; then
            info "pnpm installed globally"
        else
            # If global install fails, set up local npm prefix and install there
            info "Global install failed, setting up local npm prefix"
            mkdir -p "$HOME/.npm-global"
            npm config set prefix "$HOME/.npm-global"
            export PATH="$HOME/.npm-global/bin:$PATH"
            
            # Add to bashrc if not already there
            if ! grep -q "npm-global/bin" "$HOME/.bashrc" 2>/dev/null; then
                echo "export PATH=\"$HOME/.npm-global/bin:\$PATH\"" >> "$HOME/.bashrc"
                info "Added npm global bin directory to PATH in ~/.bashrc"
            fi
            
            npm install -g pnpm || warn "Failed to install pnpm locally"
        fi
    else
        info "Attempting direct pnpm installation without package manager"
        # Try installing pnpm directly using their standalone script
        curl -fsSL https://get.pnpm.io/install.sh | sh -
        
        # Source the pnpm environment
        export PNPM_HOME="$HOME/.local/share/pnpm"
        export PATH="$PNPM_HOME:$PATH"
        
        # Add to bashrc if not already there
        if ! grep -q "PNPM_HOME" "$HOME/.bashrc" 2>/dev/null; then
            echo "export PNPM_HOME=\"$HOME/.local/share/pnpm\"" >> "$HOME/.bashrc"
            echo "export PATH=\"\$PNPM_HOME:\$PATH\"" >> "$HOME/.bashrc"
            info "Added pnpm to PATH in ~/.bashrc"
        fi
        
        if ! command_exists pnpm; then
            warn "All installation methods failed. Please install Node (>=16) and pnpm manually: https://pnpm.io/installation"
        fi
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

echo ""
echo "=== IMPORTANT ==="
echo "To use the installed tools in your CURRENT shell session, run:"
echo "  source $ROOT_DIR/scripts/install_prereqs.sh"
echo ""
echo "For future terminal sessions, the tools will be available after running:"
echo "  source ~/.bashrc"
echo "Or by restarting your terminal."
echo ""
echo "Quick start (run these commands):"
echo "  source $ROOT_DIR/scripts/install_prereqs.sh"
echo "  make wasm && cd frontend && pnpm run dev"

# Environment setup - this section can be sourced to set up the current shell
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$HOME/.local/share/pnpm:$PATH"
export PNPM_HOME="$HOME/.local/share/pnpm"

# Only show this message when sourcing (not when executing)
if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    echo "Environment set up. You can now use rustc, wasm-pack, node, and pnpm."
fi
