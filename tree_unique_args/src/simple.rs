#[test]
fn test_simple_bool_rules() -> crate::Result {
    let build = "
    (let id1 (Id (i64-fresh!)))
    (let id2 (Id (i64-fresh!)))
    (let a (Boolean id1 true))
    (let b (Boolean id2 false))
    (let lhs1 (Not (Or a b)))
    (let lhs2 (Not (And a b)))
    (let lhs3 (Not (Not a)))
  ";
    let check = "
    (check (= lhs1 (And (Not a) (Not b))))
    (check (= lhs2 (Or (Not a) (Not b))))
    (check (= lhs3 a))
    (extract lhs3)
    (extract a)
  ";
    crate::run_test(build, check)
}
