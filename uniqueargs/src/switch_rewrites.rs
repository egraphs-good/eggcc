pub use crate::*;

#[test]
fn switch_rewrite_and() -> Result {
    let build = "
        (let test_a_id (id (i64-fresh!)))
        (let test_a (Switch test_a_id
                            (TODO)
                            (TODO)
                            (TODO)))

        (let test_b_id (id (i64-fresh!)))
        (let test_b (Switch test_b_id
                            (TODO)
                            (TODO)
                            (TODO)))
    ";
    let check = "(check (= test_a test_b))";
    run_test(build, check)
}

#[test]
fn switch_rewrite_or() -> Result {
    todo!()
}
