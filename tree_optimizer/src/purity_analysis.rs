use std::iter;

use crate::{
    expr::{ESort, Expr},
    ir::{Constructor, Purpose},
};
use strum::IntoEnumIterator;

// Builds rules like:
// (rule ((BOp op x y) (ExprIsPure x) (ExprIsPure y))
//       ((ExprIsPure (BOp op x y)))
//       :ruleset always-run)
fn purity_rule_for_ctor(ctor: Constructor) -> Option<String> {
    if !ctor.is_pure() {
        return None;
    }

    let br = "\n      ";
    if let Constructor::Expr(Expr::Call(..)) = ctor {
        return Some(format!(
            "(rule ((Call f name arg) (ExprIsPure arg) (FunctionIsPure name)){br}((ExprIsPure (Call f name arg))):ruleset always-run)"
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
        .chain(iter::once( "(relation FunctionIsPure (String))\n(rule ((Function id name tyin tyout out) (ExprIsPure out)) ((FunctionIsPure name)):ruleset always-run)".to_string()))
        .chain(Constructor::iter().filter_map(purity_rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_purity_analysis() -> crate::Result {
    let build = &*"
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let pure-loop
    (Loop id1
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id1 (Sequential) (Pair
            ; pred
            (BOp (LessThan) (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All id1 (Parallel) (Pair
                (BOp (Add) (Get (Arg id1) 0) (Num id1 1))
                (BOp (Sub) (Get (Arg id1) 1) (Num id1 1))))))))

(let id2 (Id (i64-fresh!)))
(let impure-loop
    (Loop id2
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id2 (Sequential) (Pair
            ; pred
            (BOp (LessThan) (Get (Arg id2) 0) (Get (Arg id2) 1))
            ; output
            (IgnoreFirst id2
                (Print (Num id2 1))
                (All id2 (Parallel) (Pair
                    (BOp (Add) (Get (Arg id2) 0) (Num id2 1))
                    (BOp (Sub) (Get (Arg id2) 1) (Num id2 1)))))))))
    "
    .to_string();
    let check = "
(check (ExprIsPure pure-loop))
(fail (check (ExprIsPure impure-loop)))
    ";
    crate::run_test(build, check)
}

#[test]
fn test_purity_function() -> crate::Result {
    let build = &*"
(let id1 (Id (i64-fresh!)))
(let id2 (Id (i64-fresh!)))
(let id_fun1 (Id (i64-fresh!)))
(let id_fun2 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))

;; f1 is pure
(let f1
    (Function id_fun1
            \"fun1\"
            (IntT)
            (IntT)
            (BOp (Add) 
                (Get (Arg id_fun1) 0) 
                (Get (Arg id_fun1) 0))))
;; f2 is impure
(let f2
    (Function id_fun2
        \"fun2\"
        (IntT)
        (IntT)
        (Get 
            (All id_fun2 (Sequential)
                    (Pair 
                    (Print (Get (Arg id_fun2) 0)) 
                    (BOp (Add) 
                        (Get (Arg id_fun2) 0) 
                        (Get (Arg id_fun2) 0))))
            1)))
(let pure-loop
    (Loop id1
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id1 (Sequential) (Pair
            ; pred
            (BOp (LessThan) (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All id1 (Parallel) 
                    (Pair
                    (BOp (Add) (Call id1 \"fun1\" (All id1 (Sequential) (Cons (Get (Arg id1) 0) (Nil)))) (Num id1 1))
                    (BOp (Sub) (Get (Arg id1) 1) (Num id1 1))))))))
(let impure-loop
    (Loop id2
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id2 (Sequential) (Pair
            ; pred
            (BOp (LessThan) (Get (Arg id2) 0) (Get (Arg id2) 1))
            ; output
            (All id2 (Parallel) 
                    (Pair
                    (BOp (Add) (Call id2 \"fun2\" (All id2 (Sequential) (Cons (Get (Arg id2) 0) (Nil)))) (Num id2 1))
                    (BOp (Sub) (Get (Arg id2) 1) (Num id2 1))))))))
    "
    .to_string();
    let check = "
(check (FunctionIsPure \"fun1\"))
(fail (check (FunctionIsPure \"fun2\")))
(check (ExprIsPure pure-loop))
(fail (check (ExprIsPure impure-loop)))
    ";
    crate::run_test(build, check)
}
