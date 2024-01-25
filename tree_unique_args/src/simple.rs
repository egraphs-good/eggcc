#[test]
fn test_simple_bool_rules() -> Result<(), egglog::Error> {
    let build = "
    (let id (Id (i64-fresh!)))
    (let a (Boolean id true))
    (let b (Boolean id false))
    (let lhs1 (Not (Or a b)))
    (let lhs2 (Not (And a b)))
    (let lhs3 (Not (Not a)))
  ";
    let check = "
    (check (= lhs1 (And (Not a) (Not b))))
    (check (= lhs2 (Or (Not a) (Not b))))
    (check (= lhs3 a))
  ";
    crate::run_test(build, check)
}
