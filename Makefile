.PHONY: all build test lint fmt audit check

all: check

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --check

audit:
	cargo audit

check: build lint fmt audit test
