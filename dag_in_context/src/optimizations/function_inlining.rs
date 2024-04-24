use std::{collections::HashMap, vec};

use crate::schema::{Expr, RcExpr, TreeProgram};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct CallBody {
    pub call: RcExpr,
    pub body: RcExpr,
}

// Gets a list of all the calls in the program
// and pairs them with an inlined body
fn get_calls_and_subst(
    expr: &RcExpr,
    func_to_body: &HashMap<String, &RcExpr>,
) -> HashMap<RcExpr, RcExpr> {
    // Get calls from children
    let mut calls = if !expr.children_exprs().is_empty() {
        expr.children_exprs()
            .iter()
            .flat_map(|child| get_calls_and_subst(child, func_to_body))
            .collect::<HashMap<RcExpr, RcExpr>>()
    } else {
        HashMap::new()
    };

    // Inline this call
    if let Expr::Call(func_name, args) = expr.as_ref() {
        if !calls.contains_key(expr) {
            let substituted = Expr::subst(args, func_to_body[func_name]);

            // Substitute args into the body
            calls.insert(expr.clone(), substituted);
        }
    };

    calls
}

// Generates a ruleset with pairs of (call, inlined body) to union
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
        .collect::<HashMap<RcExpr, RcExpr>>();

    let mut all_inlining = one_inlining.clone();

    // Repeat! Get calls and subst for each new substituted body.
    for _ in 1..iterations {
        let one_inlining = one_inlining
            .iter()
            .flat_map(|(_, body)| get_calls_and_subst(body, &func_name_to_body))
            .collect::<HashMap<RcExpr, RcExpr>>();
        all_inlining.extend(one_inlining)
    }

    let mut all_inlining = all_inlining
        .drain()
        .map(|(call, body)| CallBody { call, body })
        .collect::<Vec<_>>();

    // Sort to not rely on hash ordering
    all_inlining.sort();

    all_inlining
}
