use std::iter;

use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn deep_copy_rule_for_ctor(ctor: Constructor) -> String {
    // e.g. "Add x y"
    let ctor_pattern = iter::once(ctor.name())
        .chain(ctor.fields().iter().map(|field| field.name))
        .collect::<Vec<_>>()
        .join(" ");

    // e.g. ["(DeepCopyExpr x new-id)", "(DeepCopyExpr y new-id)"]
    let substed_fields = ctor
        .fields()
        .iter()
        .map(|field| match field.purpose {
            Purpose::CapturingId => "new-inner-id".to_string(),
            Purpose::Static(_) => field.name.to_string(),
            Purpose::CapturedExpr => {
                let var = field.name;
                let sort = field.sort().name();
                format!("(DeepCopy{sort} {var} new-inner-id)")
            }
            Purpose::ReferencingId => "new-id".to_string(),
            Purpose::SubExpr | Purpose::SubListExpr => {
                let var = field.name;
                let sort = field.sort().name();
                format!("(DeepCopy{sort} {var} new-id)")
            }
        })
        .collect::<Vec<_>>();

    // e.g. "Add (DeepCopyExpr x new-id) (DeepCopyExpr y new-id)"
    let copied_ctor = iter::once(ctor.name().to_string())
        .chain(substed_fields.into_iter())
        .collect::<Vec<_>>()
        .join(" ");

    let sort = ctor.sort().name();
    let br = "\n      ";
    let creates_context = ctor
        .fields()
        .iter()
        .any(|field| field.purpose == Purpose::CapturingId);
    let actions = if creates_context {
        format!("(let new-inner-id (i64-fresh!)){br} (union e ({copied_ctor}))")
    } else {
        format!("(union e ({copied_ctor}))")
    };
    format!(
        "(rule ((= e (DeepCopy{sort} ({ctor_pattern}) new-id))){br}({actions}){br}:ruleset always-run)"
    )
}

pub(crate) fn deep_copy_rules() -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for sort in ESort::iter() {
        let sort_name = sort.name();
        res.push(format!(
            "(function DeepCopy{sort_name} ({sort_name} i64) {sort_name} :unextractable)"
        ));
    }
    res.extend(Constructor::iter().map(deep_copy_rule_for_ctor));
    res
}

// We use field names as var names, and bind "v" to the value being substituted
// in, so this test checks we don't overlap/add extra equality constraints
#[test]
fn var_names_available() {
    for ctor in Constructor::iter() {
        for field in ctor.fields() {
            assert_ne!(field.name, "new-id");
            assert_ne!(field.name, "new-inner-id");
            assert_ne!(field.name, "e");
        }
    }
}

#[test]
fn test_deep_copy() -> Result<(), egglog::Error> {
    let build = "
(let id1 (i64-fresh!))
(let id2 (i64-fresh!))
(let loop
    (Loop id1
        (All (Parallel) (Pair (Arg id1) (Num 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (Let id2
                (All (Parallel) (Pair
                    (Add (Get (Arg id1) 0) (Num 1))
                    (Sub (Get (Arg id1) 1) (Num 1))))
                (Arg id2))))))
(let loop-copied (DeepCopyExpr loop (i64-fresh!)))
    ";
    let check = "
(let loop-copied-expected
    (Loop 3
        (All (Parallel) (Pair (Arg 2) (Num 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg 3) 0) (Get (Arg 3) 1))
            ; output
            (Let 4
                (All (Parallel) (Pair
                    (Add (Get (Arg 3) 0) (Num 1))
                    (Sub (Get (Arg 3) 1) (Num 1))))
                (Arg 4))))))
(run-schedule (saturate always-run))
(check (= loop-copied loop-copied-expected))
    ";
    crate::run_test(build, check)
}
