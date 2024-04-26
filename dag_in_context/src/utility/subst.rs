#[test]
fn test_subst_twice() -> crate::Result {
    use crate::ast::*;
    let arg_plus_one = add(arg(), int(1)).add_arg_type(base(intt()));
    let original = arg().add_arg_type(base(intt()));
    let expected = add(add(arg(), int(1)), int(1)).add_arg_type(base(intt()));

    let build = format!(
        "
(let substituted (Subst (NoContext) {arg_plus_one} (Subst (NoContext) {arg_plus_one} {original})))
"
    );
    let check = format!("(check (= substituted {expected}))");
    crate::egglog_test(
        &build,
        &check,
        vec![expected.to_program(base(intt()), base(intt()))],
        intv(1),
        intv(3),
        vec![],
    )
}

#[test]
fn test_subst_ten_times() -> crate::Result {
    use crate::ast::*;
    let arg_plus_one = add(arg(), int(1)).add_arg_type(base(intt()));
    let original = arg().add_arg_type(base(intt()));
    let mut expected = arg();
    for _ in 0..10 {
        expected = add(expected, int(1));
    }
    expected = expected.add_arg_type(base(intt()));

    let build = format!(
        "
(let substituted
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one}
    (Subst (NoContext) {arg_plus_one} {original})))))))))))
"
    );
    let check = format!("(check (= substituted {expected}))");
    crate::egglog_test(
        &build,
        &check,
        vec![expected.to_program(base(intt()), base(intt()))],
        intv(1),
        intv(11),
        vec![],
    )
}

#[test]
fn test_subst_cycle() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let twoint = tuplet!(intt(), intt());

    let expr = parallel!(getat(0), int(0)) // saturates if both getat(0) or both int(0)!
        .with_arg_types(
            tuplet!(intt(), intt()), // tuplet!(intt()) saturates!
            tuplet!(intt(), intt()),
        )
        .initialize_ctx();

    let replace_with = parallel!(int(3), int(4))
        .with_arg_types(twoint.clone(), twoint.clone())
        .initialize_ctx();

    let build = format!(
        "
(let substituted (Subst (NoContext)
                        {replace_with}
                        {expr}))"
    );

    crate::egglog_test_and_print_program(
        &build.to_string(),
        "",
        vec![],
        Value::Const(Constant::Int(10)),
        Value::Const(Constant::Int(10)),
        vec![],
    )
}

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
        .initialize_ctx();

    let replace_with = parallel!(int(3), int(4))
        .with_arg_types(twoint.clone(), twoint.clone())
        .initialize_ctx();

    // add context manually because inner loop uses old context still
    let expected = get(
        dowhile(
            parallel!(
                int(1),
                get(replace_with.clone(), 1),
                get(
                    dowhile(
                        single(get(replace_with.clone(), 0)),
                        parallel!(tfalse(), get(arg(), 0))
                    ),
                    0
                )
            ),
            parallel!(tfalse(), int(20), int(30), int(40)).initialize_ctx(),
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

    crate::egglog_test_and_print_program(
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
    let expr = add(int_ty(1, base(intt())), iarg());
    let replace_with = int_ty(2, base(intt())).initialize_ctx();
    let expected = add(int(1), int(2)).with_arg_types(base(intt()), base(intt()));
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

    let replace_with = int(5).with_arg_types(base(intt()), base(intt()));

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
fn test_subst_if() -> crate::Result {
    use crate::ast::*;

    let outer_if = add(int(5), arg());
    let expression = function("main", base(intt()), base(intt()), outer_if)
        .func_with_arg_types()
        .func_add_ctx();

    let replace_with = int(5)
        .with_arg_types(base(intt()), base(intt()))
        .initialize_ctx();

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
