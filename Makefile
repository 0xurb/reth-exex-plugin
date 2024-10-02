# Example plugins directory path
EXAMPLES_DIR ?= examples
# Cargo profile for builds. Default is for local builds, CI uses an override.
PROFILE ?= release

#@ common

.PHONY: build
build:
	make build-lib && \
	make build-examples

.PHONY: fmt
fmt:
	cargo +nightly fmt

.PHONY: test
test: ## tests from whole `reth-exex-plugin` crate (included doc tests also).
	cargo test -- --nocapture

.PHONY: clean
clean: ## cleanup for /target directory on all example plugins and `reth-exex-plugin` lib.
	cargo clean && \
	cd $(EXAMPLES_DIR)/minimal && \
	cargo clean

#@ `reth-exex-plugin` lib 

build-lib: ## Build the `reth-exex-plugin` lib & bin into a `/target` directory.
	cargo build --profile "$(PROFILE)"

#@ example plugins

build-examples: ## Build the `/examples/minimal` plugin dylib file(s) into a `examples/minimal/target` directory.
	cd $(EXAMPLES_DIR)/minimal && \
	cargo build --profile "$(PROFILE)"