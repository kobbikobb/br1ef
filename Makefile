.PHONY: all do-it-lady build test lint fmt audit check

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

all: check

check: build lint fmt audit test

fetch:
	cargo run fetch

count-items:
	cargo run count-items

list-items:
	cargo run list-items

delete-items:
	cargo run delete-items

do-it-lady:
	cargo run -q fetch
	cargo run -q digest
	cargo run -q daily
