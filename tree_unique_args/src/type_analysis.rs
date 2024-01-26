#[test]
fn simple_types() -> Result<(), egglog::Error> {
    let build = "
        (let id1 (Id (i64-fresh!)))
        (let id2 (Id (i64-fresh!)))
        (let n (Add (Num id1 1) (Num id2 2)))
        (let m (Mul n n))
        (let s (Sub n m))
        (let x (LessThan m n))
        (let y (Not x))
        (let z (And x (Or y y)))
        (HasTypeDemand s)
        (HasTypeDemand z)
        ";
    let check = "
    (run-schedule (saturate type-analysis))

    (check (HasType n (IntT)))
    (check (HasType m (IntT)))
    (check (HasType s (IntT)))
    (check (HasType x (BoolT)))
    (check (HasType y (BoolT)))
    (check (HasType z (BoolT)))
    ";
    crate::run_test(build, check)
}

#[test]
fn switch_boolean() -> Result<(), egglog::Error> {
    let build = "
  (let b1 (Boolean (Id (i64-fresh!)) true))
  (let n1 (Num (Id (i64-fresh!)) 1))
  (let n2 (Num (Id (i64-fresh!)) 3))
  (let switch
    (Switch (Not (LessThan n1 n2))
            (Cons (Add n1 n1) (Cons (Sub n1 n2) (Nil)))))
  (let wrong_switch
    (Switch b1 (Cons n1 (Cons n2 (Cons n1 (Nil))))))
  (HasTypeDemand switch)
  (HasTypeDemand wrong_switch)
  ";
    let check = "
  (run-schedule (saturate type-analysis))

  (check (HasType switch (IntT)))
  (fail (check (HasType wrong_switch ty))) ; should not be able to type a boolean swith with 3 cases
  ";
    crate::run_test(build, check)
}

#[test]
fn switch_int() -> Result<(), egglog::Error> {
    let build = "
  (let n1 (Num (Id (i64-fresh!)) 1))
  (let n2 (Num (Id (i64-fresh!)) 2))
  (let n3 (Num (Id (i64-fresh!)) 3))
  (let n4 (Num (Id (i64-fresh!)) 4))
  (let s1
    (Switch n1
            (Cons (Add n1 n1) (Cons (Sub n1 n2) (Nil)))))
  (let s2
    (Switch (Mul n1 n2) (Cons (LessThan n3 n4) (Nil))))
  (let s3
    (Switch (Sub n2 n2) (Cons (Print n1) (Cons (Print n4) (Cons (Print n3) (Nil))))))  
  (HasTypeDemand s1)
  (HasTypeDemand s2)
  (HasTypeDemand s3)
  ";
    let check = "
  (run-schedule (saturate type-analysis))

  (check (HasType s1 (IntT)))
  (check (HasType s2 (BoolT)))
  (check (HasType s3 (UnitT)))
  ";
    crate::run_test(build, check)
}

#[test]
fn tuple() -> Result<(), egglog::Error> {
    let build = "
  (let n (Add (Num (Id (i64-fresh!)) 1) (Num (Id (i64-fresh!)) 2)))
        (let m (Mul n n))
        (let s (Sub n m))
        (let x (LessThan m n))
        (let y (Not x))
        (let z (And x (Or y y)))
  
  (let tup1 (All (Sequential) (Nil)))
  (let tup2 (All (Sequential) (Cons z (Nil))))
  (let tup3 (All (Parallel) (Cons x (Cons m (Nil)))))
  (let tup4 (All (Parallel) (Cons tup2 (Cons tup3 (Nil)))))
  (HasTypeDemand tup1)
  (HasTypeDemand tup2)
  (HasTypeDemand tup3)
  (HasTypeDemand tup4)

  (let get1 (Get tup3 0))
  (let get2 (Get tup3 1))
  (let get3 (Get (Get tup4 1) 1))
  (HasTypeDemand get1)
  (HasTypeDemand get2)
  (HasTypeDemand get3)
  ";
    let check = "
  (run-schedule (saturate type-analysis))
  (check (HasType tup1 (TupleT (TNil))))
  (check (HasType tup2 (TupleT (TCons (BoolT) (TNil)))))
  (check (HasType tup3 (TupleT (TCons (BoolT) (TCons (IntT) (TNil))))))
  (check (HasType tup4
    (TupleT (TCons (TupleT (TCons (BoolT) (TNil)))
    (TCons (TupleT (TCons (BoolT) (TCons (IntT) (TNil))))
          (TNil))))))

  
  (check (HasType get1 (BoolT)))
  (check (HasType get2 (IntT)))
  (check (HasType get3 (IntT)))
  ";
    crate::run_test(build, check)
}

