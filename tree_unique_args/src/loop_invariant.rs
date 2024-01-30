use crate::ir::{Constructor, Purpose};
use std::iter;
use strum::IntoEnumIterator;

fn is_inv_base_case_for_ctor(ctor: Constructor) -> Option<String> {
    let br = "\n      ";
    let ruleset = " :ruleset always-run";

    match ctor {
        Constructor::Get => Some(format!(
            "(rule ((BodyContainsExpr loop_id expr) \
            {br} (Loop loop_id in out) \
            {br} (= expr (Get (Arg loop_id) i)) \
            {br} (arg-inv loop_id i)) \
            {br}((set (is-inv-Expr loop_id expr) true)){ruleset})"
        )),
        Constructor::Num | Constructor::Boolean => {
            let ctor_pattern = ctor.construct(|field| field.var());
            Some(format!(
                "(rule ((BodyContainsExpr loop_id expr) \
                {br} (Loop loop_id in out) \
                {br} (= expr {ctor_pattern})) \
                {br}((set (is-inv-Expr loop_id expr) true)){ruleset})"
            ))
        }
        _ => None,
    }
}

fn is_invariant_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let br = "\n      ";
    let ruleset = " :ruleset always-run";
    let ctor_pattern = ctor.construct(|field| field.var());

    match ctor {
        // list handled in loop_invariant.egg
        // base cases are skipped
        // print, read, write are not invariant
        // assume Arg as whole is not invariant
        Constructor::Cons
        | Constructor::Nil
        | Constructor::Num
        | Constructor::Boolean
        | Constructor::Print
        | Constructor::Read
        | Constructor::Write
        | Constructor::Arg => None,
        _ => {
            let is_inv_ctor = ctor
                .filter_map_fields(|field| match field.purpose {
                    Purpose::Static(_)
                    | Purpose::CapturingId
                    | Purpose::CapturedExpr
                    | Purpose::ReferencingId => None,
                    Purpose::SubExpr | Purpose::SubListExpr => {
                        let var = field.var();
                        let sort = field.sort().name();
                        Some(format!("(= true (is-inv-{sort} loop_id {var}))"))
                    }
                })
                .join(" ");
            let is_pure = match ctor {
                Constructor::Call | Constructor::Let | Constructor::Loop => {
                    format!("{br} (ExprIsPure expr)")
                }
                _ => String::new(),
            };
            Some(format!(
                "(rule ((BodyContainsExpr loop_id expr) \
                {br} (Loop loop_id in out) \
                {br} (= expr {ctor_pattern}) \
                {br} {is_inv_ctor} {is_pure}) \
                {br}((set (is-inv-Expr loop_id expr) true)){ruleset})"
            ))
        }
    }
}

pub(crate) fn rules() -> Vec<String> {
    iter::once(include_str!("loop_invariant.egg").to_string())
        .chain(Constructor::iter().filter_map(is_inv_base_case_for_ctor))
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
            (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 5)))
            (All id1 (Sequential) (Pair
                ; pred
                (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
                ; output
                (All id1 (Parallel) 
                    (Pair
                        (Get (Arg id1) 0)
                        (Sub (Get (Arg id1) 1) (Add (Num id1 1) (Get (Arg id1) 0))) ))))))
    ";

    let check = "
        (check (arg-inv id1 0))
        (fail (check (arg-inv id1 1)))
        (check (= true (is-inv-Expr id1 (Get (Arg id1) 0))))
        (check (= false (is-inv-Expr id1 (Get (Arg id1) 1))))
        (check (= true (is-inv-Expr id1 (Add (Num id1 1) (Get (Arg id1) 0)))))
        (check (= false (is-inv-Expr id1 (Sub (Get (Arg id1) 1) (Add (Num id1 1) (Get (Arg id1) 0))) )))
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
            (All id-outer (Parallel) (list5 (Num id-outer 0)
                                    (Num id-outer 1)
                                    (Num id-outer 2)
                                    (Num id-outer 3)
                                    (Num id-outer 4)))
            (All id1 (Sequential) (Pair
                ; pred
                (LessThan (Get (Arg id1) 0) (Get (Arg id1) 4))
                ; output
                (All id1 (Parallel) 
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
        (check (arg-inv id1 1))
        (check (arg-inv id1 2))
        (check (arg-inv id1 3))
        (check (arg-inv id1 4))
        (fail (check (arg-inv id1 0)))
        (let l4 (list4 (Num id1 1) (Num id1 2) (Num id1 3) (Num id1 4)))
        (check (is-inv-ListExpr-helper id1 l4 4))
        (check (= true (is-inv-ListExpr id1 l4)))
        (check (= true (is-inv-Expr id1 (Switch (Num id1 1) l4))))
        (check (= true (is-inv-Expr id1 inv)))
        (check (= false (is-inv-Expr id1 (Add (Get (Arg id1) 0) inv))))
        ;; a non exist expr should fail
        (fail (check (is-inv-Expr id1 (Switch (Num id1 2) l4))))
    ";

    crate::run_test(build, check)
}
