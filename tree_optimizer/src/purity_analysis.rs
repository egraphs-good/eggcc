use std::iter;

use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn is_pure(ctor: &Constructor) -> bool {
    use Constructor::*;
    match ctor {
        Num | Boolean | Add | Sub | Mul | LessThan | And | Or | Not | Get | All | Switch | Loop
        | Branch | Let | Arg | Call | Cons | Nil => true,
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

    let br = "\n      ";
    if ctor == Constructor::Call {
        return Some(format!(
            "(rule ((Call _f _arg) (ExprIsPure _arg) (FunctionIsPure _f)){br}((ExprIsPure (Call _f _arg))):ruleset always-run)"
        ));
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
    Some(format!(
        "(rule ({queries}){br}(({sort}IsPure {ctor_pattern})){br}:ruleset always-run)"
    ))
}

pub(crate) fn purity_analysis_rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(relation *IsPure (*))".replace('*', sort.name()))
        .chain(iter::once( "(relation FunctionIsPure (IdSort))\n(rule ((Function id out) (ExprIsPure out)) ((FunctionIsPure id)):ruleset always-run)".to_string()))
        .chain(Constructor::iter().filter_map(purity_rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_purity_analysis() -> Result<(), egglog::Error> {
    let build = &*"
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let pure-loop
    (Loop id1
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id1 (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All id1 (Parallel) (Pair
                (Add (Get (Arg id1) 0) (Num id1 1))
                (Sub (Get (Arg id1) 1) (Num id1 1))))))))

(let id2 (Id (i64-fresh!)))
(let impure-loop
    (Loop id2
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id2 (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id2) 0) (Get (Arg id2) 1))
            ; output
            (IgnoreFirst id2
                (Print (Num id2 1))
                (All id2 (Parallel) (Pair
                    (Add (Get (Arg id2) 0) (Num id2 1))
                    (Sub (Get (Arg id2) 1) (Num id2 1)))))))))
    "
    .to_string();
    let check = "
(check (ExprIsPure pure-loop))
(fail (check (ExprIsPure impure-loop)))
    ";
    crate::run_test(build, check)
}

#[test]
fn test_purity_function() -> Result<(), egglog::Error> {
    let build = &*"
(let id1 (Id (i64-fresh!)))
(let id2 (Id (i64-fresh!)))
(let id_fun1 (Id (i64-fresh!)))
(let id_fun2 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))

;; f1 is pure
(let f1
    (Function id_fun1
            (Add 
                (Get (Arg id_fun1) 0) 
                (Get (Arg id_fun1) 0))))
;; f2 is impure
(let f2
    (Function id_fun2
        (Get 
            (All id_fun2 (Sequential)
                    (Pair 
                    (Print (Get (Arg id_fun2) 0)) 
                    (Add 
                        (Get (Arg id_fun2) 0) 
                        (Get (Arg id_fun2) 0))))
            1)))
(let pure-loop
    (Loop id1
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id1 (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All id1 (Parallel) 
                    (Pair
                    (Add (Call id_fun1 (All id1 (Sequential) (Cons (Get (Arg id1) 0) (Nil)))) (Num id1 1))
                    (Sub (Get (Arg id1) 1) (Num id1 1))))))))
(let impure-loop
    (Loop id2
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id2 (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id2) 0) (Get (Arg id2) 1))
            ; output
            (All id2 (Parallel) 
                    (Pair
                    (Add (Call id_fun2 (All id2 (Sequential) (Cons (Get (Arg id2) 0) (Nil)))) (Num id2 1))
                    (Sub (Get (Arg id2) 1) (Num id2 1))))))))
    "
    .to_string();
    let check = "
(check (FunctionIsPure id_fun1))
(fail (check (FunctionIsPure id_fun2)))
(check (ExprIsPure pure-loop))
(fail (check (ExprIsPure impure-loop)))
    ";
    crate::run_test(build, check)
}
