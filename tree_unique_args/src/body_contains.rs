use crate::ir::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

/// Builds rules like:
/// ```txt
/// (rule ((Let id in out))
///       ((BodyContains id out))
///       :ruleset always-run)
/// ```
fn captured_expr_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let pat = ctor.construct(|field| field.var());
    let fields = ctor.fields();
    let id_name = fields
        .iter()
        .find(|field| field.purpose == Purpose::CapturingId);
    if id_name.is_none() {
        None
    } else {
        let actions = ctor.filter_map_fields(|field| {
            (field.purpose == Purpose::CapturedExpr).then(|| {
                format!(
                    "(BodyContainsExpr {id} {e})",
                    id = id_name.unwrap().var(),
                    e = field.var()
                )
            })
        });
        let actions_s = actions.join(" ");
        Some(format!("(rule ({pat}) ({actions_s}) :ruleset always-run)"))
    }
}

/// Builds rules like:
/// ```txt
/// (rule ((BodyContainsExpr body_id (Add x y)))
///       ((BodyContainsExpr body_id x)
///        (BodyContainsExpr body_id y))
///       :ruleset always-run)
/// ```
fn subexpr_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let pat = ctor.construct(|field| field.var());
    let actions = ctor.filter_map_fields(|field| {
        (field.purpose == Purpose::SubExpr || field.purpose == Purpose::SubListExpr).then(|| {
            format!(
                "(BodyContains{sort} body_id {e})",
                sort = field.sort().name(),
                e = field.var()
            )
        })
    });
    if actions.is_empty() {
        None
    } else {
        let actions_s = actions.join(" ");
        Some(format!(
            "(rule ((BodyContains{sort} body_id {pat})) ({actions_s}) :ruleset always-run)",
            sort = ctor.sort().name()
        ))
    }
}

pub(crate) fn rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(relation BodyContains* (IdSort *))".replace('*', sort.name()))
        .chain(Constructor::iter().filter_map(captured_expr_rule_for_ctor))
        .chain(Constructor::iter().filter_map(subexpr_rule_for_ctor))
        .collect::<Vec<_>>()
}

#[test]
fn test_body_contains() -> Result<(), egglog::Error> {
    let build = &*"
(let id1 (Id (i64-fresh!)))
(let id-outer (Id (i64-fresh!)))
(let loop
    (Loop id1
        (Num id-outer 1)
        (All id1 (Sequential) (Pair
            ; pred
            (LessThan (Num id1 2) (Num id1 3))
            ; output
            (Switch (Boolean id1 true) (Pair (Num id1 4) (Num id1 5)))))))
    "
    .to_string();
    let check = "
(fail (check (BodyContainsExpr id1 (Num id-outer 1))))
(check (BodyContainsExpr id1 (Num id1 2)))
(check (BodyContainsExpr id1 (Num id1 3)))
(check (BodyContainsExpr id1 (Num id1 4)))
(check (BodyContainsExpr id1 (Num id1 5)))
(check (BodyContainsListExpr id1 (Pair (Num id1 4) (Num id1 5))))
    ";
    crate::run_test(build, check)
}
