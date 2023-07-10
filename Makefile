.PHONY: test nits docs

RUST_SRC=$(shell find -type f -wholename '*/src/*.rs' -or -name 'Cargo.toml')
TESTS=$(shell find tests/ -type f -name '*.egg' -not -name '*repro-*')

all: test nits docs

test:
	cargo test --release -- -Zunstable-options --report-time

nits:
	@rustup component add clippy
	cargo clippy --tests -- -D warnings
	@rustup component add rustfmt
	cargo fmt --check
