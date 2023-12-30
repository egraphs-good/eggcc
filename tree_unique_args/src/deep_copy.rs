use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn deep_copy_rule_for_ctor(ctor: Constructor) -> String {
    // e.g. "(Add x y)"
    let ctor_pattern = ctor.construct(|field| field.name.to_string());

    // e.g. "(Add (DeepCopyExpr x new-id)", "(DeepCopyExpr y new-id))"
    let copied_ctor = ctor.construct(|field| match field.purpose {
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
    });

    let sort = ctor.sort().name();
    let br = "\n      ";
    let actions = if ctor.creates_context() {
        format!("(let new-inner-id (i64-fresh!)){br} (union e {copied_ctor})")
    } else {
        format!("(union e {copied_ctor})")
    };
    format!(
        "(rule ((= e (DeepCopy{sort} {ctor_pattern} new-id))){br}({actions}){br}:ruleset always-run)"
    )
}

pub(crate) fn deep_copy_rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(function DeepCopy* (* i64) * :unextractable)".replace("*", sort.name()))
        .chain(Constructor::iter().map(deep_copy_rule_for_ctor))
        .collect::<Vec<_>>()
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
