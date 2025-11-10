NAME = gomoku

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

dev:
	cd frontend && pnpm dev

run: all dev

start: dev

install:
	@bash scripts/install_prereqs.sh

test:
	cargo test

clean:
	rm -rf frontend/src/lib/wasm/pkg
	rm -rf frontend/build
	rm -f .cargo/config.toml.backup
	cargo clean

fclean: clean
	rm -rf frontend/node_modules
	rm -rf frontend/.svelte-kit
	rm -f Cargo.lock

re: fclean all

.PHONY: all wasm dev run start install install_frontend test clean fclean re
