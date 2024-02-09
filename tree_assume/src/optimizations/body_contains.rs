use crate::schema_helpers::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

/// Builds rules like:
/// ```txt
/// (rule ((Let in out))
///       ((BodyContains (Let in out) out))
///       :ruleset always-run)
/// ```
fn captured_expr_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let pat = ctor.construct(|field| field.var());
    let actions = ctor.filter_map_fields(|field| {
        (field.purpose == Purpose::CapturedExpr)
            .then(|| format!("(BodyContainsExpr {pat} {e})", e = field.var()))
    });
    if actions.is_empty() {
        None
    } else {
        let actions_s = actions.join(" ");
        Some(format!("(rule ({pat}) ({actions_s}) :ruleset always-run)"))
    }
}

/// Builds rules like:
/// ```txt
/// (rule ((BodyContainsExpr body (Add x y)))
///       ((BodyContainsExpr body x)
///        (BodyContainsExpr body y))
///       :ruleset always-run)
/// ```
fn subexpr_rule_for_ctor(ctor: Constructor) -> Option<String> {
    let pat = ctor.construct(|field| field.var());
    let actions = ctor.filter_map_fields(|field| {
        (field.purpose == Purpose::SubExpr || field.purpose == Purpose::SubListExpr).then(|| {
            format!(
                "(BodyContains{sort} body {e})",
                sort = field.sort().name(),
                e = field.var()
            )
        })
    });
    (!actions.is_empty()).then(|| {
        format!(
            "(rule ((BodyContains{sort} body {pat})) ({actions_s}) :ruleset always-run)",
            sort = ctor.sort().name(),
            actions_s = actions.join(" ")
        )
    })
}

pub(crate) fn rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(relation BodyContains* (Expr *))".replace('*', sort.name()))
        .chain(Constructor::iter().filter_map(captured_expr_rule_for_ctor))
        .chain(Constructor::iter().filter_map(subexpr_rule_for_ctor))
        .collect::<Vec<_>>()
}

#[cfg(test)]
use crate::ast::*;
#[cfg(test)]
use crate::Value;
#[test]
fn test_body_contains() -> Result<(), egglog::Error> {
    let myloop = dowhile(
        int(1),
        parallel!(
            less_than(arg(), int(3)),
            switch!(int(0); parallel!(int(4), int(5)))
        ),
    );
    let build = format!("{myloop}");
    let check = format!(
        "
(fail (check (BodyContainsExpr {myloop} {num1})))
(check (BodyContainsExpr {myloop} {num3}))
(check (BodyContainsExpr {myloop} {num4}))
(check (BodyContainsExpr {myloop} {num5}))
(check (BodyContainsListExpr {myloop} (Cons {tup45} (Nil))))
    ",
        num1 = int(1),
        num3 = int(3),
        num4 = int(4),
        num5 = int(5),
        tup45 = parallel!(int(4), int(5)),
    );
    crate::egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
    )
}
