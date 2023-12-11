# 507 project instructions

To run our e-interpreter, which tests the correctness of our compiler, run the following command:

```
cargo test --release
```

Two tests are expected to fail, and the rest pass.
The failing tests are `test_should_fail_union_arg_0_3` and `test_should_fail_union_arg_1_9`.
These tests are expected to fail because our intepreter raises an (unfortunately) un-catchable error when unsoundness is detected.

# eggcc

## How to set up

- [Install rust](https://www.rust-lang.org/tools/install)
- Clone the repo
- Install the `insta` command for `cargo`
  ```
  cargo install cargo-insta
  ```
- Run the tests with `make` and ensure things build and run without errors. You may need to install `graphviz`.

## How to add a test
- Add a bril file under `tests/`.
- Run `cargo insta review` to confirm the output for the new file.
