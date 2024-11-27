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
- Open a new terminal (`source`ing alone may not work).
- Run `make runtime` to install the bril llvm runtime. If this fails, try running `cargo clean` in `runtime` and trying again.
- You may need to add LSystem to your path: `export LIBRARY_PATH="$LIBRARY_PATH:/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib"`
- Run the tests with `make` and ensure things build and run without errors.

## Installation- Linux
- Install llvm 18. For Ubuntu users, we have a script for installation: `./install_ubuntu.sh`
- Run `make runtime` to install the bril llvm runtime.
- Run the tests with `make` and ensure things build and run without errors.



## How to add a test
- Add a bril file under `tests/`.
- Run `cargo insta review` to confirm the output for the new file.


## How to run local nightly
- Run `bash infra/localnightly.sh <bril file or directory>`

To run the nightly server for an existing nightly, run `cd nightly/output && python3 -m http.server`.

## Troubleshooting
- On Mac running `make` might show this error: `Library not loaded: /opt/homebrew/opt/icu4c/lib/libicui18n.74.dylib`. To resolve this, run `brew upgrade`.
- On Mac, if you get a linker error complaining about not being able to find
  zstd, try adding zstd to your `LIBRARY_PATH` in `.zshrc` or `.zprofile`, e.g: `export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/opt/zstd/lib"`
