NAME = gomoku

# Default target: build for development
all: $(NAME)

$(NAME): install_frontend wasm

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

# Development mode
dev:
	cd frontend && pnpm dev

run: all dev

start: dev

# Production mode
build_frontend:
	cd frontend && pnpm build

build_prod: all build_frontend

preview: build_prod
	cd frontend && pnpm preview

start-prod: preview

# Utilities
install:
	@bash scripts/install_prereqs.sh

test:
	cargo test

# Cleaning targets
clean:
	@echo "Cleaning build artifacts..."
	rm -rf frontend/src/lib/wasm/pkg
	rm -f .cargo/config.toml.backup
	cargo clean

clean_dev:
	@echo "Cleaning development artifacts..."
	rm -rf frontend/.svelte-kit/output

clean_prod: clean_dev
	@echo "Cleaning production build..."
	rm -rf frontend/.svelte-kit/output

fclean: clean
	@echo "Full clean: removing all dependencies..."
	rm -rf frontend/node_modules
	rm -rf frontend/.svelte-kit
	rm -f Cargo.lock

re: fclean all

.PHONY: all wasm dev run start build_frontend build_prod preview start-prod install install_frontend test clean clean_dev clean_prod fclean re
