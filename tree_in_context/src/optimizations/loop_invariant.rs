use crate::schema_helpers::{Constructor, Purpose};
use std::iter;
use strum::IntoEnumIterator;

fn is_inv_base_case_for_ctor(ctor: Constructor) -> Option<String> {
    let ruleset = " :ruleset always-run";

    match ctor {
        // I assume input is tuple here
        // TODO InContext Node
        Constructor::Get => Some(format!(
            "
(rule ((BodyContainsExpr loop expr) 
       (= loop (DoWhile in out)) 
       (= expr (Get (Arg ty) i)) 
       (= loop (DoWhile in pred_out))
       (= expr (tuple-ith pred_out (+ i 1)))) 
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
                    Purpose::SubExpr | Purpose::SubListExpr => {
                        let var = field.var();
                        let sort = field.sort().name();
                        Some(format!("(= true (is-inv-{sort} loop {var}))"))
                    }
                })
                .join(" ");
            let is_pure = match ctor {
                Constructor::Call | Constructor::Let | Constructor::DoWhile => "(ExprIsPure expr)",
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

#[cfg(test)]
use crate::{ast::*, egglog_test, interpreter::Value};

#[test]
fn test_invariant_detect_simple() -> crate::Result {
    let output_ty = tuplet!(intt(), intt(), intt(), intt());
    let inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), intt());
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), boolt());
    let not_inv = add(getat(0), inv.clone()).with_arg_types(output_ty.clone(), intt());
    let inv_in_print = add(inv.clone(), int(4)).with_arg_types(output_ty.clone(), intt());
    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4)),
        concat_par(
            parallel!(pred.clone(), not_inv.clone(), getat(1),),
            concat_par(tprint(inv_in_print.clone()), parallel!(getat(2), getat(3),)),
        ),
    )
    .with_arg_types(emptyt(), output_ty.clone());

    let build = format!("(let loop {})", my_loop);
    let check = format!(
        "(check (= true (is-inv-Expr loop {})))
		(check (= true (is-inv-Expr loop {})))
		(check (= false (is-inv-Expr loop {})))
		(check (= false (is-inv-Expr loop {})))",
        inv, inv_in_print, pred, not_inv
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
