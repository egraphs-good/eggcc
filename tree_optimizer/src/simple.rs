#[test]
fn test_simple_bool_rules() -> Result<(), egglog::Error> {
    let build = "
    (let id (Id (i64-fresh!)))
    (let a (Boolean id true))
    (let b (Boolean id false))
    (let lhs1 (UOp (Not) (BOp (Or) a b)))
    (let lhs2 (UOp (Not)  (BOp (And) a b)))
    (let lhs3 (UOp (Not)  (UOp (Not)  a)))
  ";
    let check = "
    (check (= lhs1 (BOp (And) (UOp (Not) a) (UOp (Not)  b))))
    (check (= lhs2 (BOp (Or) (UOp (Not)  a) (UOp (Not) b))))
    (check (= lhs3 a))
  ";
    crate::run_test(build, check)
}
