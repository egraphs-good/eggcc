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
(rule ((BodyContainsExpr body expr) 
       (= loop (DoWhile in body))
       (= expr (Get (Arg ty ctx) i)) 
       (= expr (Get body (+ i 1))))
      ((set (is-inv-Expr body expr) true)){ruleset})"
        )),
        Constructor::Const => {
            let ctor_pattern = ctor.construct(|field| field.var());
            Some(format!(
                "
(rule ((BodyContainsExpr body expr) 
       (= loop (DoWhile in body)) 
       (= expr {ctor_pattern}))
      ((set (is-inv-Expr body expr) true)){ruleset})"
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
                        Some(format!("(is-inv-{sort} body {var})"))
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
(rule ((BodyContainsExpr body expr) 
       (= loop (DoWhile in body)) 
       (= expr {ctor_pattern})
       {op_is_pure} 
       {is_inv_ctor}
       {is_pure}) 
      ((is-inv-Expr loop expr))
      {ruleset})"
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
fn test_invariant_detect() -> crate::Result {
    use crate::add_context::ContextCache;
    use crate::ast::*;
    use crate::schema::Assumption;

    let mut cache = ContextCache::new_dummy_ctx();

    let output_ty = tuplet!(intt(), intt(), intt(), intt(), statet());
    let inner_inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), base(intt()));
    let inv = add(inner_inv.clone(), int(3)).with_arg_types(output_ty.clone(), base(intt()));
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), base(boolt()));
    let not_inv = add(getat(0), inv.clone()).with_arg_types(output_ty.clone(), base(intt()));
    let inv_in_print = add(inv.clone(), int_ty(4, output_ty.clone()));
    let print =
        tprint(inv_in_print.clone(), getat(4)).with_arg_types(output_ty.clone(), base(statet()));

    let body = concat(
        parallel!(pred.clone(), not_inv.clone(), getat(1)),
        concat(parallel!(getat(2), getat(3)), single(print.clone())),
    )
    .with_arg_types(
        output_ty.clone(),
        tuplet!(boolt(), intt(), intt(), intt(), intt(), statet()),
    );
    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4), getat(0)),
        body.clone(),
    )
    .with_arg_types(tuplet!(statet()), output_ty.clone())
    .add_ctx_with_cache(Assumption::dummy(), &mut cache);

    let my_loop_ctx = inloop(
        parallel!(int(1), int(2), int(3), int(4), getat(0))
            .with_arg_types(tuplet!(statet()), output_ty.clone())
            .add_ctx_with_cache(Assumption::dummy(), &mut cache),
        body.clone(),
    );

    let inv = inv.add_ctx_with_cache(my_loop_ctx.clone(), &mut cache);
    let inv_in_print = inv_in_print.add_ctx_with_cache(my_loop_ctx.clone(), &mut cache);
    let pred = pred.add_ctx_with_cache(my_loop_ctx.clone(), &mut cache);
    let not_inv = not_inv.add_ctx_with_cache(my_loop_ctx.clone(), &mut cache);
    let print = print.add_ctx_with_cache(my_loop_ctx.clone(), &mut cache);
    let inner_inv = inner_inv.add_ctx_with_cache(my_loop_ctx.clone(), &mut cache);
    let body = body.add_ctx_with_cache(my_loop_ctx.clone(), &mut cache);

    let build = format!(
        "(let loop {})
        (let body {})
        (let inv {})
        (let inv_in_print {})
        (let pred {})
        (let not_inv {})
        (let print {})
        (let inner_inv {})
        {}",
        my_loop,
        body,
        inv,
        inv_in_print,
        pred,
        not_inv,
        print,
        inner_inv,
        cache.get_unions()
    );
    let check = "(check (= true (is-inv-Expr body inv)))
		(check (is-inv-Expr body inv_in_print))
		(fail (check (is-inv-Expr body pred)))
		(fail (check (is-inv-Expr body not_inv)))
        (check (boundary-Expr body inv))
        (check (boundary-Expr body inv_in_print))
        (fail (check (boundary-Expr body not_inv)))
        (fail (check (boundary-Expr body pred)))
        (check (is-inv-Expr body inner_inv))
        (fail (check (boundary-Expr body inner_inv)))";

    egglog_test(
        &build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn test_invariant_hoist() -> crate::Result {
    use crate::add_context::ContextCache;
    use crate::ast::*;
    use crate::schema::Assumption;

    let mut cache = ContextCache::new_dummy_ctx();
    let output_ty = tuplet!(intt(), intt(), intt(), statet());
    let inner_inv = sub(getat(2), getat(1));
    let inv = add(inner_inv.clone(), int(1));
    let print = tprint(inv.clone(), getat(3));

    let my_loop = dowhile(
        parallel!(getat(0), getat(1), getat(2), getat(3)),
        parallel!(
            less_than(getat(0), getat(1)),
            int(3),
            getat(1),
            getat(2),
            print,
        ),
    )
    .with_arg_types(output_ty.clone(), output_ty.clone())
    .add_ctx_with_cache(Assumption::dummy(), &mut cache);

    let new_out_ty = tuplet!(intt(), intt(), intt(), statet(), intt());
    let mut cache = ContextCache::new_symbolic_ctx();

    let hoisted_loop = dowhile(
        parallel!(
            getat(0),
            getat(1),
            getat(2),
            getat(3),
            add(int(1), sub(getat(2), getat(1)))
        ),
        parallel!(
            less_than(getat(0), getat(1)),
            int(3),
            getat(1),
            getat(2),
            tprint(getat(4), getat(3)),
            getat(4)
        ),
    )
    .with_arg_types(output_ty.clone(), new_out_ty)
    .add_ctx_with_cache(Assumption::dummy(), &mut cache);

    let build = format!("(let loop {}) \n", my_loop);
    let check = format!(
        "(check {})
        (check (= loop (SubTuple {} 0 4)))",
        hoisted_loop.clone(),
        hoisted_loop
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
