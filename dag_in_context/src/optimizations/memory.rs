#![allow(unused_imports)]
#![allow(dead_code)]

use crate::schema::{BaseType, Type};
use crate::schema_helpers::{Constructor, ESort, Purpose};
use crate::{egglog_test, prologue};
use strum::IntoEnumIterator;

pub(crate) fn rules() -> Vec<String> {
    // ESort::iter()
    //     .map(|sort| "(relation BodyContains* (Expr *))".replace('*', sort.name()))
    //     .chain(Constructor::iter().filter_map(captured_expr_rule_for_ctor))
    //     .chain(Constructor::iter().filter_map(subexpr_rule_for_ctor))
    //     .collect::<Vec<_>>()
    vec![]
}

#[cfg(test)]
use crate::schema::Constant;
#[cfg(test)]
use crate::Value;

#[test]
fn load_after_write() -> crate::Result {
    use crate::ast::*;
    // ptr = alloc int 1;
    // write ptr 2;
    // res = load ptr;
    // print res
    // =>
    // <some effects, but no load>
    // print 2;
    let one = int_ty(1, Type::Base(BaseType::IntT));
    let two = int_ty(2, Type::Base(BaseType::IntT));
    let orig_state = get(arg_ty(tuplet!(statet())), 0);
    let ptr_and_state = alloc(0, one, orig_state.clone(), intt());
    let ptr = get(ptr_and_state.clone(), 0);
    let state = get(ptr_and_state, 1);
    let state = write(ptr.clone(), two.clone(), state);
    let val_and_state = load(ptr, state);
    let val = get(val_and_state.clone(), 0);
    let state = get(val_and_state, 1);
    let res = tprint(val, state);

    egglog_test(
        &format!("{res}"),
        &format!("(check (= {res} (Bop (Print) (Const (Int 2) (Base (IntT))) rest)))"),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}

#[test]
fn load_after_write_without_alias() -> crate::Result {
    use crate::ast::*;
    // ptr1 = alloc int 1;
    // ptr2 = alloc int 1;
    // write ptr1 2;
    // write ptr2 3;
    // res = load ptr1;
    // print res
    // =>
    // <some effects, but no load>
    // print 2;
    //
    // This relies on the alias analysis to work.
    let one = int_ty(1, Type::Base(BaseType::IntT));
    let two = int_ty(2, Type::Base(BaseType::IntT));
    let three = int_ty(3, Type::Base(BaseType::IntT));
    let orig_state = get(arg_ty(tuplet!(statet())), 0);
    let ptr_and_state = alloc(0, one.clone(), orig_state.clone(), intt());
    let ptr1 = get(ptr_and_state.clone(), 0);
    let state = get(ptr_and_state, 1);
    let ptr_and_state = alloc(1, one, state, intt());
    let ptr2 = get(ptr_and_state.clone(), 0);
    let state = get(ptr_and_state, 1);
    let state = write(ptr1.clone(), two.clone(), state);
    let state = write(ptr2.clone(), three, state);
    let val_and_state = load(ptr1, state);
    let val = get(val_and_state.clone(), 0);
    let state = get(val_and_state, 1);
    let res = tprint(val, state);
    egglog_test(
        &format!("{res}"),
        &format!("(check (= {res} (Bop (Print) (Const (Int 2) (Base (IntT))) rest)))"),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}
