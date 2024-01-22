use std::iter;

use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn rules_for_ctor(ctor: Constructor) -> Option<String> {
    if ctor.sort() != ESort::Expr || ctor.creates_context() {
        return None;
    }
    Some(ctor.filter_map_fields(|varying_field| match varying_field.purpose {
        Purpose::Static(_) | Purpose::CapturingId | Purpose::CapturedExpr | Purpose::ReferencingId | Purpose::SubListExpr => None,
        Purpose::SubExpr  => {
            let ctor_name = ctor.name();
            let varying_field_name = varying_field.name;
            let relation = format!("Same{ctor_name}Ignoring-{varying_field_name}");
            let mk_pattern = |varying_field_pat: String| ctor.construct(|field| {
                if field == varying_field {
                    varying_field_pat.clone()
                } else {
                    field.var()
                }
            });
            let ctor_pattern1 = mk_pattern("e1".to_string());
            let ctor_pattern2 = mk_pattern("e2".to_string());
            let resulting_switch = mk_pattern(format!("(Switch pred (Map-{ctor_name}-{varying_field_name} exprs))"));
            Some(format!(
                "
                ; Compute {relation}, which detects opportunities for lifting
                ; {ctor_name}s through control flow when only the {varying_field_name} field varies
                (relation {relation} (ListExpr))
                (rule ((DemandSameIgnoring (Cons {ctor_pattern1} (Nil))))
                      (({relation} (Cons {ctor_pattern1} (Nil))))
                      :ruleset always-run)
                (rule ((DemandSameIgnoring (Cons {ctor_pattern1} (Cons {ctor_pattern2} rest)))
                       ({relation} (Cons {ctor_pattern2} rest)))
                      (({relation} (Cons {ctor_pattern1} (Cons {ctor_pattern2} rest))))
                      :ruleset always-run)

                ; Given a list of {ctor_name}s, return a list of each {ctor_name}'s {varying_field_name} field
                (function Map-{ctor_name}-{varying_field_name} (ListExpr) ListExpr)
                (rewrite (Map-{ctor_name}-{varying_field_name} (Nil)) (Nil) :ruleset always-run)
                (rewrite (Map-{ctor_name}-{varying_field_name} (Cons {ctor_pattern1} rest))
                         (Cons e1 (Map-{ctor_name}-{varying_field_name} rest))
                         :ruleset always-run)

                ; Lift {ctor_name} when only {varying_field_name} varies
                (rule ((ExprIsValid (Switch pred exprs))
                       ({relation} exprs)
                       ; Bind non-varying field(s)
                       (= list (Cons {ctor_pattern1} rest)))
                      ((union (Switch pred exprs)
                              {resulting_switch}))
                      :ruleset conditional-invariant-code-motion)"
            ))
        }
    })
    .join("\n"))
}

pub(crate) fn rules() -> Vec<String> {
    iter::once(
        "
        (ruleset conditional-invariant-code-motion)
        (relation DemandSameIgnoring (ListExpr))
        (rule ((DemandSameIgnoring (Cons hd tl))) ((DemandSameIgnoring tl)) :ruleset always-run)
        (rule ((Switch pred exprs)) ((DemandSameIgnoring exprs)) :ruleset always-run)"
            .to_string(),
    )
    .chain(Constructor::iter().filter_map(rules_for_ctor))
    .collect::<Vec<_>>()
}

#[test]
fn var_names_available() {
    for ctor in Constructor::iter() {
        for field in ctor.fields() {
            for var_name in ["e", "e1", "e2", "rest", "pred", "exprs"] {
                assert_ne!(field.var(), var_name);
            }
        }
    }
}

#[test]
fn test_easy_lift_switch() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
(let id1 (Id (i64-fresh!)))
(let switch1
    (Switch
        (Num id1 1)
        (Pair
            (LessThan (Get (Arg id1) 0) (Num id1 7))
            (LessThan (Get (Arg id1) 1) (Num id1 7))
        )))
(ExprIsValid switch1)
    "
    );
    let check = "
(let switch1-lifted-expected
    (LessThan
        (Switch
            (Num id1 1)
            (Pair
                (Get (Arg id1) 0)
                (Get (Arg id1) 1)
            ))
        (Num id1 7)))
(run-schedule (saturate always-run))
(check (= switch1 switch1-lifted-expected))
    ";
    crate::run_test(build, check)
}

#[test]
fn test_lift_switch_through_switch() -> Result<(), egglog::Error> {
    let build = &*format!(
        "
(let id1 (Id (i64-fresh!)))
(let switch1
    (Switch
        (Num id1 0)
        (Pair
            (Switch (LessThan (Get (Arg id1) 0) (Num id1 7)) (Cons (Num id1 11) (Nil)))
            (Switch (LessThan (Get (Arg id1) 1) (Num id1 7)) (Cons (Num id1 11) (Nil)))
        )))
(ExprIsValid switch1)
    "
    );
    let check = "
(let all1-lifted-expected
    (Switch
        (LessThan
            (Switch
                (Num id1 0)
                (Pair
                    (Get (Arg id1) 0)
                    (Get (Arg id1) 1)
                ))
            (Num id1 7))
        (Cons (Num id1 11) (Nil))))
(run-schedule (saturate always-run))
(check (= all1 all1-lifted-expected))
    ";
    crate::run_test(build, check)
}