#[test]
fn lets() -> Result<(), egglog::Error> {
    let build = "
    (let let-id (Id (i64-fresh!)))
    (let outer-ctx (Id (i64-fresh!)))
    (let l (Let let-id (Num outer-ctx 5) (Add (Arg let-id) (Arg let-id))))
    (HasTypeDemand l)
    (let outer (Id (i64-fresh!)))
    (let inner (Id (i64-fresh!)))
    (let ctx (Id (i64-fresh!)))
    (let nested
      (Let outer (Num ctx 3)
                 (Let inner (All (Parallel) (Cons (Arg outer) (Cons (Num outer 2) (Nil))))
                            (Add (Get (Arg inner) 0) (Get (Arg inner) 1)))))
    (HasTypeDemand nested)
  ";
    let check = "
    (run-schedule (saturate type-analysis))
    (check (HasType l (IntT)))
    (check (HasType nested (IntT)))
  ";
    crate::run_test(build, check)
}

#[test]
fn loops() -> Result<(), egglog::Error> {
    let build = "
  (let ctx (Id 0))
  (let loop-id (Id 1))
  (let l (Loop loop-id (Num ctx 1)
    (All (Sequential)
         (Cons (LessThan (Num loop-id 2) (Num loop-id 3))
               (Cons (Switch (Boolean loop-id true)
                             (Cons (Num loop-id 4) (Cons (Num loop-id 5) (Nil))))
                     (Nil))))))
  (HasTypeDemand l)
  ";
    let check = "
  (run-schedule (saturate type-analysis))
  (check (HasType l (IntT)))
  ";
    crate::run_test(build, check)
}

#[test]
#[should_panic]
fn loop_pred_boolean() {
    let build = "
  (let ctx (Id 0))
  (let loop-id (Id 1))
  (let l (Loop loop-id (Num ctx 1)
    (All (Sequential)
        (Cons (Add (Num loop-id 2) (Num loop-id 3))
              (Cons (Switch (Boolean loop-id true)
                            (Cons (Num loop-id 4) (Cons (Num loop-id 5) (Nil))))
                    (Nil))))))
  (HasTypeDemand l)
  (run-schedule (saturate type-analysis))";
    let check = "";

    let _ = crate::run_test(build, check);
}

#[test]
#[should_panic]
fn loop_args1() {
    let build = "
  (let ctx (Id 0))
  (let loop-id (Id 1))
  (let l (Loop loop-id (Num ctx 1) (All (Sequential) (Nil))))
  (HasTypeDemand l)
  (run-schedule (saturate type-analysis))";
    let check = "";

    let _ = crate::run_test(build, check);
}

#[test]
#[should_panic]
fn loop_args3() {
    let build = "
  (let ctx (Id 0))
  (let loop-id (Id 1))
  (let l (Loop loop-id (Num ctx 1)
    (All (Sequential)
        (Cons (LessThan (Num loop-id 2) (Num loop-id 3))
              (Cons (Switch (Boolean loop-id true)
                            (Cons (Num loop-id 4) (Cons (Num loop-id 5) (Nil))))
                    (Cons (Num loop-id 1) (Nil)))))))
  (HasTypeDemand l)
  (run-schedule (saturate type-analysis))";
    let check = "";

    let _ = crate::run_test(build, check);
}
