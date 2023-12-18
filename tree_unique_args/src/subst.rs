use std::iter;

use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn subst_rule_for_ctor(ctor: Constructor) -> String {
    if ctor == Constructor::Arg {
        return "(rewrite (SubstExpr (Arg id) v) v :ruleset subst)".to_string();
    }
    let ctor_pattern_without_parens = iter::once(ctor.name())
        .chain(ctor.fields().iter().map(|field| field.name))
        .collect::<Vec<_>>()
        .join(" ");

    // e.g. "(Add x y)"
    let ctor_pattern = format!("({ctor_pattern_without_parens})");

    // e.g. ["(SubstExpr x v)", "(SubstExpr y v)"]
    let substed_fields = ctor
        .fields()
        .iter()
        .map(|field| match field.purpose {
            Purpose::Static(_) | Purpose::CapturingId | Purpose::CapturedExpr => {
                field.name.to_string()
            }
            Purpose::ReferencingId => panic!("arg case already handled"),
            Purpose::SubExpr | Purpose::SubListExpr => {
                let var = field.name;
                let sort = field.sort().name();
                format!("(Subst{sort} {var} v)")
            }
        })
        .collect::<Vec<_>>();

    // e.g. "Add (SubstExpr x v) (SubstExpr y v)"
    let substed_ctor = iter::once(ctor.name().to_string())
        .chain(substed_fields.into_iter())
        .collect::<Vec<_>>()
        .join(" ");

    let sort = ctor.sort().name();
    let br = "\n         ";
    format!("(rewrite (Subst{sort} {ctor_pattern} v){br}({substed_ctor}){br}:ruleset subst)")
}

pub(crate) fn subst_rules() -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for sort in ESort::iter() {
        let sort_name = sort.name();
        res.push(format!(
            "(function Subst{sort_name} ({sort_name} Expr) {sort_name})"
        ));
    }
    res.extend(Constructor::iter().map(subst_rule_for_ctor));
    res
}

// We use field names as var names, as use v in the same rules to mean the
// value being substituted in, so this test makes sure we don't add a
// constructor that overlaps accidentally.
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
(run-schedule (saturate desugar))
(check (= loop1-substed loop1-substed-expected))
    ";
    crate::run_test(build, check)
}
