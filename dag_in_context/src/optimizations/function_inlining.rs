use std::{collections::HashMap, vec};

use crate::schema::{Expr, RcExpr, TreeProgram};

#[derive(Clone)]
pub struct CallBody {
    pub call: RcExpr,
    pub body: RcExpr,
}

// Gets a list of all the calls in the program
// and pairs them with an inlined body
fn get_calls_and_subst(expr: &RcExpr, func_to_body: &HashMap<String, &RcExpr>) -> Vec<CallBody> {
    // Get calls from children
    let mut calls = if !expr.children_exprs().is_empty() {
        expr.children_exprs()
            .iter()
            .flat_map(|child| get_calls_and_subst(child, func_to_body))
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // Inline this call
    if let Expr::Call(func_name, args) = expr.as_ref() {
        let substituted = Expr::subst(args, func_to_body[func_name]);

        // Substitute args into the body
        calls.push(CallBody {
            call: expr.clone(),
            body: substituted,
        })
    };

    calls
}

// Generates a ruleset with pairs of (call, inlined body) to union
// Inlines
pub fn function_inlining_pairs(program: &TreeProgram, iterations: i32) -> Vec<CallBody> {
    // Find all Calls in the program
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

    let one_inlining = all_funcs
        .iter()
        // Find calls and their inlined version within each function
        .flat_map(|func| get_calls_and_subst(func, &func_name_to_body))
        .collect::<Vec<_>>();

    let mut all_inlining = one_inlining.clone();

    // Repeat! Get calls and subst for each new substituted body.
    for _ in 1..iterations {
        let one_inlining = one_inlining
            .iter()
            .flat_map(|call_body| get_calls_and_subst(&call_body.body, &func_name_to_body))
            .collect::<Vec<_>>();
        all_inlining.extend(one_inlining)
    }

    all_inlining
}
