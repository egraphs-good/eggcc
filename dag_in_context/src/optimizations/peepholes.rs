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
        val_empty(),
        intv(1),
        vec![],
    )
}
