use std::iter;

use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn is_pure(ctor: &Constructor) -> bool {
    use Constructor::*;
    match ctor {
        Num | Boolean | UnitExpr | Add | Sub | Mul | LessThan | And | Or | Not | Get | All
        | Switch | Loop | Body | Arg | Call | Cons | Nil => true,
        Print | Read | Write => false,
    }
}

// Builds rules like:
// (rule ((Add x y) (ExprIsPure x) (ExprIsPure y))
//       ((ExprIsPure (Add x y)))
//       :ruleset fast-analyses)
fn purity_rule_for_ctor(ctor: Constructor) -> Option<String> {
    if !is_pure(&ctor) {
        return None;
    }

    // e.g. ["(ExprIsPure x)", "(ExprIsPure y)"]
    let children_pure_queries = ctor
        .fields()
        .iter()
        .filter_map(|field| match field.purpose {
            Purpose::Static(_) | Purpose::CapturingId | Purpose::ReferencingId => None,
            Purpose::SubExpr | Purpose::SubListExpr | Purpose::CapturedExpr => {
                let var = field.name;
                let sort = field.sort().name();
                Some(format!("({sort}IsPure {var})"))
            }
        })
        .collect::<Vec<_>>();

    let ctor_pattern_without_parens = iter::once(ctor.name())
        .chain(ctor.fields().iter().map(|field| field.name))
        .collect::<Vec<_>>()
        .join(" ");

    // e.g. "(Add x y)"
    let ctor_pattern = format!("({ctor_pattern_without_parens})");

    let queries = iter::once(ctor_pattern.clone())
        .chain(children_pure_queries)
        .collect::<Vec<_>>()
        .join(" ");

    let sort = ctor.sort().name();
    let br = "\n      ";
    Some(format!(
        "(rule ({queries}){br}(({sort}IsPure {ctor_pattern})){br}:ruleset fast-analyses)"
    ))
}

pub(crate) fn purity_analysis_rules() -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for sort in ESort::iter() {
        let sort_name = sort.name();
        res.push(format!("(relation {sort_name}IsPure ({sort_name}))"));
    }
    res.extend(Constructor::iter().filter_map(purity_rule_for_ctor));
    res
}

#[test]
fn test_purity_analysis() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
(let id1 (i64-fresh!))
(let pure-loop
    (Loop id1
        (All (Parallel) (Pair (Num 0) (Num 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All (Parallel) (Pair
                (Add (Get (Arg id1) 0) (Num 1))
                (Sub (Get (Arg id1) 1) (Num 1))))))))

(let id2 (i64-fresh!))
(let impure-loop
    (Loop id2
        (All (Parallel) (Pair (Num 0) (Num 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id2) 0) (Get (Arg id2) 1))
            ; output
            (IgnoreFirst
                (Print (Num 1))
                (All (Parallel) (Pair
                    (Add (Get (Arg id2) 0) (Num 1))
                    (Sub (Get (Arg id2) 1) (Num 1)))))))))
    "
    );
    let check = "
(check (ExprIsPure pure-loop))
(fail (check (ExprIsPure impure-loop)))
    ";
    crate::run_test(build, check)
}
