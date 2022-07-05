all: test

clean:
	cargo clean

test:
	cargo test

.PHONY: all clean test
