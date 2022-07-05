all: test

clean:
	cargo clean

test:
	cargo test

lint:
	cargo clippy

.PHONY: all clean test lint
