NAME = gomoku

# Environment setup for tools
export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/bin:$(HOME)/.local/share/pnpm:$(PATH)
export PNPM_HOME := $(HOME)/.local/share/pnpm

# Default target: build for production (42 school requirement)
all: $(NAME)

$(NAME): install_frontend wasm build_frontend

install_frontend:
	@if [ ! -d "frontend/node_modules" ]; then \
		echo "Installing frontend dependencies..."; \
		cd frontend && pnpm install; \
	fi

wasm:
	@cp .cargo/config.toml .cargo/config.toml.backup
	@echo "" >> .cargo/config.toml
	@echo "[unstable]" >> .cargo/config.toml
	@echo 'build-std = ["panic_abort", "std"]' >> .cargo/config.toml
	@wasm-pack build --release --target web --out-dir frontend/src/lib/wasm/pkg || (mv .cargo/config.toml.backup .cargo/config.toml && exit 1)
	@mv .cargo/config.toml.backup .cargo/config.toml

# Production mode (default)
build_frontend:
	cd frontend && pnpm build

run: all
	cd frontend && pnpm preview

# Development mode
dev: install_frontend wasm
	cd frontend && pnpm dev

# Utilities
install:
	@bash scripts/install_prereqs.sh

test:
	cargo test

# Cleaning targets
clean:
	@echo "Cleaning build artifacts..."
	rm -rf frontend/src/lib/wasm/pkg
	rm -rf frontend/.svelte-kit
	rm -f .cargo/config.toml.backup
	cargo clean

fclean: clean
	@echo "Full clean: removing all dependencies..."
	rm -rf frontend/node_modules
	rm -f Cargo.lock

re: fclean all

.PHONY: all wasm build_frontend run dev install install_frontend test clean fclean re
