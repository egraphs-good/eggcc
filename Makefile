.PHONY: test test-clean nits nightly

DIR = tree_unique_args

all: test nits

test:
	cd $(DIR) && cargo insta test --release --unreferenced=reject

test-clean:
	cd $(DIR) && cargo insta test --release --unreferenced=delete

nits:
	@rustup component add clippy
	@rustup component add rustfmt
	cd $(DIR) && cargo clippy --tests -- -D warnings
	cd $(DIR) && cargo fmt --check

nightly:
	bash infra/nightly.sh
