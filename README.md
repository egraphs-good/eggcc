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
- Add llvm to PATH in your `.zshrc` file: `export PATH="/opt/homebrew/Cellar/llvm@18/18.1.8/bin/:$PATH"`
- Install cbc using `brew tap coin-or-tools/coinor` and `brew install coin-or-tools/coinor/cbc`
- Open a new terminal (`source`ing alone may not work).
- Run `make runtime` to install the bril llvm runtime. If this fails, try running `cargo clean` in `runtime` and trying again.
- You may need to add LSystem to your path: `export LIBRARY_PATH="$LIBRARY_PATH:/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib"`
- Run the tests with `make` and ensure things build and run without errors.
- On Mac running `make` might show this error: `Library not loaded: /opt/homebrew/opt/icu4c/lib/libicui18n.74.dylib`. To resolve this, run `brew upgrade` and `brew cleanup`.
  - Warning: this updates all your brew packages
- On Mac, if you get a linker error complaining about not being able to find
  zstd, try adding zstd to your `LIBRARY_PATH` in `.zshrc` or `.zprofile`, e.g: `export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/opt/zstd/lib"`


## Installation- Linux
- Install llvm 18. For Ubuntu users, we have a script for installation: `./install_ubuntu.sh`
- Install cbc with `sudo apt install coinor-libcbc-dev coinor-cbc`
  (`coinor-libcbc-dev` is the CBC library, `coinor-cbc` is the `cbc` binary the extractor calls)
- Run `make runtime` to install the bril llvm runtime.
- Run the tests with `make` and ensure things build and run without errors.



## How to add a test
- Add a bril file under `tests/`.
- Run `cargo insta review` to confirm the output for the new file.


## How to run local nightly

The nightly builds `eggcc`, profiles the benchmarks, and generates the paper's graphs.
Beyond the build dependencies above, it needs:

- **Python packages** for graph generation: `pip install -r infra/requirements.txt`
  (matplotlib, numpy, pandas).
- **graphviz** (`dot`) for the control-flow graphs — see Installation above.
- **tokei** for the line-count table. `bash infra/nightly.sh` installs it automatically,
  but `--local` mode skips that, so install it yourself first:
  `cargo install tokei --version 13.0.0 --locked`.

Then run:
- `bash infra/localnightly.sh <bril file or directory>`

To run the nightly server for an existing nightly, run `cd nightly/output && python3 -m http.server`.

Gurobi is optional — see below. Without it the nightly uses CBC and produces the
non-Gurobi subset of graphs.

## Gurobi (optional)

Gurobi is optional. The nightly **auto-detects** whether a licensed `gurobi_cl` is
installed:

- **Without Gurobi** it times ILP with the free CBC solver and produces every graph that
  does not fundamentally need Gurobi (the tiger, statewalk, CBC, and normalized
  performance charts). The Gurobi-only plots (`egraph-size-vs-ILP-time`, the
  `heatmap-ilp-time-*` heatmaps, `ilp-encoding-size-vs-solve-time`,
  `extraction-time-histogram`, the peggy comparison) and `nightlymacros.tex` are skipped.
- **With Gurobi** it additionally runs the Gurobi treatments and generates all graphs.

Nightly flags (`infra/nightly.sh` / `infra/localnightly.sh`):

- (no flag) — auto-detect Gurobi.
- `--use-gurobi` — require Gurobi (errors out if it is unavailable).
- `--no-gurobi` — force the CBC-only subset even if Gurobi is available.
- `--paper` — production run; implies `--use-gurobi`.

### Providing a Gurobi license

1. Install Gurobi and put `gurobi_cl` on your PATH
   (see <https://www.gurobi.com/downloads/>).
2. Install your license:
   ```
   make gurobi-setup LICENSE=path/to/gurobi.lic
   # or: bash infra/setup_gurobi.sh path/to/gurobi.lic
   ```
   This copies the license to `~/gurobi.lic` (Gurobi's default location) and verifies it
   by solving a trivial model. Run `make gurobi-setup` with no `LICENSE` to just verify an
   already-installed license.
