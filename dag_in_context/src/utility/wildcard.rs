#[cfg(test)]
use crate::{egglog_test, interpreter::Value};

#[test]
fn test_wildcard() -> crate::Result {
    use crate::ast::*;
    let in_ty = tuplet!(intt(), intt());
    let expr = add(getat(0), getat(1))
        .with_arg_types(in_ty.clone(), base(intt()))
        .add_ctx(Assumption::dummy());
    let expr_with_wildcard = add(getat(0), getat(1))
        .with_arg_types(in_ty.clone(), base(intt()))
        .add_ctx(wildcardctx("CTX".to_string()));

    let build = format!("(let expr {})", expr);
    let check = format!("(check {})", expr_with_wildcard,);

    egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
