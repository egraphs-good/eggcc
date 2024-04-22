#[test]
fn test_subst_nested() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let twoint = tuplet!(intt(), intt());
    let inputs = parallel!(
        int(1),
        get(arg(), 1),
        get(
            dowhile(single(get(arg(), 0)), parallel!(tfalse(), get(arg(), 0))),
            0
        )
    )
    .with_arg_types(twoint.clone(), tuplet!(intt(), intt(), intt()));

    let body = parallel!(tfalse(), int(20), int(30), int(40)).with_arg_types(
        tuplet!(intt(), intt(), intt()),
        tuplet!(boolt(), intt(), intt(), intt()),
    );

    let expr = get(dowhile(inputs.clone(), body.clone()), 0)
        .with_arg_types(twoint.clone(), base(intt()))
        .add_ctx(noctx());

    let old_loop_ctx = inloop(inputs.add_ctx(noctx()), body.clone());

    let replace_with = parallel!(int(3), int(4))
        .with_arg_types(twoint.clone(), twoint.clone())
        .add_ctx(noctx());

    let old_second_loop_ctx = inloop(
        single(get(arg(), 0))
            .with_arg_types(twoint.clone(), tuplet!(intt()))
            .add_ctx(noctx()),
        parallel!(tfalse(), get(arg(), 0))
            .with_arg_types(tuplet!(intt()), tuplet!(boolt(), intt())),
    );

    // add context manually because inner loop uses old context still
    let expected = get(
        dowhile(
            parallel!(
                inctx(noctx(), int(1)),
                get(inctx(noctx(), replace_with.clone()), 1),
                get(
                    dowhile(
                        single(get(inctx(noctx(), replace_with.clone()), 0)),
                        parallel!(
                            inctx(old_second_loop_ctx.clone(), tfalse()),
                            get(inctx(old_second_loop_ctx, arg()), 0)
                        )
                    ),
                    0
                )
            ),
            parallel!(
                inctx(old_loop_ctx.clone(), tfalse()),
                inctx(old_loop_ctx.clone(), int(20)),
                inctx(old_loop_ctx.clone(), int(30)),
                inctx(old_loop_ctx.clone(), int(40))
            ),
        ),
        0,
    )
    .with_arg_types(twoint.clone(), base(intt()));

    let build = format!(
        "
(let substituted (Subst (NoContext)
                        {replace_with}
                        {expr}))"
    );
    let check = format!(
        "
(let expected {expected})
(check (= substituted expected))"
    );

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
        Value::Const(Constant::Int(20)),
        vec![],
    )
}

#[test]
fn test_subst_makes_new_context() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let expr = add(
        inctx(noctx(), int_ty(1, base(intt()))),
        inctx(noctx(), iarg()),
    );
    let replace_with = int_ty(2, base(intt()));
    let expected = add(inctx(noctx(), int(1)), inctx(noctx(), int(2)))
        .with_arg_types(base(intt()), base(intt()));
    let build = format!(
        "
(let substituted (Subst (NoContext)
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
    let expr = add(iarg(), iarg()).add_ctx(noctx());
    let tupletype = tuplet!(intt(), intt());
    let replace_with = get(arg(), 0)
        .with_arg_types(tupletype.clone(), base(intt()))
        .add_ctx(noctx());

    let expected = add(get(arg(), 0), get(arg(), 0))
        .with_arg_types(tupletype.clone(), base(intt()))
        .add_ctx(noctx());
    let build = format!(
        "
(let substituted (Subst (NoContext)
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
fn test_subst_identity() -> crate::Result {
    use crate::ast::*;

    let expression = function(
        "main",
        base(intt()),
        base(intt()),
        tif(ttrue(), int(5), int(1), int(2)),
    )
    .func_with_arg_types()
    .func_add_ctx();

    let replace_with = inctx(noctx(), int(5).with_arg_types(base(intt()), base(intt())));

    let build = format!(
        "
(let substituted (Subst (NoContext)
                        {replace_with}
                        {expression}))"
    );
    let check = format!("(check (= substituted {expression}))");
    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![expression.func_to_program()],
        intv(5),
        intv(1),
        vec![],
    )
}

#[test]
fn test_subst_preserves_context() -> crate::Result {
    use crate::ast::*;

    let outer_if = add(int(5), arg());
    let expression = function("main", base(intt()), base(intt()), outer_if)
        .func_with_arg_types()
        .func_add_ctx();

    let replace_with = int(5).with_arg_types(base(intt()), base(intt()));

    let expected = function("main", base(intt()), base(intt()), add(int(5), int(5)))
        .func_with_arg_types()
        .func_add_ctx();

    let build = format!(
        "
(let substituted (Subst (NoContext)
                        {replace_with}
                        {expression}))"
    );
    let check = format!("(check (= substituted {expected}))");
    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![expression.func_to_program()],
        intv(5),
        intv(10),
        vec![],
    )
}
