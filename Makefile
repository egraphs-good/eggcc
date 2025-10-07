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


tiger: json2egraph tigermain

json2egraph:
	cd dag_in_context/src/tiger && g++ $(CPPFLAGS) json2egraph.cpp -o ../../../target/json2egraph

tigermain:
	cd dag_in_context/src/tiger && g++ $(CPPFLAGS) main.cpp -o ../../../target/tiger


# build the llvm runtime for bril
# if you edit the runtime crate, you must re-run this to rebuild rt.bc
runtime:
	bash runtime/install.sh

nightly:
	bash infra/nightly.sh "benchmarks/passing"
