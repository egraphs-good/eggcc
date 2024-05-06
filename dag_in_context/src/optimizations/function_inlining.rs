use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};

use crate::schema::{Expr, RcExpr, TreeProgram};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct CallBody {
    pub call: RcExpr,
    pub body: RcExpr,
}

// Gets a set of all the calls in the program
#[allow(dead_code)]
fn get_calls_with_cache(
    expr: &RcExpr,
    calls: &mut Vec<RcExpr>,
    seen_exprs: &mut HashSet<*const Expr>,
) {
    if seen_exprs.get(&Rc::as_ptr(expr)).is_some() {
        return;
    };

    // Get calls from children
    if !expr.children_exprs().is_empty() {
        expr.children_exprs()
            .iter()
            .for_each(|child| get_calls_with_cache(child, calls, seen_exprs));
    }

    // Add to set if this is a call
    if let Expr::Call(_, _) = expr.as_ref() {
        calls.push(expr.clone());
    }

    seen_exprs.insert(Rc::as_ptr(expr));
}

// Pairs a call with its equivalent inlined body, using the passed-in function -> body map
// to look up the body
#[allow(dead_code)]
fn subst_call(call: &RcExpr, func_to_body: &HashMap<String, &RcExpr>) -> CallBody {
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
#[allow(dead_code)]
pub fn function_inlining_pairs(program: &TreeProgram, iterations: usize) -> Vec<CallBody> {
    if iterations == 0 {
        return vec![];
    }

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
        .collect::<HashMap<String, &RcExpr>>();

    // Inline once
    let mut seen_exprs: HashSet<*const Expr> = HashSet::new();
    let mut calls: Vec<RcExpr> = Vec::new();
    all_funcs
        .iter()
        .for_each(|func| get_calls_with_cache(func, &mut calls, &mut seen_exprs));

    let mut inlined_calls = calls
        .iter()
        .map(|call| subst_call(call, &func_name_to_body))
        .collect::<Vec<_>>();

    // Repeat! Get calls and subst for each new substituted body.
    let mut new_inlines = inlined_calls.clone();
    for _ in 1..iterations {
        // Only repeat on new inlines
        let mut new_calls: Vec<RcExpr> = Vec::new();
        new_inlines
            .iter()
            .for_each(|cb| get_calls_with_cache(&cb.body, &mut new_calls, &mut seen_exprs));

        // No more new calls to discover
        if new_calls.is_empty() {
            break;
        }

        // Only work on new calls, added from the new inlines
        new_inlines = new_calls
            .iter()
            .map(|call| subst_call(call, &func_name_to_body))
            .collect::<Vec<CallBody>>();
        inlined_calls.extend(new_inlines.clone());
    }

    inlined_calls
}

// Check that function inling pairs produces the right number of pairs for
// a simple, non-cyclic call graph
#[test]
fn test_function_inlining_pairs() {
    use crate::ast::*;

    let iterations = 10;

    let main = function(
        "main",
        emptyt(),
        base(intt()),
        add(call("inc_twice", int(1)), call("inc", int(5))),
    );

    let inc_twice = function(
        "inc_twice",
        base(intt()),
        base(intt()),
        call("inc", call("inc", arg())),
    );

    let inc = function("inc", base(intt()), base(intt()), add(int(1), arg()));

    let program = program!(main, inc_twice, inc);

    let pairs = function_inlining_pairs(&program, iterations);

    // First iteration:
    // call inc_twice 1 --> call inc (call inc 1) ... so the new calls are call inc (call inc 1), call inc 1
    // call inc 5 --> add 1 5
    // call inc arg --> add 1 arg
    // call inc (call inc arg) --> add 1 (call inc arg)

    // Second iteration
    // call inc (call inc 1) --> add 1 (call inc 1)
    // call inc 1 --> add 1 1

    // No more iterations!

    assert_eq!(pairs.len(), 6)
}

// Infinite recursion should produce as many pairs as iterations
#[test]
fn test_inf_recursion_function_inlining_pairs() {
    use crate::ast::*;

    let program = function(
        "inf_rec",
        base(intt()),
        base(intt()),
        call("inf_rec", add(int(1), arg())),
    )
    .to_program(base(intt()), base(intt()));

    for iterations in 0..10 {
        let pairs = function_inlining_pairs(&program, iterations);
        assert_eq!(pairs.len(), iterations);
    }
}
