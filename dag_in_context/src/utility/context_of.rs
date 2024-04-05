use crate::schema_helpers::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

// Propagates context knowledge up
fn context_of_propagation_for_ctor(ctor: Constructor) -> Option<String> {
    let pat = ctor.construct(|field| field.var());

    // Create rule to get context from each field
    match ctor.sort() {
        ESort::ListExpr => None,
        ESort::Expr => match ctor {
            // If is not a region, but has context, so its then/else branches are not
            // "captured" despite having different contexts than the If.
            Constructor::Function => None,
            Constructor::If => Some(
                "
(rule ((If _pred _then _else) (ContextOf _pred ctx))
    ((ContextOf (If _pred _then _else) ctx)) :ruleset always-run) 
            "
                .to_string(),
            ),

            // All other scopes are also regions, so the CapturedExpr/SubExpr field distinction
            // corresponds to context propagation
            _ => Some(
                ctor.filter_map_fields(|field| {
                    let field_var = field.var();

                    match field.purpose {
                        Purpose::Static(_) | Purpose::CapturedExpr | Purpose::SubListExpr => None,
                        Purpose::SubExpr => Some(format!(
                            "
(rule ({pat} (ContextOf {field_var} ctx))
    ((ContextOf {pat} ctx)) :ruleset always-run)"
                        )),
                    }
                })
                .join("\n"),
            ),
        },
    }
}

pub(crate) fn rules() -> Vec<String> {
    // We only have context for Exprs, not ListExprs.
    let mut rules = vec![
        "(relation ContextOf (Expr Assumption))".to_string(),
        // Base case: InContext means ContextOf
        "
(rule ((InContext ctx e))
    ((ContextOf e ctx)) :ruleset always-run)"
            .to_string(),
        // Error checking - each expr should only have a single context
        "
        (relation TwoContexts (Expr Assumption Assumption))
(rule ((ContextOf x ctx1)
        (ContextOf x ctx2)
        (!= ctx1 ctx2))
    (
        (TwoContexts x ctx1 ctx2)
        ;panic \"Equivalent expressions have nonequivalent context\"
    )
    :ruleset error-checking)"
            .to_string(),
    ];
    rules.append(
        &mut Constructor::iter()
            .filter_map(context_of_propagation_for_ctor)
            .collect::<Vec<_>>(),
    );
    rules
}

#[test]
fn test_context_of() -> crate::Result {
    use crate::ast::*;

    // fn main(x): if x > 3 then x else 4
    let pred = eq(arg(), int(5)).with_arg_types(base(intt()), base(boolt()));
    let body = tif(pred.clone(), arg(), int(4)).with_arg_types(base(intt()), base(intt()));
    let build = function("main", base(intt()), base(intt()), body.clone()).func_add_context();

    // If statement should have the context of its predicate
    let check = "
        (let pred-ctx (InFunc \"main\"))
        (let pred (Bop (Eq) (InContext (InFunc \"main\") (Arg (Base (IntT)))) (InContext (InFunc \"main\") (Const (Int 5) (Base (IntT))))))
        (check (ContextOf pred pred-ctx))
        (let if (If (Bop (Eq) (InContext (InFunc \"main\") (Arg (Base (IntT)))) (InContext (InFunc \"main\") (Const (Int 5) (Base (IntT))))) (InContext (InIf true (Bop (Eq) (InContext (InFunc \"main\") (Arg (Base (IntT)))) (InContext (InFunc \"main\") (Const (Int 5) (Base (IntT)))))) (Arg (Base (IntT)))) (InContext (InIf false (Bop (Eq) (InContext (InFunc \"main\") (Arg (Base (IntT)))) (InContext (InFunc \"main\") (Const (Int 5) (Base (IntT)))))) (Const (Int 4) (Base (IntT))))))
        (check (ContextOf if pred-ctx))
        ".to_string();

    // Don't pass in any programs, just want build/check to be checked
    crate::egglog_test(
        &format!("(let build {build})"),
        &check,
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}
