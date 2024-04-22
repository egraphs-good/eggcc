#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

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

    // expression with nexted loop, we'll add context to inner loop
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
    let expr = dowhile(single(arg()), parallel!(tfalse(), int(3)))
        .with_arg_types(base(intt()), tuplet!(intt()));
    let inner = single(int(3)).with_arg_types(tuplet!(intt()), tuplet!(intt()));

    egglog_test(
        &format!(
            "
(union {expr} {inner})
(AddContext (NoContext) (Full) {expr})
",
        ),
        &format!(
            "
(check (= {expr} {inner}))
"
        ),
        vec![expr.to_program(base(intt()), tuplet!(intt()))],
        Value::Const(Constant::Int(3)),
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
