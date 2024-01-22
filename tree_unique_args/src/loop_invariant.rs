use crate::ir::{Constructor, Purpose, Sort};
use strum::IntoEnumIterator;

fn find_invariant_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let br = "\n      ";
    let ruleset = " :ruleset always-run";

    match ctor {
        Constructor::Cons 
        | Constructor::Nil 
        | Constructor::Arg 
        | Constructor::UnitExpr => None,
        Constructor::Call => Some(format!(
            "(rule ((find-inv-Expr loop expr){br}(= expr (Call f arg))){br}((find-inv-Expr loop arg)){ruleset})"
        )),
        Constructor::Get => Some(format!(
        "{br}(rule ((find-inv-Expr loop expr)
                (= expr (Get tup i)))
            ((find-inv-Expr loop tup)){ruleset})

        (rule ((find-inv-Expr loop expr)
               (= expr (Get (Arg id) i))
               (arg-inv loop i))
            ((set (is-inv-Expr loop expr) true)){ruleset})"
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
            Some(format!("\n(rule ((find-inv-Expr loop expr){br} (= expr {ctor_pattern})){br}({find_inv_ctor}){ruleset})"))
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
        //
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
            "{br}(rule ((find-inv-Expr loop expr)
                (= expr (Get tup i))
                (= true (is-inv-Expr loop tup)))
            ((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        Constructor::Loop => Some(format!(
            "{br}(rule ((find-inv-Expr loop expr)
                (= expr (Loop id inputs pred-out))
                (= true (is-inv-Expr loop inputs))
                (ExprIsPure expr))
            ((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        Constructor::Let => Some(format!(
            "{br}(rule ((find-inv-Expr loop expr)
            (= expr (Let id inputs outputs))
            (= true (is-inv-Expr loop inputs))
            (ExprIsPure expr))
        ((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        Constructor::Switch => Some(format!(
            "{br}(rule ((find-inv-Expr loop expr)
            (= expr (Switch pred branch))
            (= true (is-inv-ListExpr loop branch)))
        ((set (is-inv-Expr loop expr) true)){ruleset})"
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
            "{br}(rule ((find-inv-Expr loop expr)
                (= expr {ctor_pattern})
                {is_inv_ctor})
            ((set (is-inv-Expr loop expr) true)){ruleset})"
            ))
        }
    }
}

pub(crate) fn is_inv_expr_rules() -> Vec<String> {
    Constructor::iter()
        .filter_map(is_invariant_rule_for_ctor)
        .collect::<Vec<_>>()
}

pub(crate) fn rules() -> String {
    [
        include_str!("loop_invariant.egg"),
        &find_inv_expr_rules().join("\n"),
        &is_inv_expr_rules().join("\n"),
    ]
    .join("\n")
}
