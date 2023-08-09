# eggcc

## How to set up

- [Install rust](https://www.rust-lang.org/tools/install)
- Clone the repo (don't forget to `--recurse-submodules`!)
  ```
  git clone --recurse-submodules git@github.com:oflatt/eggcc.git
  ```
  - If you forget to pass `--recurse-submodules` to clone, you can either
    remove the directory and try again, or you can
    ```
    cd egcc
    git submodule update --init --recursive
    ```
- Run the tests with `cargo test` and ensure things build and run without errors.
