# eggcc

## Installation
- [Install rust](https://www.rust-lang.org/tools/install)
- Clone the repo
- Install the `insta` command for `cargo`
  ```
  cargo install cargo-insta
  ```
- Install `graphviz` on your system. You'll need a `dot` executable in your path.

## Installation- Mac
- Install llvm 18 with `brew install llvm@18`  
  - If you get an error with this step, try upgrading MacOS to at least Sonoma (14)
- Add llvm to PATH in your `.zshrc` file: `export PATH="/opt/homebrew/Cellar/llvm/18.1.5/bin/:$PATH"`
- Run `make runtime` to install the bril llvm runtime.
- You may need to add LSystem to your path: `export LIBRARY_PATH="$LIBRARY_PATH:/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib"`
- Run the tests with `make` and ensure things build and run without errors.


## Installation- Linux
- Run `make runtime` to install the bril llvm runtime.
- Run the tests with `make` and ensure things build and run without errors.



## How to add a test
- Add a bril file under `tests/`.
- Run `cargo insta review` to confirm the output for the new file.
