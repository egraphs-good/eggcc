#[test]
fn test_subst_nested() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let twoint = tuplet!(intt(), intt());
    let expr = get(
        dowhile(
            parallel!(
                int(1),
                get(arg(), 1),
                get(
                    dowhile(single(get(arg(), 0)), parallel!(tfalse(), get(arg(), 0))),
                    0
                )
            ),
            parallel!(tfalse(), int(0), int(1), int(2)),
        ),
        0,
    )
    .with_arg_types(twoint.clone(), base(intt()));
    let replace_with = parallel!(int(3), int(4)).with_arg_types(twoint.clone(), twoint.clone());
    let replacement = in_context(infunc("main"), replace_with.clone());
    let expected = get(
        dowhile(
            parallel!(
                in_context(infunc("main"), int(1)),
                get(replacement.clone(), 1),
                get(
                    dowhile(
                        single(get(replacement.clone(), 0)),
                        parallel!(tfalse(), get(arg(), 0))
                    ),
                    0
                )
            ),
            parallel!(tfalse(), int(0), int(1), int(2)),
        ),
        0,
    )
    .with_arg_types(twoint.clone(), base(intt()));

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
            expr.to_program(twoint.clone(), base(intt())),
            expected.to_program(tuplet!(intt(), intt()), base(intt())),
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
        in_context(infunc("otherfunc"), int_ty(1, base(intt()))),
        in_context(infunc("otherfunc"), iarg()),
    );
    let replace_with = int_ty(2, base(intt()));
    let expected = add(
        in_context(infunc("main"), int(1)),
        in_context(infunc("main"), int(2)),
    )
    .with_arg_types(base(intt()), base(intt()));
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
            expr.to_program(base(intt()), base(intt())),
            expected.to_program(base(intt()), base(intt())),
        ],
        Value::Const(Constant::Int(2)),
        Value::Const(Constant::Int(3)),
        vec![],
    )
}

#[test]
fn test_subst_arg_type_changes() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let expr = add(iarg(), iarg());
    let tupletype = tuplet!(intt(), intt());
    let replace_with = get(arg(), 0).with_arg_types(tupletype.clone(), base(intt()));

    let expected = add(
        in_context(infunc("main"), get(arg(), 0)),
        in_context(infunc("main"), get(arg(), 0)),
    )
    .with_arg_types(tupletype.clone(), base(intt()));
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
        vec![expr.to_program(base(intt()), base(intt()))],
        Value::Const(Constant::Int(2)),
        Value::Const(Constant::Int(4)),
        vec![],
    )
}

#[test]
fn test_subst_preserves_context() -> crate::Result {
    use crate::ast::*;

    let outer_if = tif(ttrue(), tif(tfalse(), int(1), arg()), arg());
    let expression = function("main", base(intt()), base(intt()), outer_if)
        .func_with_arg_types()
        .func_add_context();

    let replace_with = int(5).with_arg_types(base(intt()), base(intt()));

    let expected = function(
        "main",
        base(intt()),
        base(intt()),
        tif(ttrue(), tif(tfalse(), int(1), int(5)), int(5)),
    )
    .func_with_arg_types()
    .func_add_context();

    let build = format!(
        "
(let substituted (Subst (InFunc \"main\")
                        {replace_with}
                        {expression}))"
    );
    let check = format!("(check (= substituted {expected}))");
    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![expression.func_to_program(), expected.func_to_program()],
        intv(5),
        intv(5),
        vec![],
    )
}
