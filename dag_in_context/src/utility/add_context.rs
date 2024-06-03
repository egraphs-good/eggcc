#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_in_context_tuple() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;
    use crate::{interpreter::Value, schema::Constant};

    let tuple =
        parallel!(int(3), int(4)).with_arg_types(tuplet!(intt(), intt()), tuplet!(intt(), intt()));

    let ctx = Assumption::dummy();
    let build = format!(
        "
(let substituted (AddContext {ctx} {tuple}))"
    );
    let check = format!(
        "
(check (= substituted {tuple}))
        "
    );

    crate::egglog_test_and_print_program(
        &build.to_string(),
        &check.to_string(),
        vec![],
        Value::Tuple(vec![
            Value::Const(Constant::Int(10)),
            Value::Const(Constant::Int(10)),
        ]),
        Value::Const(Constant::Int(20)),
        vec![],
    )
}

#[test]
fn test_in_context_two_loops() -> crate::Result {
    use crate::ast::*;
    use crate::egglog_test_and_print_program;
    use crate::schema::Assumption;

    let loop_body = parallel!(
        tfalse(),
        get(
            dowhile(
                single(add(getat(0), getat(0))),
                parallel!(tfalse(), mul(getat(0), int(2)))
            ),
            0
        )
    );

    let ctx = Assumption::dummy();
    // expression with nested loop, we'll add context to inner loop
    let expr = dowhile(single(int(1)), loop_body.clone()).with_arg_types(emptyt(), tuplet!(intt()));

    egglog_test_and_print_program(
        &format!(
            "
(let egglog-version (AddContext {ctx} {expr}))"
        ),
        &format!(
            "
(check (= egglog-version {expr}))"
        ),
        vec![expr.to_program(emptyt(), tuplet!(intt()))],
        Value::Tuple(vec![]),
        tuplev!(intv(4)),
        vec![],
    )
}

#[test]
fn test_simple_context_cycle() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;
    let inputs = single(arg()).with_arg_types(base(intt()), tuplet!(intt()));
    let outputs =
        parallel!(tfalse(), int(3)).with_arg_types(tuplet!(intt()), tuplet!(boolt(), intt()));
    let expr = dowhile(arg(), parallel!(tfalse(), int(3)))
        .with_arg_types(tuplet!(intt()), tuplet!(intt()));
    let inner = single(int(3)).with_arg_types(tuplet!(intt()), tuplet!(intt()));
    let ctx = Assumption::dummy();

    egglog_test(
        &format!(
            "
(union {expr} {inner})
(Subst {ctx} {inputs} {outputs})
(AddContext {ctx} {expr})
",
        ),
        &format!(
            "
(check (= {expr} {inner}))
"
        ),
        vec![expr.to_program(tuplet!(intt()), tuplet!(intt()))],
        tuplev!(intv(3)),
        Value::Tuple(vec![Value::Const(Constant::Int(3))]),
        vec![],
    )
}

#[test]
fn simple_context() -> crate::Result {
    use crate::ast::*;
    use crate::egglog_test;
    use crate::{interpreter::Value, schema::Constant};
    let expr = int(2).with_arg_types(emptyt(), base(intt()));
    let context_to_add = inif(
        true,
        ttrue().with_arg_types(emptyt(), base(boolt())),
        parallel!().with_arg_types(emptyt(), emptyt()),
    );
    let (expected, cache) = expr.add_ctx(context_to_add.clone());

    egglog_test(
        &format!("(let egglog (AddContext {context_to_add} {expr}))"),
        &format!(
            "{expected}\n{}\n(check (= egglog {expected}))",
            cache.get_unions(),
        ),
        vec![expr.to_program(emptyt(), base(intt()))],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
        vec![],
    )
}
