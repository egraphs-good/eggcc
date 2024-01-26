#[test]
#[should_panic]
fn empty_all_should_panic() {
    let build = "(ExprIsValid (All (Parallel) (Nil)))";
    let check = "";
    let _ = crate::run_test(build, check);
}
