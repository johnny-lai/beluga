build:
	cargo build

musl.build:
	docker run --rm -it -v `pwd`:/app -w /app clux/muslrust cargo build

test:
	cd examples/rails1 && ../../target/buil
