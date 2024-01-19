use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn rule_for_ctor(ctor: Constructor) -> Option<String> {
    let actions = ctor.filter_map_fields(|field| match field.purpose {
        Purpose::Static(_) | Purpose::CapturingId | Purpose::ReferencingId => None,
        Purpose::CapturedExpr | Purpose::SubExpr | Purpose::SubListExpr => Some(format!(
            "({sort}ShouldBeValid {var})",
            sort = field.sort().name(),
            var = field.var()
        )),
    });

    if actions.is_empty() {
        return None;
    }
    let actions = actions.join("\n");
    let pat = ctor.construct(|field| field.var());
    Some(format!("(rule ({pat}) ({actions}) :ruleset always-run)"))
}

pub(crate) fn rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(relation *ShouldBeValid (*))".replace('*', sort.name()))
        .chain(Constructor::iter().filter_map(rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_should_be_valid() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let loop
    (Loop id1
        (All (Parallel) (Pair (Num id-outer 0) (Num id-outer 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All (Parallel) (Pair
                (Add (Get (Arg id1) 0) (Num id1 1))
                (Sub (Get (Arg id1) 1) (Num id1 1))))))))
(ExprShouldBeValid loop)
    "
    );
    let check = "
(check (ExprShouldBeValid (Num id-outer 0)))
(check (ExprShouldBeValid (Arg id1)))
(check (ListExprShouldBeValid
         (Pair (Add (Get (Arg id1) 0) (Num id1 1))
               (Sub (Get (Arg id1) 1) (Num id1 1)))))
    ";
    crate::run_test(build, check)
}
