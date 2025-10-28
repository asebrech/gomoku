NAME = gomoku

all: $(NAME)

$(NAME): wasm

wasm:
	wasm-pack build --target web --out-dir frontend/src/lib/wasm/pkg

dev:
	cd frontend && pnpm dev

start: dev

install:
	cd frontend && pnpm install
	curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

test:
	cargo test --features parallel

clean:
	rm -rf frontend/src/lib/wasm/pkg
	cargo clean

fclean: clean

re: fclean all

.PHONY: all wasm dev start install test clean fclean re
