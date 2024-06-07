#[cfg(test)]
use crate::{egglog_test, interpreter::Value};

#[test]
fn test_wildcard() -> crate::Result {
    use crate::ast::*;

    let in_ty = tuplet!(intt(), intt());
    let (expr, expr_cache) = add(getat(0), getat(1))
        .with_arg_types(in_ty.clone(), base(intt()))
        .add_dummy_ctx();
    let (expr_with_wildcard, wildcard_cache) = add(getat(0), getat(1))
        .with_arg_types(in_ty.clone(), base(intt()))
        .add_ctx(wildcardctx("CTX".to_string()));

    let build = format!("(let expr {expr})\n{}", expr_cache.get_unions());
    let check = format!(
        "{}\n(check {expr_with_wildcard})",
        wildcard_cache.get_unions()
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
