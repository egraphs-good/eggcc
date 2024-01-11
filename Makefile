.PHONY: test test-clean nits nightly

all: test nits

test:
	cd tree_unique_args
	cargo insta test --release --unreferenced=reject

test-clean:
	cargo insta test --release --unreferenced=delete

nits:
	cd tree_unique_args
	@rustup component add clippy
	cargo clippy --tests -- -D warnings
	@rustup component add rustfmt
	cargo fmt --check

nightly:
	bash infra/nightly.sh
