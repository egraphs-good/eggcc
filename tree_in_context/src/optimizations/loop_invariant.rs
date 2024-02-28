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
       (= expr (Get (Arg (LoopScope) ty) i)) 
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
        | Constructor::Alloc
        | Constructor::Function => None,
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

fn boundary_for_ctor(ctor: Constructor) -> Option<String> {
    let ruleset = " :ruleset boundary-analysis";

    match ctor {
        // Ops with one SubExpr should not be boundary, except effects
        // ListExpr handled separately
        Constructor::Cons
        | Constructor::Nil
        | Constructor::Arg
        | Constructor::Get
        | Constructor::Function => None,
        _ => {
            let ctor_pattern = ctor.construct(|field| field.var());
            let res = ctor
                .filter_map_fields(|field| {
                    let var = field.var();
                    match field.purpose {
                        Purpose::SubExpr => Some(format!(
                            "
(rule ((= true (is-inv-Expr loop expr1)) 
       (= false (is-inv-Expr loop expr2)) 
       (= expr2 {ctor_pattern}) 
       (= expr1 {var})) 
       ((boundary-Expr loop expr1)){ruleset})"
                        )),
                        _ => None,
                    }
                })
                .join("\n");
            Some(res)
        }
    }
}

pub(crate) fn rules() -> Vec<String> {
    iter::once(include_str!("loop_invariant.egg").to_string())
        .chain(Constructor::iter().filter_map(is_inv_base_case_for_ctor))
        .chain(Constructor::iter().filter_map(is_invariant_rule_for_ctor))
        .chain(Constructor::iter().filter_map(boundary_for_ctor))
        .collect::<Vec<_>>()
}

#[cfg(test)]
use crate::{ast::*, egglog_test, interpreter::Value};

#[test]
fn test_invariant_detect_simple() -> crate::Result {
    let output_ty = tuplet!(intt(), intt(), intt(), intt());

    // let inv = sub(get_looparg(2), get_looparg(1)).with_loop_arg_types(output_ty.clone(), intt());
    // let pred =
    //     less_than(get_looparg(0), get_looparg(3)).with_loop_arg_types(output_ty.clone(), boolt());
    // let not_inv = add(get_looparg(0), inv.clone()).with_loop_arg_types(output_ty.clone(), intt());

    let inner_inv =
        sub(get_looparg(2), get_looparg(1)).with_loop_arg_types(output_ty.clone(), intt());
    let inv = add(inner_inv.clone(), int(0)).with_loop_arg_types(output_ty.clone(), intt());
    let pred =
        less_than(get_looparg(0), get_looparg(3)).with_loop_arg_types(output_ty.clone(), boolt());
    let not_inv = add(get_looparg(0), inv.clone()).with_loop_arg_types(output_ty.clone(), intt());
    let inv_in_print = add(inv.clone(), int(4));
    let print = tprint(inv_in_print.clone()).with_loop_arg_types(output_ty.clone(), emptyt());

    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4)),
        concat_par(
            // parallel!(pred.clone(), not_inv.clone(), get_looparg(1),),
            // concat_par(
            //     tprint(inv_in_print.clone()),
            //     parallel!(get_looparg(2), get_looparg(3),),
            // ),
            parallel!(pred.clone(), not_inv.clone(), get_looparg(1)),
            concat_par(print.clone(), parallel!(get_looparg(2), get_looparg(3))),
        ),
    )
    .with_arg_types(emptyt(), output_ty.clone());

    let build = format!(
        "(let loop {})
        (let inv {})
        (let inv_in_print {})
        (let pred {})
        (let not_inv {})
        (let print {})
        (let inner_inv {})",
        my_loop, inv, inv_in_print, pred, not_inv, print, inner_inv
    );
    let check = format!(
        "(check (= true (is-inv-Expr loop inv)))
		(check (= true (is-inv-Expr loop inv_in_print)))
		(check (= false (is-inv-Expr loop pred)))
		(check (= false (is-inv-Expr loop not_inv)))
        (check (boundary-Expr loop inv))
        (check (boundary-Expr loop inv_in_print))
        (fail (check (boundary-Expr loop not_inv)))
        (fail (check (boundary-Expr loop pred)))
        (check (= true (is-inv-Expr loop inner_inv)))
        (fail (check (boundary-Expr loop inner_inv)))"
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
