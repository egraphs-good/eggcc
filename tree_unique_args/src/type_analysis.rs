#[test]
fn type_analysis() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
        (let id1 (Id (i64-fresh!)))
        (let id2 (Id (i64-fresh!)))
        (let n (Add (Num id1 1) (Num id2 2)))
        (let m (Mul n n))
        (let x (LessThan m n))
        (HasTypeDemand x)
        "
    );
    let check = "
    (run-schedule (saturate type-analysis))

    (check (HasType n (IntT)))
    (check (HasType m (IntT)))
    (check (HasType x (BoolT)))
    ";
    crate::run_test(build, check)
}

#[test]
fn switch() -> Result<(), egglog::Error> {
  let build =
  "
  (let b1 (Boolean (Id (i64-fresh!)) true))
  (let n1 (Num (Id (i64-fresh!)) 1))
  (let n2 (Num (Id (i64-fresh!)) 3))
  (let switch
    (Switch (Not (LessThan n1 n2))
            (Cons (Add n1 n1) (Cons (Sub n1 n2) (Cons (Mul n2 n2) (Nil))))))
  (HasTypeDemand switch)
  ";
  let check =
  "
  (run-schedule (saturate type-analysis))

  (check (HasType switch (IntT)))
  ";
  crate::run_test(build, check)
}
