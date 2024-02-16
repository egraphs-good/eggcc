#[test]
fn test_subst_nested() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let twoint = tuplet!(intt(), intt());
    let expr = tlet(
        parallel!(int(1), get(arg(), 1), tlet(int(2), arg())),
        int(0),
    )
    .with_arg_types(twoint.clone(), intt());
    let replace_with = parallel!(int(3), int(4));
    let replacement = assume(infunc("main"), replace_with.clone());
    let expected = tlet(
        parallel!(
            assume(infunc("main"), int(1)),
            get(replacement.clone(), 1),
            tlet(assume(infunc("main"), int(2)), arg())
        ),
        int(0),
    )
    .with_arg_types(twoint, intt());

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
            expr.to_program(tuplet!(intt(), intt()), intt()),
            expected.to_program(tuplet!(intt(), intt()), intt()),
        ],
        Value::Tuple(vec![
            Value::Const(Constant::Int(10)),
            Value::Const(Constant::Int(10)),
        ]),
        Value::Const(Constant::Int(0)),
    )
}

#[test]
fn test_subst_makes_new_context() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let expr = add(
        assume(infunc("otherfunc"), int(1)),
        assume(infunc("otherfunc"), int_arg()),
    );
    let replace_with = assume(infunc("main"), int(2));
    let expected = add(
        assume(infunc("main"), int(1)),
        assume(infunc("main"), int(2)),
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
    )
}
