# Cargo profile for builds. Default is for local builds, CI uses an override.
PROFILE ?= release

build-lib: ## Build the `reth-exex-plugin` lib & bin into a `/target` directory.
	cargo build --profile "$(PROFILE)"

.PHONY: fmt
fmt:
	cargo +nightly fmt

.PHONY: test
test: ## tests from whole `reth-exex-plugin` crate (included doc tests also)
	cargo test -- --nocapture