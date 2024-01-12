use std::iter;

use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn is_pure(ctor: &Constructor) -> bool {
    use Constructor::*;
    match ctor {
        Num | Boolean | UnitExpr | Add | Sub | Mul | LessThan | And | Or | Not | Get | All
        | Switch | Loop | Let | Arg | Call | Cons | Nil => true,
        Print | Read | Write => false,
    }
}

// Builds rules like:
// (rule ((Add x y) (ExprIsPure x) (ExprIsPure y))
//       ((ExprIsPure (Add x y)))
//       :ruleset always-run)
fn purity_rule_for_ctor(ctor: Constructor) -> Option<String> {
    if !is_pure(&ctor) {
        return None;
    }

    // e.g. ["(ExprIsPure x)", "(ExprIsPure y)"]
    let children_pure_queries = ctor.filter_map_fields(|field| match field.purpose {
        Purpose::Static(_) | Purpose::CapturingId | Purpose::ReferencingId => None,
        Purpose::SubExpr | Purpose::SubListExpr | Purpose::CapturedExpr => {
            let var = field.var();
            let sort = field.sort().name();
            Some(format!("({sort}IsPure {var})"))
        }
    });

    // e.g. "(Add x y)"
    let ctor_pattern = ctor.construct(|field| field.var());

    let queries = iter::once(ctor_pattern.clone())
        .chain(children_pure_queries)
        .collect::<Vec<_>>()
        .join(" ");

    let sort = ctor.sort().name();
    let br = "\n      ";
    Some(format!(
        "(rule ({queries}){br}(({sort}IsPure {ctor_pattern})){br}:ruleset always-run)"
    ))
}

pub(crate) fn purity_analysis_rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(relation *IsPure (*))".replace('*', sort.name()))
        .chain(Constructor::iter().filter_map(purity_rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_purity_analysis() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let pure-loop
    (Loop id1
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All (Parallel) (Pair
                (Add (Get (Arg id1) 0) (Num id1 1))
                (Sub (Get (Arg id1) 1) (Num id1 1))))))))

(let id2 (Id (i64-fresh!)))
(let impure-loop
    (Loop id2
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id2) 0) (Get (Arg id2) 1))
            ; output
            (IgnoreFirst
                (Print (Num id2 1))
                (All (Parallel) (Pair
                    (Add (Get (Arg id2) 0) (Num id2 1))
                    (Sub (Get (Arg id2) 1) (Num id2 1)))))))))
    "
    );
    let check = "
(check (ExprIsPure pure-loop))
(fail (check (ExprIsPure impure-loop)))
    ";
    crate::run_test(build, check)
}
