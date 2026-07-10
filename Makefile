.PHONY: all do-it-lady build test lint fmt audit check help config fetch count-items list-items delete-items digest daily

help: ## Show this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

config: ## Configure br1ef preferences
	cargo run config

fetch: ### Fetch raw data from configured sources
	cargo run fetch

count-items: ## Show stored item counts by source
	cargo run count-items

list-items: ## Show stored item info
	cargo run list-items

delete-items: ## Delete all stored items
	cargo run delete-items

digest: ## Digest fetched data into a brief
	cargo run digest

daily: ## Show the daily brief
	cargo run daily

build: ## Build the workspace
	cargo build --workspace

test: ## Run tests
	cargo test --workspace

lint: ## Run clippy (deny warnings)
	cargo clippy --workspace -- -D warnings

fmt: ## Format source files (write mode)
	cargo fmt

fmt-chk: ## Check formatting (dry-run)
	cargo fmt --check

audit: ## Audit dependencies for vulnerabilities
	cargo audit

all: check ## Run check target

check: build lint fmt audit test ## Build + lint + fmt + audit + test

do-it-lady: ## Quick pipeline: fetch → digest → daily (quiet)
	cargo run -q fetch && cargo run -q digest && cargo run -q daily
