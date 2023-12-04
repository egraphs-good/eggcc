.PHONY: test nits docs nightly test-clean uniqueargs

all: test nits docs uniqueargs

test:
	cargo insta test --release --unreferenced=reject

uniqueargs:
	cargo test --manifest-path uniqueargs/Cargo.toml --release

test-clean:
	cargo insta test --release --unreferenced=delete

nits:
	@rustup component add clippy
	cargo clippy --tests -- -D warnings
	@rustup component add rustfmt
	cargo fmt --check

nightly:
	bash infra/nightly.sh
