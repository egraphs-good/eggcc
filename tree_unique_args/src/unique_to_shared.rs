use crate::ir::{Constructor, SPurpose};
use std::iter::once;
use strum::IntoEnumIterator;

// Builds rules like:
// (rule ((Add x y) (ExprIsPure x) (ExprIsPure y))
//       ((ExprIsPure (Add x y)))
//       :ruleset always-run)
pub(crate) fn rules() -> String {
    once(
        "(function ToSExpr (Expr) SExpr)\n\
         (function ToListSExpr (ListExpr) ListSExpr)"
            .to_string(),
    )
    .chain(Constructor::iter().map(|ctor| {
        format!(
            "(rewrite (To{ssort} {expr})\
                      {sexpr}
                      :ruleset always-run)",
            // sort = ctor.sort().name(),
            ssort = ctor.sort_shared().name(),
            expr = ctor.construct(|field| field.var()),
            sexpr = ctor.construct_shared(|sfield| match sfield.purpose {
                SPurpose::CapturedExpr | SPurpose::SubExpr | SPurpose::SubListExpr => format!(
                    "(To{ssort} {var})",
                    ssort = sfield.sort().name(),
                    var = sfield.var()
                ),
                SPurpose::Static(_) => sfield.var(),
            })
        )
    }))
    .collect::<Vec<_>>()
    .join("\n")
}

#[test]
fn test_unique_to_shared() -> Result<(), egglog::Error> {
    let build = "
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let loop1
    (Loop id1
        (All (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All (Parallel) (Pair
                (Add (Get (Arg id1) 0) (Num id1 1))
                (Sub (Get (Arg id1) 1) (Num id1 1))))))))
(let loop1-shared (ToSExpr loop1))
    ";
    let check = "
(let loop1-shared-expected
    (SLoop
        (SAll (Parallel) (SPair (SNum 0) (SNum 0)))
        (SAll (Sequential) (SPair
            ; pred
            (SLessThan (SGet (SArg) 0) (SGet (SArg) 1))
            ; output
            (SAll (Parallel) (SPair
                (SAdd (SGet (SArg) 0) (SNum 1))
                (SSub (SGet (SArg) 1) (SNum 1))))))))

(run-schedule (saturate always-run)) ; degugar SPair
(check (= loop1-shared loop1-shared-expected))
    ";
    crate::run_test(build, check)
}
