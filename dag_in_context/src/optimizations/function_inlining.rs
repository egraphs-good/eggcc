use std::{
    collections::HashSet,
    rc::Rc,
    vec,
};

use egglog::util::IndexMap;

use crate::schema::{Expr, RcExpr, TreeProgram};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct CallBody {
    pub call: RcExpr,
    pub body: RcExpr,
}

// Gets a set of all the calls in the program
fn get_calls(expr: &RcExpr) -> Vec<RcExpr> {
    // Get calls from children
    let mut calls = if !expr.children_exprs().is_empty() {
        expr.children_exprs()
            .iter()
            .flat_map(get_calls)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // Add to set if this is a call
    if let Expr::Call(_, _) = expr.as_ref() {
        calls.push(expr.clone());
    }

    calls
}

// Pairs a call with its equivalent inlined body, using the passed-in function -> body map
// to look up the body
fn subst_call(call: &RcExpr, func_to_body: &IndexMap<String, &RcExpr>) -> CallBody {
    if let Expr::Call(func_name, args) = call.as_ref() {
        CallBody {
            call: call.clone(),
            body: Expr::subst(args, func_to_body[func_name]),
        }
    } else {
        panic!("Tried to substitute non-calls.")
    }
}

// Generates a list of (call, body) pairs (in a CallBody) that can be unioned
pub fn function_inlining_pairs(program: &TreeProgram, iterations: usize) -> Vec<CallBody> {
    let mut all_funcs = vec![program.entry.clone()];
    all_funcs.extend(program.functions.clone());

    // Make func name -> body map
    let func_name_to_body = all_funcs
        .iter()
        .map(|func| {
            (
                func.func_name().expect("Func has name"),
                func.func_body().expect("Func has body"),
            )
        })
        .collect::<IndexMap<String, &RcExpr>>();

    // Inline once
    // Keep track of all calls we've seen so far to avoid duplication
    let mut prev_calls: HashSet<*const Expr> = HashSet::new();
    let mut prev_inlining = all_funcs
        .iter()
        .flat_map(get_calls)
        // Deduplicate calls before substitution
        .filter(|call| prev_calls.insert(Rc::as_ptr(call)))
        // We cannot hash RcExprs because it is too slow
        .map(|call| subst_call(&call, &func_name_to_body))
        .collect::<Vec<_>>();

    let mut all_inlining = prev_inlining.clone();

    // Repeat! Get calls and subst for each new substituted body.
    for _ in 1..iterations {
        let next_inlining = prev_inlining
            .iter()
            .flat_map(|cb| get_calls(&cb.body))
            .filter(|call| prev_calls.insert(Rc::as_ptr(call)))
            .map(|call| subst_call(&call, &func_name_to_body))
            .collect::<Vec<_>>();
        all_inlining.extend(next_inlining.clone());
        prev_inlining = next_inlining;
    }

    all_inlining
}
