.PHONY: all build test lint fmt audit check

all: check

do-it-lady:
	cargo run fetch
	cargo run digest
	cargo run daily

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt

fmt-chk:
	cargo fmt --check

audit:
	cargo audit

check: build lint fmt audit test
