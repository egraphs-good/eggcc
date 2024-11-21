//! IsResolved is helper that markes every eclass that has at least one grounded version of
//! the program available. That means it doesn't have any helpers, like Subst in it.
//! ex: (Add (Subst ...) (Int 2)) is not resolved, but (Add (Int 2) (Int 2)) is resolved.
//! IsValid is like IsResolved, but top-down instead of bottom-up. It marks eclasses with resolved parents.

use crate::schema_helpers::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn rule_for_ctor(ctor: Constructor) -> String {
    let query = ctor
        .filter_map_fields(|field| match field.purpose {
            Purpose::Static(_) => None,
            Purpose::CapturedExpr | Purpose::SubExpr | Purpose::CapturedSubListExpr => {
                Some(format!(
                    "({sort}IsResolved {var})",
                    sort = field.sort().name(),
                    var = field.var()
                ))
            }
        })
        .join("\n");
    let pat = ctor.construct(|field| field.var());
    let sort = ctor.sort().name();
    format!("(rule ((= lhs {pat}) {query}) (({sort}IsResolved lhs)) :ruleset is-resolved)")
}

pub(crate) fn rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(relation *IsResolved (*))".replace('*', sort.name()))
        .chain(Constructor::iter().map(rule_for_ctor))
        .collect::<Vec<_>>()
}

#[cfg(test)]
use crate::ast::*;
#[cfg(test)]
use crate::schema::Constant;
#[cfg(test)]
use crate::Value;

#[test]
fn test_is_resolved() -> crate::Result {
    use crate::schedule::helpers;
    use crate::schema::Assumption;
    let myloop = dowhile(
        single(int(1)),
        parallel!(
            less_than(get(arg(), 0), int(3)),
            get(switch!(int(0), arg(); parallel!(int(4), int(5))), 0)
        ),
    )
    .with_arg_types(base(intt()), tuplet!(intt()));
    let add1 = add(arg(), int(1)).add_arg_type(base(intt()));
    let helpers = helpers();
    let build = format!("{myloop}");
    let ctx = Assumption::dummy();
    let check = format!(
        "
(check (ExprIsResolved myloop))

(let substituted (Subst {ctx} {add1} {myloop}))
;; run the IsResolved rules
(run-schedule (saturate is-resolved))

(check (ExprIsResolved {add1}))
;; substitution hasn't happened
(fail (check (ExprIsResolved substituted)))

(run-schedule {helpers})
(check (ExprIsResolved substituted))
    ",
    );
    crate::egglog_test(
        &build,
        &check,
        vec![myloop.to_program(base(intt()), tuplet!(intt()))],
        intv(1),
        // Value::Tuple(vec![]),
        Value::Tuple(vec![Value::Const(Constant::Int(4))]),
        vec![],
    )
}
