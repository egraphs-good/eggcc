use crate::schema_helpers::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

fn rule_for_ctor(ctor: Constructor) -> Option<String> {
    let actions = ctor.filter_map_fields(|field| match field.purpose {
        Purpose::Static(_) => None,
        Purpose::CapturedExpr | Purpose::SubExpr | Purpose::SubListExpr => Some(format!(
            "({sort}IsValid {var})",
            sort = field.sort().name(),
            var = field.var()
        )),
    });

    if actions.is_empty() {
        return None;
    }
    let actions = actions.join("\n");
    let pat = ctor.construct(|field| field.var());
    let sort = ctor.sort().name();
    Some(format!(
        "(rule (({sort}IsValid {pat})) ({actions}) :ruleset always-run)"
    ))
}

pub(crate) fn rules() -> Vec<String> {
    ESort::iter()
        .map(|sort| "(relation *IsValid (*))".replace('*', sort.name()))
        .chain(Constructor::iter().filter_map(rule_for_ctor))
        .collect::<Vec<_>>()
}
#[cfg(test)]
use crate::ast::*;
#[cfg(test)]
use crate::schema::Constant;
#[cfg(test)]
use crate::Value;

#[test]
fn test_is_valid() -> Result<(), egglog::Error> {
    let myloop = dowhile(
        in_context(inlet(int(2)), single(int(1))),
        parallel!(
            less_than(get(looparg(), 0), int(3)),
            get(switch!(int(0); parallel!(int(4), int(5))), 0)
        ),
    )
    .with_arg_types(emptyt(), tuplet!(intt()));
    // this expression is valid (it uses only IR constructors)
    // but it isn't a sub-expression of the initial one, so it won't be
    // marked as valid.
    let not_made_valid = sub(looparg(), looparg()).with_arg_types(intt(), intt());
    let build = format!("(ExprIsValid {myloop}) {not_made_valid}");
    let check = format!(
        "
(check (ExprIsValid {num0}))
(check (ExprIsValid {arg}))
(check (ListExprIsValid (Cons {tup45} (Nil))))
(fail (check (ExprIsValid {not_made_valid})))
(fail (check (ExprIsValid {num2})))
    ",
        num0 = int(0),
        num2 = int(2),
        arg = looparg().with_arg_types(tuplet!(intt()), tuplet!(intt())),
        tup45 = parallel!(int(4), int(5)),
    );
    crate::egglog_test(
        &build,
        &check,
        vec![myloop.to_program(emptyt(), tuplet!(intt()))],
        Value::Tuple(vec![]),
        // Value::Tuple(vec![]),
        Value::Tuple(vec![Value::Const(Constant::Int(4))]),
        vec![],
    )
}
