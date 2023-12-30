use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn subst_rule_for_ctor(ctor: Constructor) -> String {
    if ctor == Constructor::Arg {
        return "(rewrite (SubstExpr (Arg id) v) v :ruleset always-run)".to_string();
    }

    // e.g. "(Add x y)"
    let ctor_pattern = ctor.construct(|field| field.name.to_string());

    // e.g. "(Add (SubstExpr x v) (SubstExpr y v))"
    let substed_ctor = ctor.construct(|field| match field.purpose {
        Purpose::Static(_) | Purpose::CapturingId | Purpose::CapturedExpr => field.name.to_string(),
        Purpose::ReferencingId => panic!("arg case already handled"),
        Purpose::SubExpr | Purpose::SubListExpr => {
            let var = field.name;
            let sort = field.sort().name();
            format!("(Subst{sort} {var} v)")
        }
    });

    let sort = ctor.sort().name();
    let br = "\n         ";
    format!("(rewrite (Subst{sort} {ctor_pattern} v){br}{substed_ctor}{br}:ruleset always-run)")
}

pub(crate) fn subst_rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(function Subst* (* Expr) * :unextractable)".replace("*", sort.name()))
        .chain(Constructor::iter().map(subst_rule_for_ctor))
        .collect::<Vec<_>>()
}

// We use field names as var names, and bind "v" to the value being substituted
// in, so this test checks we don't overlap/add extra equality constraints
#[test]
fn var_names_available() {
    for ctor in Constructor::iter() {
        for field in ctor.fields() {
            assert_ne!(field.name, "v");
        }
    }
}

#[test]
fn test_subst() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
(let id1 (i64-fresh!))
(let loop1
    (Loop id1
        (All (Parallel) (Pair (Arg id1) (Num 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All (Parallel) (Pair
                (Add (Get (Arg id1) 0) (Num 1))
                (Sub (Get (Arg id1) 1) (Num 1))))))))
(let loop1-substed (SubstExpr loop1 (Num 7)))
    "
    );
    let check = "
(let loop1-substed-expected
    (Loop id1
        (All (Parallel) (Pair (Num 7) (Num 0)))
        (All (Sequential) (Pair
            ; pred
            (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
            ; output
            (All (Parallel) (Pair
                (Add (Get (Arg id1) 0) (Num 1))
                (Sub (Get (Arg id1) 1) (Num 1))))))))
(run-schedule (saturate always-run))
(check (= loop1-substed loop1-substed-expected))
    ";
    crate::run_test(build, check)
}
