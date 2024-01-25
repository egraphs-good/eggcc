use std::iter;
use strum::IntoEnumIterator;

use crate::ir::{Constructor, Purpose, Sort};

fn find_invariant_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let br = "\n      ";
    let ruleset = " :ruleset always-run";

    match ctor {
        Constructor::Cons | Constructor::Nil | Constructor::Arg | Constructor::UnitExpr => None,
        Constructor::Call => Some(format!(
            "(rule ((find-inv-Expr loop expr) \
            {br} (= expr (Call f arg))) \
            {br}((find-inv-Expr loop arg)){ruleset})"
        )),
        Constructor::Get => Some(format!(
            "(rule ((find-inv-Expr loop expr) \
            {br} (= expr (Get tup i))) \
            {br}((find-inv-Expr loop tup)){ruleset})\n \
            (rule ((find-inv-Expr loop expr) \
            {br} (= expr (Get (Arg id) i)) \
            {br} (arg-inv loop i)) \
            {br}((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        _ => {
            let ctor_pattern = ctor.construct(|field| field.var());

            let find_inv_ctor = ctor
                .fields()
                .iter()
                .filter_map(|field| match field.purpose {
                    Purpose::Static(Sort::I64) | Purpose::Static(Sort::Bool) => {
                        Some("(set (is-inv-Expr loop expr) true)".to_string())
                    }
                    Purpose::Static(_)
                    | Purpose::CapturingId
                    | Purpose::CapturedExpr
                    | Purpose::ReferencingId => None,
                    Purpose::SubExpr | Purpose::SubListExpr => {
                        let var = field.var();
                        let sort = field.sort().name();
                        Some(format!("(find-inv-{sort} loop {var})"))
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            Some(format!(
                "(rule ((find-inv-Expr loop expr) \
                {br} (= expr {ctor_pattern})) \
                {br}({find_inv_ctor}){ruleset})"
            ))
        }
    }
}

fn is_invariant_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let br = "\n      ";
    let ruleset = " :ruleset always-run";
    let ctor_pattern = ctor.construct(|field| field.var());

    match ctor {
        // list handled in loop_invariant.egg
        // print, read, write are not invariant
        // assume Arg as whole is not invariant
        Constructor::Cons
        | Constructor::Nil
        | Constructor::UnitExpr
        | Constructor::Print
        | Constructor::Read
        | Constructor::Write
        | Constructor::Arg => None,
        Constructor::Get => Some(format!(
            "(rule ((find-inv-Expr loop expr) \
            {br} (= expr (Get tup i)) \
            {br} (= true (is-inv-Expr loop tup))) \
            {br}((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        _ => {
            let is_inv_ctor = ctor
                .fields()
                .iter()
                .filter_map(|field| match field.purpose {
                    Purpose::Static(_)
                    | Purpose::CapturingId
                    | Purpose::CapturedExpr
                    | Purpose::ReferencingId => None,
                    Purpose::SubExpr | Purpose::SubListExpr => {
                        let var = field.var();
                        let sort = field.sort().name();
                        Some(format!("(= true (is-inv-{sort} loop {var}))"))
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            let is_pure = match ctor {
                Constructor::Call | Constructor::Let | Constructor::Loop => {
                    format!("{br} (ExprIsPure expr)")
                }
                _ => String::new(),
            };
            Some(format!(
                "(rule ((find-inv-Expr loop expr) \
                {br} (= expr {ctor_pattern}) \
                {br} {is_inv_ctor} {is_pure}) \
                {br}((set (is-inv-Expr loop expr) true)){ruleset})"
            ))
        }
    }
}

pub(crate) fn rules() -> Vec<String> {
    iter::once(include_str!("loop_invariant.egg").to_string())
        .chain(Constructor::iter().filter_map(find_invariant_rule_for_ctor))
        .chain(Constructor::iter().filter_map(is_invariant_rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn loop_invariant_detection1() -> Result<(), egglog::Error> {
    let build = "
    (let id1 (Id (i64-fresh!)))
    (let id-outer (Id (i64-fresh!)))
    (let loop
        (Loop id1
            (All (Parallel) (Pair (Num id-outer 0) (Num id-outer 5)))
            (All (Sequential) (Pair
                ; pred
                (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
                ; output
                (All (Parallel) 
                    (Pair
                        (Get (Arg id1) 0)
                        (Sub (Get (Arg id1) 1) (Add (Num id1 1) (Get (Arg id1) 0))) ))))))
    ";

    let check = "
        (check (arg-inv loop 0))
        (fail (check (arg-inv loop 1)))
        (check (= true (is-inv-Expr loop (Get (Arg id1) 0))))
        (check (= false (is-inv-Expr loop (Get (Arg id1) 1))))
        (check (= true (is-inv-Expr loop (Add (Num id1 1) (Get (Arg id1) 0)))))
        (check (= false (is-inv-Expr loop (Sub (Get (Arg id1) 1) (Add (Num id1 1) (Get (Arg id1) 0))) )))
    ";

    crate::run_test(build, check)
}

#[test]
fn loop_invariant_detection2() -> Result<(), egglog::Error> {
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
        (check (arg-inv loop 1))
        (check (arg-inv loop 2))
        (check (arg-inv loop 3))
        (check (arg-inv loop 4))
        (fail (check (arg-inv loop 0)))
        (let l4 (list4 (Num id1 1) (Num id1 2) (Num id1 3) (Num id1 4)))
        (check (is-inv-ListExpr-helper loop l4 4))
        (check (= true (is-inv-ListExpr loop l4)))
        (check (= true (is-inv-Expr loop (Switch (Num id1 1) l4))))
        (check (= true (is-inv-Expr loop inv)))
        (check (= false (is-inv-Expr loop (Add (Get (Arg id1) 0) inv))))
        ;; a non exist expr should fail
        (fail (check (is-inv-Expr loop (Switch (Num id1 2) l4))))
    ";

    crate::run_test(build, check)
}
