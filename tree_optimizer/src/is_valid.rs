use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn rule_for_ctor(ctor: Constructor) -> Option<String> {
    let actions = ctor.filter_map_fields(|field| match field.purpose {
        Purpose::Static(_) | Purpose::CapturingId | Purpose::ReferencingId => None,
        Purpose::CapturedExpr | Purpose::SubExpr | Purpose::SubListExpr => Some(format!(
            "({sort}IsValid {var})",
            sort = field.sort().name(),
            var = field.var()
        )),
    });

    if actions.is_empty() {
        return None;
    }
    let actions = actions.join("\n");
    let pat = ctor.construct(|field| field.var());
    let sort = ctor.sort().name();
    Some(format!(
        "(rule (({sort}IsValid {pat})) ({actions}) :ruleset always-run)"
    ))
}

pub(crate) fn rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| {
            format!(
                "
(relation {sort}IsValid ({sort}))
"
            )
        })
        .chain(Constructor::iter().filter_map(rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_is_valid() -> Result<(), egglog::Error> {
    let build = &*"
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let loop
    (Loop id1
        (All id-outer (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All id1 (Sequential) (Pair
            ; pred
            (BOp (LessThan) (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All id1 (Parallel) (Pair
                (BOp (Add) (Get (Arg id1) 0) (Num id1 1))
                (BOp (Sub) (Get (Arg id1) 1) (Num id1 1))))))))
(ExprIsValid loop)
(let bad-expr (BOp (Sub) (Arg id1) (Arg id-outer)))
    "
    .to_string();
    let check = "
(check (ExprIsValid (Num id-outer 0)))
(check (ExprIsValid (Arg id1)))
(check (ListExprIsValid
         (Pair (BOp (Add) (Get (Arg id1) 0) (Num id1 1))
               (BOp (Sub) (Get (Arg id1) 1) (Num id1 1)))))
(fail (check (ExprIsValid bad-expr)))
    ";
    crate::run_test(build, check)
}
