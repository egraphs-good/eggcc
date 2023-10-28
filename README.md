# eggcc

## How to set up

- [Install rust](https://www.rust-lang.org/tools/install)
- Clone the repo
- Install the `insta` command for `cargo`
  ```
  cargo install cargo-insta
  ```
- Run the tests with `cargo test` and ensure things build and run without errors.

## How to add a test
- Add a bril file under `tests/`.
- Run `cargo insta review` to confirm the output for the new file.
