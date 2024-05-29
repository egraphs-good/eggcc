#[cfg(test)]
use crate::{egglog_test, interpreter::Value};

#[test]
fn test_wildcard() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;
    let in_ty = tuplet!(intt(), intt());
    let expr = add(getat(0), getat(1))
        .with_arg_types(in_ty.clone(), base(intt()))
        .add_ctx(Assumption::dummy());
    let expr_with_wildcard = add(getat(0), getat(1))
        .with_arg_types(in_ty.clone(), base(intt()))
        .add_ctx(wildcardctx("CTX".to_string()));

    let build = format!("(let expr {})\n{}", expr.value, expr.get_unions());
    let check = format!(
        "{}\n(check {})",
        expr_with_wildcard.get_unions(),
        expr_with_wildcard.value
    );

    egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
