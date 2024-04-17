.PHONY: test test-clean nits nightly

DIRS = . dag_in_context

all: nits test

test:
	$(foreach dir,$(DIRS),(cd $(dir) && cargo insta test --release --unreferenced=reject) &&) :

test-clean:
	$(foreach dir,$(DIRS),(cd $(dir) && cargo insta test --release --unreferenced=delete) &&) :

nits:
	@rustup component add clippy
	@rustup component add rustfmt
	$(foreach dir,$(DIRS),(cd $(dir) && cargo clippy --tests -- -D warnings && cargo fmt --check) &&) :

nightly:
	bash infra/nightly.sh
