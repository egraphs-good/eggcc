.PHONY: test test-clean nits nightly

DIR = tree_unique_args

all: test nits

test:
	cd $(DIR) && cargo insta test --release --unreferenced=reject

test-clean:
	cd $(DIR) && cargo insta test --release --unreferenced=delete

nits:
	cd $(DIR) && @rustup component add clippy
	cd $(DIR) && cargo clippy --tests -- -D warnings
	cd $(DIR) && @rustup component add rustfmt
	cd $(DIR) && cargo fmt --check

nightly:
	bash infra/nightly.sh
