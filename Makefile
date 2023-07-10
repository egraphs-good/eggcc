.PHONY: test nits docs

all: test nits docs

test:
	cargo test --release

nits:
	@rustup component add clippy
	cargo clippy --tests -- -D warnings
	@rustup component add rustfmt
	cargo fmt --check
