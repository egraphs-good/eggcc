#![allow(clippy::useless_format)]

use main_error::MainError;
use tree_unique_args::run_test;

// Might be useful for typechecking?
fn main() -> std::result::Result<(), MainError> {
    run_test("", "").map_err(|e| e.into())
}
