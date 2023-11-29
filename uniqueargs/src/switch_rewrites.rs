pub use crate::*;

#[test]
fn switch_rewrite_and() -> Result {
    let a = "(Num global-id 1)";
    let b = "(Num global-id 2)";
    run_test(a, b)
}

#[test]
fn switch_rewrite_or() -> Result {
    todo!()
}
