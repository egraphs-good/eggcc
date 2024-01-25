use crate::ir::{Constructor, Purpose};
use std::iter;
use strum::IntoEnumIterator;

fn ast_size_for_ctor(ctor: Constructor) -> String {
    let ctor_pattern = ctor.construct(|field| field.var());
    let ruleset = " :ruleset always-run";
    match ctor {
        // List itself don't count size
        Constructor::Nil =>  format!("(rule ({ctor_pattern}) ((set (ListExpr-size {ctor_pattern}) 0)) {ruleset})"),
        Constructor::Cons => format!("(rule ((= list (Cons expr xs)) (= a (Expr-size expr)) (= b (ListExpr-size xs))) ((set (ListExpr-size list) (+ a b))){ruleset})"), 
        // let Get and All's size = children's size (I prefer not +1 here)
        Constructor::Get => format!("(rule ((= expr (Get tup i)) (= n (Expr-size tup))) ((set (Expr-size expr) n)) {ruleset})"),
        Constructor::All => format!("(rule ((= expr (All ord list)) (= n (ListExpr-size list))) ((set (Expr-size expr) n)) {ruleset})"),
        _ => {
            let field_pattern = ctor.fields().iter().filter_map(|field| {
                let sort = field.sort().name();
                let var = field.var();
                match field.purpose {
                    Purpose::CapturedExpr
                    | Purpose::SubExpr
                    | Purpose::SubListExpr =>
                        Some(format!("({sort}-size {var})")),
                    _ => None
                }
            }).collect::<Vec<_>>();

            let len = field_pattern.len();
            let result_str = field_pattern.join(" ");

            match len {
                // Num, Bool Arg, UnitExpr for 0
                0 => format!("(rule ((= expr {ctor_pattern})) ((set (Expr-size expr) 1)) {ruleset})"),
                1 => format!("(rule ((= expr {ctor_pattern}) (= n {result_str})) ((set (Expr-size expr) (+ 1 n))){ruleset})"),
                2 => format!("(rule ((= expr {ctor_pattern}) (= sum (+ {result_str}))) ((set (Expr-size expr) (+ 1 sum))){ruleset})"),
                _ => panic!("Unimplemented") // we don't have ast take three Expr
            }
        },
    }
}

pub(crate) fn rules() -> Vec<String> {
    iter::once(include_str!("util.egg").to_string())
        .chain(Constructor::iter().map(ast_size_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_list_util() -> Result<(), egglog::Error> {
    let build = &*"
        (let id (Id 1))
        (let list (Cons (Num id 0) (Cons (Num id 1) (Cons (Num id 2) (Cons (Num id 3) (Cons (Num id 4) (Nil)))))))
        (let t (All (Sequential) list))
    ".to_string();
    let check = &*"
        (check (= (ListExpr-ith list 1) (Num id 1)))
        (check (= (ListExpr-ith list 4) (Num id 4)))
        (check (= (ListExpr-length list) 5))
        
    "
    .to_string();
    crate::run_test(build, check)
}

#[test]
fn append_test() -> Result<(), egglog::Error> {
    let build = "
        (let id (Id (i64-fresh!)))
        (let appended
            (Append
                (Cons (Num id 0) (Cons (Num id 1) (Nil)))
                (Num id 2)))
    ";

    let check = "
        (check (
            =
            (Cons (Num id 0) (Cons (Num id 1) (Cons (Num id 2) (Nil))))
            appended
        ))
    ";

    crate::run_test(build, check)
}

#[test]
fn get_loop_output_ith_test() -> Result<(), egglog::Error> {
    let build = "
    (let id1 (Id (i64-fresh!)))
    (let id-outer (Id (i64-fresh!)))
    (let loop1
        (Loop id1
            (All (Parallel) (Pair (Arg id-outer) (Num id-outer 0)))
            (All (Sequential) (Pair
                ; pred
                (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
                ; output
                (All (Parallel) (Pair
                    (Add (Get (Arg id1) 0) (Num id1 1))
                    (Sub (Get (Arg id1) 1) (Num id1 1))))))))
    (let out0 (Add (Get (Arg id1) 0) (Num id1 1)))
    (let out1 (Sub (Get (Arg id1) 1) (Num id1 1)))
    ";

    let check = "
        (check (
            =
            (get-loop-outputs-ith loop 0)
            out0
        ))
        (check (
            =
            (get-loop-outputs-ith loop 1)
            out1
        ))
    ";

    crate::run_test(build, check)
}

#[test]
fn ast_size_test() -> Result<(), egglog::Error> {
    let build = "
    (let id1 (Id (i64-fresh!)))
    (let id-outer (Id (i64-fresh!)))
    (let inv 
        (Sub (Get (Arg id1) 4)
            (Mul (Get (Arg id1) 2) 
                (Switch (Num id1 1) (list4 (Num id1 1)
                                        (Num id1 2)
                                        (Num id1 3)
                                        (Num id1 4))
                )
            )
        ))
    
    (let loop
        (Loop id1
            (All (Parallel) (list5 (Num id-outer 0)
                                    (Num id-outer 1)
                                    (Num id-outer 2)
                                    (Num id-outer 3)
                                    (Num id-outer 4)))
            (All (Sequential) (Pair
                ; pred
                (LessThan (Get (Arg id1) 0) (Get (Arg id1) 4))
                ; output
                (All (Parallel) 
                    (list5
                        (Add (Get (Arg id1) 0) 
                            inv
                        )
                        (Get (Arg id1) 1)
                        (Get (Arg id1) 2)
                        (Get (Arg id1) 3)
                        (Get (Arg id1) 4) ))))))
    ";

    let check = "
       (check (= 10 (Expr-size inv)))
       (check (= 25 (Expr-size loop)))
    ";

    crate::run_test(build, check)
}
