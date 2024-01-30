use crate::{expr::Expr, ir::{Constructor, Purpose}};
use std::iter;

fn ast_size_for_ctor(ctor: Constructor) -> String {
    let ctor_pattern = ctor.construct(|field| field.var());
    let ruleset = " :ruleset always-run";
    match ctor {
        // List itself don't count size
        Constructor::Nil =>  format!("(rule ({ctor_pattern}) ((set (ListExpr-size {ctor_pattern}) 0)) {ruleset})"),
        Constructor::Cons => format!("(rule ((= list (Cons expr xs)) (= a (Expr-size expr)) (= b (ListExpr-size xs))) ((set (ListExpr-size list) (+ a b))){ruleset})"), 
        // let Get and All's size = children's size (I prefer not +1 here)
        Constructor::Expr(Expr::Get(..)) => format!("(rule ((= expr (Get tup i)) (= n (Expr-size tup))) ((set (Expr-size expr) n)) {ruleset})"),
        Constructor::Expr(Expr::All(..)) => format!("(rule ((= expr (All id ord list)) (= n (ListExpr-size list))) ((set (Expr-size expr) n)) {ruleset})"),
        Constructor::Expr(Expr::Branch(..)) => format!("
(rule ((= expr (Branch id child))
       (= n (Expr-size child)))
      ((set (Expr-size expr) n)) {ruleset})
        "),
        _ => {
            let field_pattern = ctor.filter_map_fields(|field| {
                let sort = field.sort().name();
                let var = field.var();
                match field.purpose {
                    Purpose::CapturedExpr
                    | Purpose::SubExpr
                    | Purpose::SubListExpr =>
                        Some(format!("({sort}-size {var})")),
                    _ => None
                }
            });

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
        (let t (All id (Sequential) list))
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
fn ast_size_test() -> Result<(), egglog::Error> {
    let build = "
    (let inv 
        (BOp (Sub) (Get (Arg shared) 4)
            (BOp (Mul) (Get (Arg shared) 2) 
                (Switch (Num shared 1)
                  (list4 (Branch shared (Num shared 1))
                         (Branch shared (Num shared 2))
                         (Branch shared (Num shared 3))
                         (Branch shared (Num shared 4)))
                )
            )
        ))
    
    (let loop
        (Loop shared
            (All shared (Parallel) (list5 (Num shared 0)
                                    (Num shared 1)
                                    (Num shared 2)
                                    (Num shared 3)
                                    (Num shared 4)))
            (All shared (Sequential)
                (Pair
                ; pred
                (BOp (LessThan) (Get (Arg shared) 0) (Get (Arg shared) 4))
                ; output
                (All shared (Parallel) 
                    (list5
                        (BOp (Add) (Get (Arg shared) 0) 
                            inv
                        )
                        (Get (Arg shared) 1)
                        (Get (Arg shared) 2)
                        (Get (Arg shared) 3)
                        (Get (Arg shared) 4) ))))))
    ";

    let check = "
       (check (= 10 (Expr-size inv)))
       (check (= 25 (Expr-size loop)))
    ";

    crate::run_test(build, check)
}
