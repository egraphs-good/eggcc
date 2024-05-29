use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};

use egglog::Term;

use crate::{
    add_context::UnionsAnd,
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
fn subst_call(
    call: &RcExpr,
    func_to_body: &HashMap<String, &RcExpr>,
    unions: &mut Vec<(String, String)>,
) -> CallBody {
    if let Expr::Call(func_name, args) = call.as_ref() {
        let unions_and_value = Expr::subst(args, func_to_body[func_name]);
        unions.extend(unions_and_value.unions);
        CallBody {
            call: call.clone(),
            body: unions_and_value.value,
        }
    } else {
        panic!("Tried to substitute non-calls.")
    }
}

// Generates a list of (call, body) pairs (in a CallBody) that can be unioned
pub fn function_inlining_pairs(
    program: &TreeProgram,
    iterations: usize,
) -> UnionsAnd<Vec<CallBody>> {
    let mut unions = Vec::new();

    if iterations == 0 {
        return UnionsAnd {
            unions,
            value: Vec::new(),
        };
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
        .map(|call| subst_call(call, &func_name_to_body, &mut unions))
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
            .map(|call| subst_call(call, &func_name_to_body, &mut unions))
            .collect::<Vec<CallBody>>();
        inlined_calls.extend(new_inlines.clone());
    }

    UnionsAnd {
        unions,
        value: inlined_calls,
    }
}

// Returns a formatted string of (union call body) for each pair
pub fn print_function_inlining_pairs(
    function_inlining_pairs: UnionsAnd<Vec<CallBody>>,
    printed: &mut String,
    tree_state: &mut TreeToEgglog,
    term_cache: &mut HashMap<Term, String>,
) -> String {
    let inlined_calls = "(relation InlinedCall (String Expr))";
    // Get unions and mark each call as inlined for extraction purposes
    let printed_pairs = function_inlining_pairs
        .value
        .iter()
        .map(|cb| {
            if let Expr::Call(callee, _) = cb.call.as_ref() {
                let call_term = cb.call.to_egglog_internal(tree_state);
                let call_with_intermed = print_with_intermediate_helper(
                    &tree_state.termdag,
                    call_term.clone(),
                    term_cache,
                    printed,
                );

                let body_term = cb.body.to_egglog_internal(tree_state);
                let inlined_with_intermed = print_with_intermediate_helper(
                    &tree_state.termdag,
                    body_term,
                    term_cache,
                    printed,
                );

                let call_args = cb.call.children_exprs()[0].to_egglog_internal(tree_state);
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
            } else {
                panic!("Tried to inline non-call")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{inlined_calls} {printed_pairs} {}",
        function_inlining_pairs.get_unions()
    )
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

    assert_eq!(pairs.value.len(), 6)
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
        assert_eq!(pairs.value.len(), iterations);
    }
}
