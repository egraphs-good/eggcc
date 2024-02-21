#[cfg(test)]
use crate::{ast::*, egglog_test, interpreter::Value};

#[test]
fn test_list_util() -> crate::Result {
    let build = "
		(let list (Cons (Const (Int 0))
                  (Cons (Const (Int 1))
                  (Cons (Const (Int 2))
                  (Cons (Const (Int 3))
                  (Cons (Const (Int 4)) (Nil)))))))
		(let expr (Switch (Const (Int 1)) list))
	";
    let check = "
		(check (= (ListExpr-ith list 1) (Const (Int 1))))
        (check (= (ListExpr-ith list 4) (Const (Int 4))))
        (check (= (ListExpr-length list) 5))
	";
    egglog_test(
        build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn append_test() -> crate::Result {
    let build = "
        (let appended
            (Append
                (Cons (Const (Int 0)) (Cons (Const (Int 1)) (Nil)))
                (Const (Int 2))))
    ";

    let check = "
        (check (
            =
            (Cons (Const (Int 0)) (Cons (Const (Int 1)) (Cons (Const (Int 2)) (Nil))))
            appended
        ))
    ";
    egglog_test(
        build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn test_tuple_ith() -> crate::Result {
    let build = "
    (let tup (Concat par 
                  (Concat sequ (Single (Const (Int 0))) (Single (Const (Int 1))))
                  (Concat sequ (Single (Const (Int 2))) (Single (Const (Int 3))))))
    
    ;; with print
    (let tup2 (Concat par
                (Concat par 
                    (Uop (Print) (Const (Int 0))) 
                    (Concat par (Single (Const (Int 1))) 
                                (Concat par (Uop (Print) (Const (Int 0))) 
                                            (Single (Const (Int 2))))))
                (Concat par (Single (Const (Int 3))) (Uop (Print) (Const (Int 0))))))
    ";

    let check = "
    (check (= (tuple-ith tup 0) (Const (Int 0))))
    (check (= (tuple-ith tup 1) (Const (Int 1))))
    (check (= (tuple-ith tup 2) (Const (Int 2))))
    (check (= (tuple-ith tup 3) (Const (Int 3))))
    (check (= 4 (tuple-length tup)))
    (fail (check (tuple-ith tup 4)))

    (check (= (tuple-ith tup2 0) (Const (Int 1))))
    (check (= (tuple-ith tup2 1) (Const (Int 2))))
    (check (= (tuple-ith tup2 2) (Const (Int 3))))
    (check (= 3 (tuple-length tup2)))
    (fail (check (tuple-ith tup2 3)))
    ";
    egglog_test(
        build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn test_tuple_remove_fst() -> crate::Result {
    let tup1 = parallel!(int(1), int(2), int(3), int(4));
    let tup2 = concat_par(
        concat_par(tprint(int(0)), tprint(int(0))),
        concat_par(single(int(1)), single(int(2))),
    );
    let build = format!(
        "(let tup1 {}) 
        (let tup2 {}) 
        (tuple-remove-fst tup1) 
        (tuple-remove-fst tup2)",
        tup1, tup2
    );
    let check = "(check (= (tuple-remove-fst tup1) 
                    (Concat (Parallel) (Concat (Parallel) 
                            (Single (Const (Int 2))) 
                            (Single (Const (Int 3)))) 
                            (Single (Const (Int 4))))))
        ;; it should only remove the first one that is not Tuple (TNil) type
        (check (= (tuple-remove-fst tup2) 
                (Concat (Parallel) (Concat (Parallel) 
                    (Uop (Print) (Const (Int 0))) 
                    (Uop (Print) (Const (Int 0)))) 
                    (Single (Const (Int 2))))))";
    egglog_test(
        &build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn test_do_while_output_ith() -> crate::Result {
    let output_ty = tuplet!(intt(), intt(), intt(), intt());
    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4)),
        parallel!(
            less_than(getarg(0), getarg(3)),
            add(getarg(0), sub(getarg(2), getarg(1))),
            getarg(1),
            getarg(2),
            getarg(3)
        ),
    )
    .with_arg_types(emptyt(), output_ty.clone());

    let build = format!("(let loop {})", my_loop);
    let check = str::replace(
        "(let ty *)
        (check (= pred (tuple-ith pred_out 0)))
        (check (= pred (DoWhile-pred loop)))
        (check (= (DoWhile-outputs-ith loop 0) (Bop (Add) (Get (Arg ty) 0) (Bop (Sub) (Get (Arg ty) 2) (Get (Arg ty) 1)))))       
        (check (= (DoWhile-outputs-ith loop 1) (Get (Arg ty ) 1)))
        (check (= (DoWhile-outputs-ith loop 2) (Get (Arg ty ) 2)))
        (check (= (DoWhile-outputs-ith loop 3) (Get (Arg ty ) 3)))", "*", output_ty.to_string().as_str());

    egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
