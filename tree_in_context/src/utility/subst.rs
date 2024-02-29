#[test]
fn test_subst_nested() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let twoint = tuplet!(intt(), intt());
    let expr = tlet(
        parallel!(int(1), get(funcarg(), 1), tlet(int(2), funcarg())),
        int(0),
    )
    .with_arg_types(twoint.clone(), intt());
    let replace_with = parallel!(int(3), int(4));
    let replacement = in_context(infunc("main"), replace_with.clone());
    let expected = tlet(
        parallel!(
            in_context(infunc("main"), int(1)),
            get(replacement.clone(), 1),
            tlet(in_context(infunc("main"), int(2)), funcarg())
        ),
        int(0),
    )
    .with_arg_types(twoint, intt());

    let build = format!(
        "
(let substituted (Subst (InFunc \"main\")
                        (FuncScope)
                        {replace_with}
                        {expr}))"
    );
    let check = format!("(check (= substituted {expected}))");

    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![
            expr.to_program(tuplet!(intt(), intt()), intt()),
            expected.to_program(tuplet!(intt(), intt()), intt()),
        ],
        Value::Tuple(vec![
            Value::Const(Constant::Int(10)),
            Value::Const(Constant::Int(10)),
        ]),
        Value::Const(Constant::Int(0)),
        vec![],
    )
}

#[test]
fn test_subst_makes_new_context() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let expr = add(
        in_context(infunc("otherfunc"), int(1)),
        in_context(infunc("otherfunc"), int_funcarg()),
    );
    let replace_with = in_context(infunc("main"), int(2));
    let expected = add(
        in_context(infunc("main"), int(1)),
        in_context(infunc("main"), int(2)),
    );
    let build = format!(
        "
(let substituted (Subst (InFunc \"main\") 
                        {replace_with}
                        {expr}))"
    );
    let check = format!("(check (= substituted {expected}))");

    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![
            expr.to_program(intt(), intt()),
            expected.to_program(intt(), intt()),
        ],
        Value::Const(Constant::Int(2)),
        Value::Const(Constant::Int(3)),
        vec![],
    )
}
