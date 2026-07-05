.PHONY: all do-it-lady build test lint fmt audit check

all: check

do-it-lady:
	cargo run -q fetch > /dev/null
	cargo run -q digest > /dev/null
	cargo run -q daily > /dev/null

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
