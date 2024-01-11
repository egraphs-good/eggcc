.PHONY: test test-clean nits nightly

DIRS = . uniqueargs tree_unique_args

all: test nits

test:
	for dir in $(DIRS) ; do \
		(cd $$dir && cargo insta test --release --unreferenced=reject) ; \
	done

test-clean:
	for dir in $(DIRS) ; do \
		(cd $$dir && cargo insta test --release --unreferenced=delete) ; \
	done

nits:
	@rustup component add clippy
	@rustup component add rustfmt
	for dir in $(DIRS) ; do \
		(cd $$dir && cargo clippy --tests -- -D warnings && cargo fmt --check) ; \
	done

nightly:
	bash infra/nightly.sh
