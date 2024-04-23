use crate::{optimize, schema_helpers::{Constructor, Purpose}};
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

#[test]
fn test_invariant_detect() -> crate::Result {
    use crate::ast::*;
    let output_ty = tuplet!(intt(), intt(), intt(), intt(), statet());
    let inner_inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), base(intt()));
    let inv = add(inner_inv.clone(), int(0)).with_arg_types(output_ty.clone(), base(intt()));
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), base(boolt()));
    let not_inv = add(getat(0), inv.clone()).with_arg_types(output_ty.clone(), base(intt()));
    let inv_in_print = add(inv.clone(), int_ty(4, output_ty.clone()));
    let print =
        tprint(inv_in_print.clone(), getat(4)).with_arg_types(output_ty.clone(), base(statet()));

    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4), getat(0)),
        concat(
            parallel!(pred.clone(), not_inv.clone(), getat(1)),
            concat(parallel!(getat(2), getat(3)), single(print.clone())),
        ),
    )
    .with_arg_types(tuplet!(statet()), output_ty.clone());

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


#[test]
fn test_invariant_hoist() -> crate::Result {
    use crate::ast::*;
    let output_ty = tuplet!(intt(), intt(), intt(), intt(), statet());
    let inner_inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), base(intt()));
    let inv = add(inner_inv.clone(), int(0)).with_arg_types(output_ty.clone(), base(intt()));
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), base(boolt()));
    let not_inv = add(getat(0), inv.clone()).with_arg_types(output_ty.clone(), base(intt()));
    let print =
        tprint(inv.clone(), getat(4)).with_arg_types(output_ty.clone(), base(statet()));

    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4), getat(0)),
        concat(
            parallel!(pred.clone(), not_inv.clone(), getat(1)),
            concat(parallel!(getat(2), getat(3)), single(print.clone())),
        ),
    )
    .with_arg_types(tuplet!(statet()), output_ty.clone());

    let main_fun = function("main", tuplet!(statet()), output_ty.clone(), my_loop.clone()).func_add_ctx();

    //let prog = program!(main_fun.clone(),);
    //let res = optimize(&prog).unwrap();
    //dbg!(res);
     print!("\n\n{}\n\n", main_fun.clone());
    let build = format!(
        "(let loop {})
        (let inv {})
        (let pred {})
        (let not_inv {})
        (let print {})
        (let inner_inv {})
        (let main {})
        (let new_input)",
        my_loop, inv, pred, not_inv, print, inner_inv, main_fun
    );

    print!("{}\n\n", build.clone());


    let check = format!(
        ""
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