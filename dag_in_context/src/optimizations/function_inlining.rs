use std::{rc::Rc, vec};

use egglog::Term;
use indexmap::{IndexMap, IndexSet};

use crate::{
    add_context::ContextCache,
    print_with_intermediate_helper,
    schema::{Expr, RcExpr, TreeProgram},
    to_egglog::TreeToEgglog,
};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct CallBody {
    pub call: RcExpr,
    pub body: RcExpr,
}

// Gets a set of all the calls in the program
fn get_calls_with_cache(
    expr: &RcExpr,
    calls: &mut Vec<RcExpr>,
    seen_exprs: &mut IndexSet<*const Expr>,
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
fn subst_call(
    call: &RcExpr,
    func_to_body: &IndexMap<String, &RcExpr>,
    cache: &mut ContextCache,
) -> CallBody {
    if let Expr::Call(func_name, args) = call.as_ref() {
        CallBody {
            call: call.clone(),
            body: Expr::subst(args, func_to_body[func_name], cache),
        }
    } else {
        panic!("Tried to substitute non-calls.")
    }
}

/// Generates a list of (call, body) pairs (in a CallBody) that can be unioned
/// Only generates inlining for the batch of fn names passed in `fns`
pub fn function_inlining_pairs(
    program: &TreeProgram,
    fns: Vec<String>,
    iterations: usize,
    cache: &mut ContextCache,
) -> Vec<CallBody> {
    if iterations == 0 {
        return vec![];
    }

    let mut all_funcs = vec![program.entry.clone()];
    all_funcs.extend(program.functions.clone());
    let target_funcs = all_funcs
        .iter()
        .filter(|func| fns.contains(&func.func_name().expect("Func has name")))
        .collect::<Vec<_>>();

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
    let mut seen_exprs: IndexSet<*const Expr> = IndexSet::new();
    let mut calls: Vec<RcExpr> = Vec::new();
    for target_func in target_funcs {
        get_calls_with_cache(target_func, &mut calls, &mut seen_exprs);
    }

    let mut inlined_calls = calls
        .iter()
        .map(|call| subst_call(call, &func_name_to_body, cache))
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
            .map(|call| subst_call(call, &func_name_to_body, cache))
            .collect::<Vec<CallBody>>();
        inlined_calls.extend(new_inlines.clone());
    }

    inlined_calls
}

// Returns a formatted string of (union call body) for each pair
pub fn print_function_inlining_pairs(
    function_inlining_pairs: Vec<CallBody>,
    printed: &mut String,
    tree_state: &mut TreeToEgglog,
    term_cache: &mut IndexMap<Term, String>,
) -> String {
    let inlined_calls = "";
    // Get unions and mark each call as inlined for extraction purposes
    let printed_pairs = function_inlining_pairs
        .iter()
        .map(|cb| {
            let Expr::Call(callee, _) = cb.call.as_ref() else {
                panic!("Tried to inline non-call")
            };
            let call_term = cb.call.to_egglog_with(tree_state);
            let call_with_intermed = print_with_intermediate_helper(
                &tree_state.termdag,
                call_term.clone(),
                term_cache,
                printed,
            );

            let body_term = cb.body.to_egglog_with(tree_state);
            let inlined_with_intermed =
                print_with_intermediate_helper(&tree_state.termdag, body_term, term_cache, printed);

            let call_args = cb.call.children_exprs()[0].to_egglog_with(tree_state);
            let call_args_with_intermed = print_with_intermediate_helper(
                &tree_state.termdag,
                call_args.clone(),
                term_cache,
                printed,
            );
            format!(
                // We need to subsume, otherwise the Call in the original program could get
                // substituted into another context during optimization and no longer match InlinedCall.
                "
(union {call_with_intermed} {inlined_with_intermed})
(InlinedCall \"{callee}\" {call_args_with_intermed})
(subsume (Call \"{callee}\" {call_args_with_intermed}))
",
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{inlined_calls} {printed_pairs}")
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

    let pairs_main = function_inlining_pairs(
        &program,
        vec!["main".to_string()],
        iterations,
        &mut ContextCache::new(),
    );
    let pairs_inc_twice = function_inlining_pairs(
        &program,
        vec!["inc_twice".to_string()],
        iterations,
        &mut ContextCache::new(),
    );
    let pairs_inc = function_inlining_pairs(
        &program,
        vec!["inc".to_string()],
        iterations,
        &mut ContextCache::new(),
    );

    // First iteration:
    // call inc_twice 1 --> call inc (call inc 1) ... so the new calls are call inc (call inc 1), call inc 1
    // call inc 5 --> add 1 5
    // call inc arg --> add 1 arg
    // call inc (call inc arg) --> add 1 (call inc arg)

    // Second iteration
    // call inc (call inc 1) --> add 1 (call inc 1)
    // call inc 1 --> add 1 1

    // No more iterations!

    assert_eq!(pairs_main.len(), 4);
    assert_eq!(pairs_inc_twice.len(), 2);
    assert_eq!(pairs_inc.len(), 0)
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
        let pairs = function_inlining_pairs(
            &program,
            vec!["inf_rec".to_string()],
            iterations,
            &mut ContextCache::new(),
        );
        assert_eq!(pairs.len(), iterations);
    }
}
