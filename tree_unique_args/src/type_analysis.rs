#[test]
fn simple_types() -> Result<(), egglog::Error> {
    let build = "
        (let id (Id (i64-fresh!)))
        (let n (Add (Num id 1) (Num id 2)))
        (let m (Mul n n))
        (let s (Sub n m))
        (let x (LessThan m n))
        (let y (Not x))
        (let z (And x (Or y y)))
        (HasTypeDemand id s)
        (HasTypeDemand id z)
        ";
    let check = "
    (run-schedule (saturate type-analysis))

    (check (HasType id n (IntT)))
    (check (HasType id m (IntT)))
    (check (HasType id s (IntT)))
    (check (HasType id x (BoolT)))
    (check (HasType id y (BoolT)))
    (check (HasType id z (BoolT)))
    ";
    crate::run_test(build, check)
}

#[test]
fn switch_boolean() -> Result<(), egglog::Error> {
    let build = "
  (let id (Id (i64-fresh!)))
  (let b1 (Boolean id true))
  (let n1 (Num id 1))
  (let n2 (Num id 3))
  (let switch
    (Switch (Not (LessThan n1 n2))
            (Cons (Add n1 n1) (Cons (Sub n1 n2) (Nil)))))
  (let wrong_switch
    (Switch b1 (Cons n1 (Cons n2 (Cons n1 (Nil))))))
  (HasTypeDemand id switch)
  (HasTypeDemand id wrong_switch)
  ";
    let check = "
  (run-schedule (saturate type-analysis))

  (check (HasType id switch (IntT)))
  (fail (check (HasType id wrong_switch ty))) ; should not be able to type a boolean swith with 3 cases
  ";
    crate::run_test(build, check)
}

#[test]
fn switch_int() -> Result<(), egglog::Error> {
    let build = "
  (let id (Id (i64-fresh!)))
  (let n1 (Num id 1))
  (let n2 (Num id 2))
  (let n3 (Num id 3))
  (let n4 (Num id 4))
  (let s1
    (Switch n1
            (Cons (Add n1 n1) (Cons (Sub n1 n2) (Nil)))))
  (let s2
    (Switch (Mul n1 n2) (Cons (LessThan n3 n4) (Nil))))
  (let s3
    (Switch (Sub n2 n2) (Cons (Print n1) (Cons (Print n4) (Cons (Print n3) (Nil))))))  
  (HasTypeDemand id s1)
  (HasTypeDemand id s2)
  (HasTypeDemand id s3)
  ";
    let check = "
  (run-schedule (saturate type-analysis))

  (check (HasType id s1 (IntT)))
  (check (HasType id s2 (BoolT)))
  (check (HasType id s3 (UnitT)))
  ";
    crate::run_test(build, check)
}

#[test]
fn tuple() -> Result<(), egglog::Error> {
    let build = "
  (let id (Id (i64-fresh!)))
  (let n (Add (Num id 1) (Num id 2)))
        (let m (Mul n n))
        (let s (Sub n m))
        (let x (LessThan m n))
        (let y (Not x))
        (let z (And x (Or y y)))
  
  (let tup1 (All (Sequential) (Nil)))
  (let tup2 (All (Sequential) (Cons z (Nil))))
  (let tup3 (All (Parallel) (Cons x (Cons m (Nil)))))
  (let tup4 (All (Parallel) (Cons tup2 (Cons tup3 (Nil)))))
  (HasTypeDemand id tup1)
  (HasTypeDemand id tup2)
  (HasTypeDemand id tup3)
  (HasTypeDemand id tup4)

  (let get1 (Get tup3 0))
  (let get2 (Get tup3 1))
  (let get3 (Get (Get tup4 1) 1))
  (HasTypeDemand id get1)
  (HasTypeDemand id get2)
  (HasTypeDemand id get3)
  ";
    let check = "
  (run-schedule (saturate type-analysis))
  (check (HasType id tup1 (TupleT (TNil))))
  (check (HasType id tup2 (TupleT (TCons (BoolT) (TNil)))))
  (check (HasType id tup3 (TupleT (TCons (BoolT) (TCons (IntT) (TNil))))))
  (check (HasType id tup4
    (TupleT (TCons (TupleT (TCons (BoolT) (TNil)))
    (TCons (TupleT (TCons (BoolT) (TCons (IntT) (TNil))))
          (TNil))))))

  
  (check (HasType id get1 (BoolT)))
  (check (HasType id get2 (IntT)))
  (check (HasType id get3 (IntT)))
  
  ";
    crate::run_test(build, check)
}

#[test]
fn lets() -> Result<(), egglog::Error> {
    let build = "
    (let let-id (Id (i64-fresh!)))
    (let outer-ctx (Id (i64-fresh!)))
    (let l (Let let-id (Num outer-ctx 5) (Add (Arg let-id) (Arg let-id))))

    (HasTypeDemand outer-ctx l)

    (let outer (Id (i64-fresh!)))
    (let inner (Id (i64-fresh!)))
    (let ctx (Id (i64-fresh!)))
    (let nested
      (Let outer (Num ctx 3)
                 (Let inner (All (Parallel) (Cons (Arg outer) (Cons (Num outer 2) (Nil))))
                            (Add (Get (Arg inner) 0) (Get (Arg inner) 1)))))
    (HasTypeDemand ctx nested)
  ";
    let check = "
    (run-schedule (saturate type-analysis))
    (check (HasType outer-ctx l (IntT)))

    (check (HasType ctx nested (IntT)))
  ";
    crate::run_test(build, check)
}
