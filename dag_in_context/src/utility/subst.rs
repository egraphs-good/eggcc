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
    let replacement = inctx(noctx(), replace_with.clone());
    let expected = get(
        dowhile(
            parallel!(
                inctx(noctx(), int(1)),
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
(let substituted (Subst inf-fuel (NoContext)
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
        inctx(noctx(), int_ty(1, base(intt()))),
        inctx(noctx(), iarg()),
    );
    let replace_with = int_ty(2, base(intt()));
    let expected = add(inctx(noctx(), int(1)), inctx(noctx(), int(2)))
        .with_arg_types(base(intt()), base(intt()));
    let build = format!(
        "
(let substituted (Subst inf-fuel (NoContext)
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

    let expected = add(inctx(noctx(), get(arg(), 0)), inctx(noctx(), get(arg(), 0)))
        .with_arg_types(tupletype.clone(), base(intt()));
    let build = format!(
        "
(let substituted (Subst inf-fuel (NoContext)
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
        tif(ttrue(), arg(), int(1), int(2)),
    )
    .func_with_arg_types();

    let with_context = function(
        "main",
        base(intt()),
        base(intt()),
        tif(
            inctx(noctx(), ttrue()),
            inctx(noctx(), int(5)),
            int(1),
            int(2),
        ),
    )
    .func_with_arg_types();

    let replace_with = int(5).with_arg_types(base(intt()), base(intt()));

    let build = format!(
        "
(let substituted (Subst inf-fuel (NoContext)
                        {replace_with}
                        {expression}))"
    );
    let check = format!("(check (= substituted {with_context}))");
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
(let substituted (Subst inf-fuel (NoContext)
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

// replace with, original expression, and expected for the two fuel tests
// to show that not enough fuel causes substitution to not propagate fully,
// but enough fuel causes substitution to replace the desired arg.
// For these tests, only the build/check are different but the setup is the same.
#[cfg(test)]
fn fuel_test_replacewith_expression_expected() -> (
    std::rc::Rc<crate::schema::Expr>,
    std::rc::Rc<crate::schema::Expr>,
    std::rc::Rc<crate::schema::Expr>,
) {
    use crate::ast::*;

    let expression = function(
        "main",
        base(intt()),
        base(intt()),
        get(
            dowhile(
                single(arg()),
                parallel!(less_than(get(arg(), 0), int(0)), int(3)),
            ),
            0,
        ),
    )
    .func_with_arg_types();

    let expected = function(
        "main",
        base(intt()),
        base(intt()),
        get(
            dowhile(
                single(inctx(noctx(), int(4))),
                parallel!(less_than(get(arg(), 0), int(0)), int(3)),
            ),
            0,
        ),
    )
    .func_with_arg_types();

    let replace_with = int(4).with_arg_types(base(intt()), base(intt()));

    (replace_with, expression, expected)
}

#[test]
fn test_subst_with_enough_fuel_goes() -> crate::Result {
    use crate::ast::*;

    let (replace_with, expression, expected) = fuel_test_replacewith_expression_expected();

    let build =
        format!("(let substituted (Subst 3 (NoContext) {replace_with} {expression})) (let expected {expected})");
    let check = format!("(check (= substituted {expected}))");

    crate::egglog_test(
        &build,
        &check,
        vec![expression.func_to_program()],
        intv(0),
        intv(3),
        vec![],
    )
}

#[test]
fn test_subst_with_little_fuel_stops() -> crate::Result {
    use crate::ast::*;

    let (replace_with, expression, expected) = fuel_test_replacewith_expression_expected();

    let build =
        format!("(let substituted (Subst 2 (NoContext) {replace_with} {expression})) (let expected {expected})");
    let check = format!("(fail (check (= substituted {expected})))");

    crate::egglog_test(
        &build,
        &check,
        vec![expression.func_to_program()],
        intv(0),
        intv(3),
        vec![],
    )
}
