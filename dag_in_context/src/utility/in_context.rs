#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_in_context_tuple() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};

    let tuple = parallel!(int(3), int(4))
        .with_arg_types(tuplet!(intt(), intt()), tuplet!(intt(), intt()))
        .initialize_ctx();

    let build = format!(
        "
(let substituted (AddContext (NoContext) (Region) {tuple}))"
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

    // expression with nested loop, we'll add context to inner loop
    let expr = function(
        "main",
        base(intt()),
        tuplet!(intt()),
        dowhile(single(int(1)), loop_body.clone()),
    )
    .func_with_arg_types()
    .initialize_ctx();
    let with_ctx = expr.replace_ctx(noctx());

    egglog_test(
        &format!(
            "
(let egglog-version (AddFuncContext {expr}))"
        ),
        &format!(
            "
(let rust-version {with_ctx})
(check (= egglog-version rust-version))"
        ),
        vec![expr.func_to_program()],
        Value::Tuple(vec![]),
        tuplev!(intv(4)),
        vec![],
    )
}

#[test]
fn test_simple_context_cycle() -> crate::Result {
    use crate::ast::*;
    let inputs = single(arg())
        .with_arg_types(base(intt()), tuplet!(intt()))
        .initialize_ctx();
    let outputs = parallel!(tfalse(), int(3))
        .with_arg_types(tuplet!(intt()), tuplet!(boolt(), intt()))
        .initialize_ctx();
    let expr = dowhile(arg(), parallel!(tfalse(), int(3)))
        .with_arg_types(tuplet!(intt()), tuplet!(intt()))
        .initialize_ctx();
    let inner = single(int(3))
        .with_arg_types(tuplet!(intt()), tuplet!(intt()))
        .initialize_ctx();

    egglog_test(
        &format!(
            "
(union {expr} {inner})
(Subst (NoContext) {inputs} {outputs})
(AddContext (NoContext) (Full) {expr})
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
    let expr =
        function("main", base(intt()), base(intt()), inctx(noctx(), int(2))).func_with_arg_types();

    egglog_test(
        &format!("(AddFuncContext {expr})"),
        &format!(
            "
(check (= (AddFuncContext {expr}) {expr}))",
        ),
        vec![expr.to_program(emptyt(), base(intt()))],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
        vec![],
    )
}
