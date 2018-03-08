build:
	cargo build

musl.build:
	docker run --rm -it -v `pwd`:/app -w /app clux/muslrust cargo build
