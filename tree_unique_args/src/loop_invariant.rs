
use crate::ir::{Constructor, Purpose, Sort};
use strum::IntoEnumIterator;

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

            let find_inv_ctor = ctor.construct_only_fields(|field| match field.purpose {
                Purpose::Static(Sort::I64) | Purpose::Static(Sort::Bool) => {
                    format!("(set (is-inv-Expr loop expr) true)")
                }
                Purpose::Static(_)
                | Purpose::CapturingId
                | Purpose::CapturedExpr
                | Purpose::ReferencingId => format!(""),
                Purpose::SubExpr | Purpose::SubListExpr => {
                    let var = field.var();
                    let sort = field.sort().name();
                    format!("(find-inv-{sort} loop {var})")
                }
            });
            Some(format!(
                "(rule ((find-inv-Expr loop expr) \
                {br} (= expr {ctor_pattern})) \
                {br}({find_inv_ctor}){ruleset})"
            ))
        }
    }
}

pub(crate) fn find_inv_expr_rules() -> Vec<String> {
    Constructor::iter()
        .filter_map(find_invariant_rule_for_ctor)
        .collect::<Vec<_>>()
}

fn is_invariant_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let br = "\n      ";
    let ruleset = " :ruleset always-run";

    match ctor {
        // list are handled in loop_invariant.egg
        // print, read, write are not invariant
        // assume Arg as whole is not invariant
        // Unit?
        Constructor::Cons
        | Constructor::Nil
        | Constructor::UnitExpr
        | Constructor::Print
        | Constructor::Read
        | Constructor::Write
        | Constructor::Arg => None,
        Constructor::Call => None,
        // TODO fix expr is pure?
        // Some(format!(
        // "{br}(rule ((find-inv-Expr loop expr)
        //         (= expr (Call f arg))
        //         (= true (is-inv-Expr loop arg))
        //         (ExprIsPure expr))
        //     ((set (is-inv-Expr loop expr) true)){ruleset})")),
        Constructor::Get => Some(format!(
            "(rule ((find-inv-Expr loop expr) \
            {br} (= expr (Get tup i)) \
            {br} (= true (is-inv-Expr loop tup))) \
            {br}((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        Constructor::Loop => Some(format!(
            "(rule ((find-inv-Expr loop expr) \
            {br} (= expr (Loop id inputs pred-out)) \
            {br} (= true (is-inv-Expr loop inputs)) \
            {br} (ExprIsPure expr)) \
            {br}((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        Constructor::Let => Some(format!(
            "(rule ((find-inv-Expr loop expr) \
            {br} (= expr (Let id inputs outputs)) \
            {br} (= true (is-inv-Expr loop inputs)) \
            {br} (ExprIsPure expr)) \
            {br}((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        Constructor::Switch => Some(format!(
            "(rule ((find-inv-Expr loop expr) \
            {br} (= expr (Switch pred branch)) \
            {br} (= true (is-inv-ListExpr loop branch)) \
            {br} (= true (is-inv-Expr loop pred))) \
            {br}((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        _ => {
            let ctor_pattern = ctor.construct(|field| field.var());

            let is_inv_ctor = ctor.construct_only_fields(|field| match field.purpose {
                Purpose::Static(_)
                | Purpose::CapturingId
                | Purpose::CapturedExpr
                | Purpose::ReferencingId => format!(""),
                Purpose::SubExpr | Purpose::SubListExpr => {
                    let var = field.var();
                    let sort = field.sort().name();
                    format!("(= true (is-inv-{sort} loop {var}))")
                }
            });
            Some(format!(
                "(rule ((find-inv-Expr loop expr) \
                {br} (= expr {ctor_pattern}) \
                {br} {is_inv_ctor}) \
                {br}((set (is-inv-Expr loop expr) true)){ruleset})"
            ))
        }
    }
}

pub(crate) fn is_inv_expr_rules() -> Vec<String> {
    Constructor::iter()
        .filter_map(is_invariant_rule_for_ctor)
        .collect::<Vec<_>>()
}

fn boundary_for_ctor(ctor: Constructor) -> Option<String> {
    let br = "\n      ";
    let ruleset = " :ruleset boundary-analysis";

    match ctor {
        // Ops with one SubExpr should not be boundary, except effects
        // ListExpr handled separately
        // We 
        // Unit?
        Constructor::Cons
        | Constructor::Nil
        | Constructor::UnitExpr
        | Constructor::Arg
        | Constructor::Not 
        | Constructor::Get 
        | Constructor::All => None,
        _ => {
            let ctor_pattern = ctor.construct(|field| field.var());
            let res = ctor
                .fields()
                .iter()
                .filter_map(|field| {
                    let var = field.var();
                    match field.purpose {
                        Purpose::SubExpr => Some(format!(
                            "(rule ((= true (is-inv-Expr loop expr1)) \
                            {br} (= false (is-inv-Expr loop expr2)) \
                            {br} (= expr2 {ctor_pattern}) \
                            {br} (= expr1 {var})) \
                            {br}((boundary-Expr loop expr1)){ruleset})\n"
                        )),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            Some(res)
        }
    }
}

pub(crate) fn boundary_rules() -> Vec<String> {
    Constructor::iter()
        .filter_map(boundary_for_ctor)
        .collect::<Vec<_>>()
}


// fn complexity_analysis_for_ctor(ctor: Constructor) -> Option<String> {
//     let br = "\n      ";
//     let ruleset = " :ruleset always-run";

//     match ctor {

//     }

// }

// pub(crate) fn complexity_analysis_rules() -> Vec<String> {
//     Constructor::iter()
//         .filter_map(complexity_analysis_for_ctor)
//         .collect::<Vec<_>>()
// }

pub(crate) fn rules() -> String {
    [
        include_str!("loop_invariant.egg"),
        &find_inv_expr_rules().join("\n\n"),
        &is_inv_expr_rules().join("\n\n"),
        &boundary_rules().join("\n\n"),
    ]
    .join("\n\n")
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

        (run-schedule (saturate boundary-analysis))
        (check (boundary-Expr loop (Get (Arg id1) 0)))
        (check (boundary-Expr loop (Add (Num id1 1) (Get (Arg id1) 0)) ))
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

        (run-schedule (saturate boundary-analysis))
        ;; inv is boundary
        (check (boundary-Expr loop inv))
        (fail (check (boundary-Expr loop (Add (Get (Arg id1) 0) inv))))
        (fail (check (boundary-Expr loop (Switch (Num id1 1) l4)))
    ";

    crate::run_test(build, check)
}
