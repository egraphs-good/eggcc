use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn deep_copy_rule_for_ctor(ctor: Constructor) -> String {
    // e.g. "(Add x y)"
    let ctor_pattern = ctor.construct(|field| field.var());

    // e.g. "(Add (DeepCopyExpr x new-id)", "(DeepCopyExpr y new-id))"
    let copied_ctor = ctor.construct(|field| match field.purpose {
        Purpose::CapturingId => "new-inner-id".to_string(),
        Purpose::Static(_) => field.var(),
        Purpose::CapturedExpr => {
            let var = field.var();
            let sort = field.sort().name();
            format!("(DeepCopy{sort} {var} new-inner-id)")
        }
        Purpose::ReferencingId => "new-id".to_string(),
        Purpose::SubExpr | Purpose::SubListExpr => {
            let var = field.var();
            let sort = field.sort().name();
            format!("(DeepCopy{sort} {var} new-id)")
        }
    });

    let sort = ctor.sort().name();
    let br = "\n      ";
    let actions = if ctor.creates_context() {
        format!("(let new-inner-id (Id (i64-fresh!))){br} (union e {copied_ctor})")
    } else {
        format!("(union e {copied_ctor})")
    };
    format!(
        "(rule ((= e (DeepCopy{sort} {ctor_pattern} new-id))){br}({actions}){br}:ruleset always-run)"
    )
}

pub(crate) fn deep_copy_rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(function DeepCopy* (* IdSort) * :unextractable)".replace('*', sort.name()))
        .chain(Constructor::iter().map(deep_copy_rule_for_ctor))
        .collect::<Vec<_>>()
}

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
(let id1 (Id (i64-fresh!)))
(let id2 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let loop
    (Loop id1
        (All (Parallel) (Pair (Arg id-outer) (Num id-outer 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (Let id2
                (All (Parallel) (Pair
                    (Add (Get (Arg id1) 0) (Num id1 1))
                    (Sub (Get (Arg id1) 1) (Num id1 1))))
                (Arg id2))))))
(let loop-copied (DeepCopyExpr loop (Id (i64-fresh!))))
    ";
    let check = "
(run-schedule (saturate always-run))
(extract loop-copied)
(check (= loop-copied
          (Loop new-id
        (All (Parallel) (Pair (Arg some-other-id) (Num some-other-id 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg new-id) 0) (Get (Arg new-id) 1))
            ; output
            (Let third-new-id
                (All (Parallel) (Pair
                    (Add (Get (Arg new-id) 0) (Num new-id 1))
                    (Sub (Get (Arg new-id) 1) (Num new-id 1))))
                (Arg third-new-id))))))
            )
    ";
    crate::run_test(build, check)
}
