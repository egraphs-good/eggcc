.PHONY: test test-clean nits nightly runtime

DIRS = . dag_in_context
CPPFLAGS = -O2 -Wno-unused-result

all: nits test

test:
	cargo insta test --release --unreferenced=reject
	cd dag_in_context && cargo insta test --release --unreferenced=reject

test-clean:
	cargo insta test --release --unreferenced=delete
	cd dag_in_context && cargo insta test --release --unreferenced=delete

nits:
	npx prettier infra/nightly-resources/*.js --check
	@rustup component add clippy
	@rustup component add rustfmt
	cargo clippy --tests -- -D warnings && cargo fmt --check
	cd dag_in_context && cargo clippy --tests -- -D warnings && cargo fmt --check


# build the llvm runtime for bril
# if you edit the runtime crate, you must re-run this to rebuild rt.bc
runtime:
	bash runtime/install.sh

nightly:
	bash infra/nightly.sh "benchmarks/passing"
