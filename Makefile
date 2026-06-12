.PHONY: fmt clippy audit test build install

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

audit:
	cargo audit

test:
	cargo test --verbose

build:
	cargo build --release

install:
	cargo build --release
	sudo cp target/release/zephyr_vault /usr/local/bin/

ci: fmt clippy audit test build
	@echo "✅ All CI checks passed"

help:
	@echo "Available targets: fmt, clippy, audit, test, build, install, ci"