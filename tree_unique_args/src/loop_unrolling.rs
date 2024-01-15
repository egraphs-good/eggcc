#[test]
fn test_unroll_once() -> Result<(), egglog::Error> {
    let build = "
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let loop
  (Loop id1
    (Num id-outer 2)
    (All (Parallel)
     (Pair (LessThan (Num id1 3) (Num id1 3))
           (Print (Num id1 4))))))
  ";
    let expected = "(Let newid1
    (Num id-outer 2)
    (Let newid2
      (All (Parallel)
           (Pair
              (LessThan (Num newid1 3) (Num newid1 3))
              (Print (Num newid1 4))))
      (Switch
        (Get (Arg newid2) 0)
        (Pair
          (Loop newid3
            (All (Parallel) (Single (Get (Arg newid2) 1)))
            (All (Parallel)
              (Pair (LessThan (Num newid3 3) (Num newid3 3))
                    (Print (Num newid3 4)))))
         (All (Parallel) (Single (Get (Arg newid2) 1)))))))";
    let check = format!(
        "
;; first check that loop was desugared
;; to a let loop
(check 
  (= loop
     {expected}))
"
    );
    crate::run_test(build, &check)
}
