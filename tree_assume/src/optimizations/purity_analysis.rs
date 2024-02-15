#![allow(dead_code)]
#![allow(unused_imports)]
use crate::schema::{BinaryOp, UnaryOp};
use crate::schema_helpers::{Constructor, ESort, Purpose, Sort};
use std::iter;
use strum::IntoEnumIterator;

fn bop_is_pure(bop: &BinaryOp) -> bool {
    use BinaryOp::*;
    match bop {
        Add | Sub | Mul | LessThan | And | Or | PtrAdd => true,
        Write => false,
    }
}

fn uop_is_pure(uop: &UnaryOp) -> bool {
    use UnaryOp::*;
    match uop {
        Not => true,
        Print | Load => false,
    }
}

// fn is_pure(ctor: &Constructor) -> bool {
//     use Constructor::*;
//     match ctor {
//         Num | Boolean | Add | Sub | Mul | LessThan | And | Or | Not | Get | All | Switch | Loop
//         | Let | Arg | Call | Cons | Nil => true,
//         Print | Read | Write => false,
//     }
// }

// Builds rules like:
// (rule ((Bop op x y) (BinaryOpIsPure op) (ExprIsPure x) (ExprIsPure y))
//       ((ExprIsPure (Bop op x y)))
//       :ruleset always-run)
fn purity_rules_for_ctor(ctor: Constructor) -> String {
    use Constructor::*;
    match ctor {
        Call => "
            (rule ((Call _f _arg) (ExprIsPure _arg) (ExprIsPure (Function _f inty outty out)))
                  ((ExprIsPure (Call _f _arg)))
                  :ruleset always-run)"
            .to_string(),
        Function | Const | Get | Concat | Single | Switch | If | DoWhile | Let | Arg | Empty
        | Cons | Nil | Assume | Bop | Uop => {
            // e.g. ["(ExprIsPure x)", "(ExprIsPure y)"]
            let children_pure_queries = ctor.filter_map_fields(|field| match field.purpose {
                Purpose::Static(Sort::BinaryOp)
                | Purpose::Static(Sort::UnaryOp)
                | Purpose::SubExpr
                | Purpose::SubListExpr
                | Purpose::CapturedExpr => Some(format!(
                    "({sort}IsPure {var})",
                    sort = field.sort().name(),
                    var = field.var()
                )),
                Purpose::Static(_) => None,
            });

            // e.g. "(Bop op x y)"
            let ctor_pattern = ctor.construct(|field| field.var());

            let queries = iter::once(ctor_pattern.clone())
                .chain(children_pure_queries)
                .collect::<Vec<_>>()
                .join(" ");

            let sort = ctor.sort().name();
            format!(
                "
                (rule ({queries})
                      (({sort}IsPure {ctor_pattern}))
                      :ruleset always-run)"
            )
        }
        Alloc => "".to_string(),
    }
}

pub(crate) fn rules() -> Vec<String> {
    iter::once(
        "
        (relation ExprIsPure (Expr))
        (relation ListExprIsPure (ListExpr))
        (relation BinaryOpIsPure (BinaryOp))
        (relation UnaryOpIsPure (UnaryOp))"
            .to_string(),
    )
    .chain(BinaryOp::iter().filter_map(|bop| {
        bop_is_pure(&bop).then(|| format!("(BinaryOpIsPure ({name}))", name = bop.name()))
    }))
    .chain(UnaryOp::iter().filter_map(|uop| {
        uop_is_pure(&uop).then(|| format!("(UnaryOpIsPure ({name}))", name = uop.name()))
    }))
    .chain(Constructor::iter().map(purity_rules_for_ctor))
    .collect::<Vec<String>>()
}

// #[test]
// fn test_purity_analysis() -> Result<(), egglog::Error> {
//     let build = &*"
// (let id1 (Id (i64-fresh!)))
// (let id-outer (Id (i64-fresh!)))
// (let pure-loop
//     (Loop id1
//         (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
//         (All id1 (Sequential) (Pair
//             ; pred
//             (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
//             ; output
//             (All id1 (Parallel) (Pair
//                 (Add (Get (Arg id1) 0) (Num id1 1))
//                 (Sub (Get (Arg id1) 1) (Num id1 1))))))))

// (let id2 (Id (i64-fresh!)))
// (let impure-loop
//     (Loop id2
//         (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
//         (All id2 (Sequential) (Pair
//             ; pred
//             (LessThan (Get (Arg id2) 0) (Get (Arg id2) 1))
//             ; output
//             (IgnoreFirst id2
//                 (Print (Num id2 1))
//                 (All id2 (Parallel) (Pair
//                     (Add (Get (Arg id2) 0) (Num id2 1))
//                     (Sub (Get (Arg id2) 1) (Num id2 1)))))))))
//     "
//     .to_string();
//     let check = "
// (check (ExprIsPure pure-loop))
// (fail (check (ExprIsPure impure-loop)))
//     ";
//     crate::run_test(build, check)
// }

// #[test]
// fn test_purity_function() -> Result<(), egglog::Error> {
//     let build = &*"
// (let id1 (Id (i64-fresh!)))
// (let id2 (Id (i64-fresh!)))
// (let id_fun1 (Id (i64-fresh!)))
// (let id_fun2 (Id (i64-fresh!)))
// (let id-outer (Id (i64-fresh!)))

// ;; f1 is pure
// (let f1
//     (Function id_fun1
//             (Add
//                 (Get (Arg id_fun1) 0)
//                 (Get (Arg id_fun1) 0))
//             (TupleT (TCons (IntT) (TNil))) (IntT)))
// ;; f2 is impure
// (let f2
//     (Function id_fun2
//         (Get
//             (All id_fun2 (Sequential)
//                     (Pair
//                     (Print (Get (Arg id_fun2) 0))
//                     (Add
//                         (Get (Arg id_fun2) 0)
//                         (Get (Arg id_fun2) 0))))
//             1)
//         (TupleT (TCons (IntT) (TNil))) (IntT)))
// (let pure-loop
//     (Loop id1
//         (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
//         (All id1 (Sequential) (Pair
//             ; pred
//             (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
//             ; output
//             (All id1 (Parallel)
//                     (Pair
//                     (Add (Call id_fun1 (All id1 (Sequential) (Cons (Get (Arg id1) 0) (Nil)))) (Num id1 1))
//                     (Sub (Get (Arg id1) 1) (Num id1 1))))))))
// (let impure-loop
//     (Loop id2
//         (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
//         (All id2 (Sequential) (Pair
//             ; pred
//             (LessThan (Get (Arg id2) 0) (Get (Arg id2) 1))
//             ; output
//             (All id2 (Parallel)
//                     (Pair
//                     (Add (Call id_fun2 (All id2 (Sequential) (Cons (Get (Arg id2) 0) (Nil)))) (Num id2 1))
//                     (Sub (Get (Arg id2) 1) (Num id2 1))))))))
//     "
//     .to_string();
//     let check = "
// (check (FunctionIsPure id_fun1))
// (fail (check (FunctionIsPure id_fun2)))
// (check (ExprIsPure pure-loop))
// (fail (check (ExprIsPure impure-loop)))
//     ";
//     crate::run_test(build, check)
// }
