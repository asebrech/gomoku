NAME = gomoku

all: $(NAME)

$(NAME): wasm

wasm:
	@cp .cargo/config.toml .cargo/config.toml.backup
	@echo "" >> .cargo/config.toml
	@echo "[unstable]" >> .cargo/config.toml
	@echo 'build-std = ["panic_abort", "std"]' >> .cargo/config.toml
	@wasm-pack build --target web --out-dir frontend/src/lib/wasm/pkg || (mv .cargo/config.toml.backup .cargo/config.toml && exit 1)
	@mv .cargo/config.toml.backup .cargo/config.toml

dev:
	cd frontend && pnpm dev

start: dev

install:
	cd frontend && pnpm install
	curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

test:
	cargo test

clean:
	rm -rf frontend/src/lib/wasm/pkg
	cargo clean

fclean: clean

re: fclean all

.PHONY: all wasm dev start install test clean fclean re
