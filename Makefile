.PHONY: build clean test

build:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
