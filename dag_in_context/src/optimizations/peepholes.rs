//! Tests for the peepholes ruleset
#![cfg(test)]

use crate::{egglog_test, Result};

#[test]
fn arith_rewrites() -> Result {
    use crate::ast::*;
    // (0 + x + 0 + 1 + 2 + y * 1) -> (x + 3 + y)
    let ctx_ty = tuplet_vec(vec![intt(), intt(), statet()]);
    let zero = int_ty(0, ctx_ty.clone());
    let one = int_ty(1, ctx_ty.clone());
    let two = int_ty(2, ctx_ty.clone());
    let three = int_ty(3, ctx_ty.clone());
    let x = get(arg_ty(ctx_ty.clone()), 0);
    let y = get(arg_ty(ctx_ty.clone()), 1);
    let expr = add(
        add(add(zero.clone(), x.clone()), zero.clone()),
        add(add(one.clone(), two.clone()), mul(y.clone(), one.clone())),
    );

    let expected = add(x.clone(), add(three.clone(), y.clone()));
    egglog_test(
        &format!("(let expr_ {expr})"),
        &format!("(check (= expr_ {expected}))"),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}

#[test]
fn algebraic_identities() -> Result {
    use crate::ast::*;
    // (x - 0) / 1  ->  x , and min(x,x) -> x , so the whole thing collapses to x.
    let ctx_ty = tuplet_vec(vec![intt(), statet()]);
    let x = get(arg_ty(ctx_ty.clone()), 0);
    let zero = int_ty(0, ctx_ty.clone());
    let one = int_ty(1, ctx_ty.clone());
    let expr = smin(div(sub(x.clone(), zero), one), x.clone());

    egglog_test(
        &format!("(let expr_ {expr})"),
        &format!("(check (= expr_ {x}))"),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}

#[test]
fn interval_div_fold() -> Result {
    use crate::ast::*;
    // x is known in [0,0] after (x - x); (x - x + 20) / 4 -> 5 via interval analysis.
    let ctx_ty = tuplet_vec(vec![intt(), statet()]);
    let x = get(arg_ty(ctx_ty.clone()), 0);
    let twenty = int_ty(20, ctx_ty.clone());
    let four = int_ty(4, ctx_ty.clone());
    let expr = div(add(sub(x.clone(), x.clone()), twenty), four);
    let five = int_ty(5, ctx_ty.clone());

    egglog_test(
        &format!("(let expr_ {expr})"),
        &format!("(check (= expr_ {five}))"),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}

#[test]
fn self_comparison_and_double_neg() -> Result {
    use crate::ast::*;
    // (x == x)  ->  true ; --x -> x
    let ctx_ty = tuplet_vec(vec![intt(), statet()]);
    let x = get(arg_ty(ctx_ty.clone()), 0);
    let expr = eq(neg(neg(x.clone())), x.clone());

    let t = ttrue_ty(ctx_ty.clone());
    egglog_test(
        &format!("(let expr_ {expr})"),
        &format!("(check (= expr_ {t}))"),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}
