.PHONY: test test-clean nits nightly runtime gurobi-setup

DIRS = . dag_in_context

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


fixnits:
	npx prettier infra/nightly-resources/*.js --write
	cargo fmt
	cd dag_in_context && cargo fmt
	cargo clippy --fix --allow-dirty
	cd dag_in_context && cargo clippy --fix --allow-dirty

# build the llvm runtime for bril
# if you edit the runtime crate, you must re-run this to rebuild rt.bc
runtime:
	bash runtime/install.sh

nightly:
	bash infra/nightly.sh "benchmarks/passing"

# Install/verify an optional Gurobi license so the nightly auto-detects Gurobi.
# Usage: make gurobi-setup LICENSE=path/to/gurobi.lic  (LICENSE optional -- omit to just verify)
gurobi-setup:
	bash infra/setup_gurobi.sh "$(LICENSE)"
