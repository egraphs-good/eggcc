#[test]
fn test_subst_twice() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;
    let arg_plus_one = add(arg(), int(1)).add_arg_type(base(intt()));
    let original = arg().add_arg_type(base(intt()));
    let expected = add(add(arg(), int(1)), int(1)).add_arg_type(base(intt()));

    let ctx = Assumption::dummy();
    let build = format!(
        "
(let substituted (Subst {ctx} {arg_plus_one} (Subst {ctx} {arg_plus_one} {original})))
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
    use crate::schema::Assumption;
    let arg_plus_one = add(arg(), int(1)).add_arg_type(base(intt()));
    let original = arg().add_arg_type(base(intt()));
    let mut expected = arg();
    for _ in 0..10 {
        expected = add(expected, int(1));
    }
    expected = expected.add_arg_type(base(intt()));
    let ctx = Assumption::dummy();

    let build = format!(
        "
(let substituted
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one}
    (Subst {ctx} {arg_plus_one} {original})))))))))))
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
    use crate::schema::Assumption;
    use crate::{interpreter::Value, schema::Constant};
    let twoint = tuplet!(intt(), intt());

    let expr = parallel!(getat(0), int(0)) // saturates if both getat(0) or both int(0)!
        .with_arg_types(
            tuplet!(intt(), intt()), // tuplet!(intt()) saturates!
            tuplet!(intt(), intt()),
        );

    let replace_with = parallel!(int(3), int(4)).with_arg_types(twoint.clone(), twoint.clone());
    let ctx = Assumption::dummy();

    let build = format!(
        "
(let substituted (Subst {ctx}
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
    use crate::schema::Assumption;
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

    let expr =
        get(dowhile(inputs.clone(), body.clone()), 0).with_arg_types(twoint.clone(), base(intt()));

    let replace_with = parallel!(int(3), int(4)).with_arg_types(twoint.clone(), twoint.clone());

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
            parallel!(tfalse(), int(20), int(30), int(40)),
        ),
        0,
    )
    .with_arg_types(twoint.clone(), base(intt()));
    let ctx = Assumption::dummy();

    let build = format!(
        "
(let substituted (Subst {ctx}
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
fn test_subst_if() -> crate::Result {
    use crate::ast::*;
    use crate::interpreter::Value;
    use crate::schema::Assumption;
    let expr = tif(
        getat(0),
        add(getat(1), int(1)),
        add(arg(), int(1)),
        sub(arg(), int(1)),
    )
    .add_arg_type(tuplet!(boolt(), intt()));

    let replace_with = parallel!(ttrue(), int(5)).add_arg_type(emptyt());

    let expected = tif(
        ttrue(),
        add(int(5), int(1)),
        add(arg(), int(1)),
        sub(arg(), int(1)),
    )
    .add_arg_type(emptyt())
    .add_symbolic_ctx();
    let ctx = Assumption::dummy();

    let build = format!(
        "
(let substituted (Subst {ctx}
                        {replace_with}
                        {expr}))"
    );
    let check = format!("(check (= substituted {}))", expected.value);

    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![expr.to_program(tuplet!(boolt(), intt()), base(intt()))],
        Value::Tuple(vec![truev(), intv(5)]),
        intv(7),
        vec![],
    )
}

#[test]
fn test_subst_makes_new_context() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;
    use crate::{interpreter::Value, schema::Constant};
    let expr = add(int_ty(1, base(intt())), iarg());
    let replace_with = int_ty(2, base(intt()));
    let expected = add(int(1), int(2)).with_arg_types(base(intt()), base(intt()));
    let ctx = Assumption::dummy();
    let build = format!(
        "
(let substituted (Subst {ctx}
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
    use crate::schema::Assumption;
    use crate::{interpreter::Value, schema::Constant};
    let expr = add(iarg(), iarg()).add_ctx(Assumption::dummy());
    let tupletype = tuplet!(intt(), intt());
    let replace_with = get(arg(), 0)
        .with_arg_types(tupletype.clone(), base(intt()))
        .add_ctx(Assumption::dummy());

    let expected = add(get(arg(), 0), get(arg(), 0))
        .with_arg_types(tupletype.clone(), base(intt()))
        .add_ctx(Assumption::dummy());
    let ctx = Assumption::dummy();
    let build = format!(
        "{}\n{}\n{}\n{}\n(let substituted (Subst {ctx} {} {}))",
        replace_with.value,
        replace_with.get_unions(),
        expr.value,
        expr.get_unions(),
        replace_with.value,
        expr.value
    );
    let check = format!(
        "{}\n{}\n(check (= substituted {}))",
        expected.value,
        expected.get_unions(),
        expected.value
    );
    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![expr.value.to_program(base(intt()), base(intt()))],
        Value::Const(Constant::Int(2)),
        Value::Const(Constant::Int(4)),
        vec![],
    )
}

#[test]
fn test_subst_identity() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let expression = function(
        "main",
        base(intt()),
        base(intt()),
        tif(ttrue(), int(5), int(1), int(2)),
    )
    .func_with_arg_types()
    .func_add_ctx();

    let replace_with = int(5).with_arg_types(base(intt()), base(intt()));
    let ctx = Assumption::InFunc("main".to_string());

    let build = format!(
        "{}\n{}\n(let substituted (Subst {ctx} {replace_with} {}))",
        expression.value,
        expression.get_unions(),
        expression.value,
    );
    let check = format!("(check (= substituted {}))", expression.value);
    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![expression.value.func_to_program()],
        intv(5),
        intv(1),
        vec![],
    )
}

#[test]
fn test_subst_add() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let outer_if = add(int(5), arg());
    let expression = function("main", base(intt()), base(intt()), outer_if)
        .func_with_arg_types()
        .func_add_ctx();

    let replace_with = int(5).with_arg_types(base(intt()), base(intt()));

    let expected = function("main", base(intt()), base(intt()), add(int(5), int(5)))
        .func_with_arg_types()
        .func_add_ctx();
    let ctx = Assumption::InFunc("main".to_string());

    let build = format!(
        "{}\n{}\n(let substituted (Subst {ctx} {replace_with} {}))",
        expression.value,
        expression.get_unions(),
        expression.value,
    );
    let check = format!(
        "{}\n{}\n(check (= substituted {}))",
        expected.value,
        expected.get_unions(),
        expected.value,
    );
    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![expression.value.func_to_program()],
        intv(5),
        intv(10),
        vec![],
    )
}
