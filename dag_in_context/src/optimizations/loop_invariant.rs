use crate::schema_helpers::{Constructor, Purpose};
use std::iter;
use strum::IntoEnumIterator;

#[cfg(test)]
use crate::{egglog_test, interpreter::Value};

fn is_inv_base_case_for_ctor(ctor: Constructor) -> Option<String> {
    let ruleset = " :ruleset always-run";

    match ctor {
        Constructor::Get => Some(format!(
            "
(rule ((BodyContainsExpr loop expr) 
       (= loop (DoWhile in out)) 
       (= expr (Get (Arg ty ctx) i)) 
       (= loop (DoWhile in pred_out))
       (= expr (Get pred_out (+ i 1)))) 
      ((set (is-inv-Expr loop expr) true)){ruleset})"
        )),
        Constructor::Const => {
            let ctor_pattern = ctor.construct(|field| field.var());
            Some(format!(
                "
(rule ((BodyContainsExpr loop expr) 
       (= loop (DoWhile in out)) 
       (= expr {ctor_pattern})) 
      ((set (is-inv-Expr loop expr) true)){ruleset})"
            ))
        }
        _ => None,
    }
}

fn is_invariant_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let ruleset = " :ruleset always-run";
    let ctor_pattern = ctor.construct(|field| field.var());

    match ctor {
        // list handled in loop_invariant.egg
        // base cases are skipped
        // print, load, and Write are not invariant
        Constructor::Cons
        | Constructor::Nil
        | Constructor::Const
        | Constructor::Arg
        | Constructor::Alloc => None,
        _ => {
            let is_inv_ctor = ctor
                .filter_map_fields(|field| match field.purpose {
                    Purpose::Static(_) | Purpose::CapturedExpr => None,
                    Purpose::SubExpr | Purpose::CapturedSubListExpr => {
                        let var = field.var();
                        let sort = field.sort().name();
                        Some(format!("(= true (is-inv-{sort} loop {var}))"))
                    }
                })
                .join(" ");
            let is_pure = match ctor {
                Constructor::Call | Constructor::DoWhile => "(ExprIsPure expr)",
                _ => "",
            };

            let op_is_pure = match ctor {
                Constructor::Bop => "(BinaryOpIsPure _op)",
                Constructor::Uop => "(UnaryOpIsPure _op)",
                _ => "",
            };

            Some(format!(
                "
(rule ((BodyContainsExpr loop expr) 
       (= loop (DoWhile in out)) 
       (= expr {ctor_pattern}) {op_is_pure} 
       {is_inv_ctor} 
       {is_pure}) 
      ((set (is-inv-Expr loop expr) true)){ruleset})"
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
fn test_invariant_detect_simple() -> crate::Result {
    use crate::ast::*;
    let output_ty = tuplet!(intt(), intt(), intt(), intt());
    let inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), base(intt()));
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), base(boolt()));
    let not_inv = add(getat(0), inv.clone()).with_arg_types(output_ty.clone(), base(intt()));
    let add_inv = add(inv.clone(), int(4)).with_arg_types(output_ty.clone(), base(intt()));
    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4)),
        concat(
            parallel!(pred.clone(), not_inv.clone(), getat(1),),
            parallel!(getat(2), get(parallel!(add_inv.clone(), getat(3)), 1),),
        ),
    )
    .with_arg_types(emptyt(), output_ty.clone());

    let build = format!("(let loop {})", my_loop);
    let check = format!(
        "(check (= true (is-inv-Expr loop {inv})))
         (check (= true (is-inv-Expr loop {add_inv})))
         (check (= false (is-inv-Expr loop {pred})))
         (check (= false (is-inv-Expr loop {not_inv})))",
    );

    egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
