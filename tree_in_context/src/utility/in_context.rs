#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_in_context_two_loops() -> crate::Result {
    use crate::ast::*;

    let loop1_body = parallel!(
        tfalse(),
        get(dowhile(
            single(add(getat(0), getat(0))),
            parallel!(tfalse(), mul(getat(0), int(2)))
        ), 0)
    );

    let expr = function(
        "main",
        base(intt()),
        tuplet!(intt()),
        dowhile(
            single(int(1)),
            loop1_body
        ),
    )
    .func_with_arg_types();

    let with_context = expr.clone().func_add_context();

    egglog_test(
        &format!("(AddFuncContext {expr})"),
        &format!(
            "
(check (= {expr} {with_context}))"
        ),
        vec![
            expr.func_to_program(),
            with_context.func_to_program(),
        ],
        Value::Tuple(vec![]),
        tuplev!(val_int(4)),
        vec![],
    )
}

/*
#[test]
fn test_if_contexts() -> crate::Result {
    use crate::ast::*;
    let expr = function(
        "main",
        base(intt()),
        base(intt()),
        tif(ttrue(), int(1), int(2)),
    )
    .func_with_arg_types();
    let pred = in_context(infunc("main"), ttrue_ty(base(intt())));
    let expr2 = function(
        "main",
        base(intt()),
        base(intt()),
        tif(
            pred.clone(),
            in_context(inif(true, pred.clone()), int(1)),
            in_context(inif(false, pred.clone()), int(2)),
        ),
    )
    .func_with_arg_types();
    egglog_test(
        &format!("(AddFuncContext {expr})"),
        &format!("(check (= {expr} {expr2}))"),
        vec![
            expr.to_program(emptyt(), base(intt())),
            expr2.to_program(emptyt(), base(intt())),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(1)),
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
(AddContext (InFunc \"main\") (Full) {expr})
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
fn test_harder_context_cycle() -> crate::Result {
    use crate::ast::*;
    // loop runs one iteration and returns 3
    let myloop = dowhile(arg(), parallel!(tfalse(), int(3)))
        .with_arg_types(tuplet!(intt()), tuplet!(intt()));
    let expr =
        function("main", tuplet!(intt()), tuplet!(intt()), myloop.clone()).func_with_arg_types();
    let int3func =
        function("main", tuplet!(intt()), tuplet!(intt()), single(int(3))).func_with_arg_types();

    let fargincontext = in_context(
        infunc("main"),
        arg().with_arg_types(tuplet!(intt()), tuplet!(intt())),
    );
    let inner_in_context = inloop(
        fargincontext.clone(),
        parallel!(tfalse(), int(3)).with_arg_types(tuplet!(intt()), tuplet!(boolt(), intt())),
    );
    let expr2 = function(
        "main",
        tuplet!(intt()),
        tuplet!(intt()),
        dowhile(
            fargincontext.clone(),
            parallel!(
                in_context(inner_in_context.clone(), tfalse()), // false gets the loop context
                in_context(infunc("main"), int(3)) // 3 is equal to the loop, which is equal to 3 in the outer context
            ),
        ),
    )
    .func_with_arg_types();

    egglog_test(
        &format!(
            "
{expr}
(AddFuncContext {expr})
    ",
        ),
        &format!(
            "
(check (= {expr} {expr2}))
(check (= {expr} {int3func}))"
        ),
        vec![
            expr.to_program(tuplet!(intt()), tuplet!(intt())),
            expr2.to_program(tuplet!(intt()), tuplet!(intt())),
        ],
        Value::Tuple(vec![Value::Const(Constant::Int(3))]),
        Value::Tuple(vec![Value::Const(Constant::Int(3))]),
        vec![],
    )
}

#[test]
fn simple_context() -> crate::Result {
    use crate::ast::*;
    use crate::egglog_test;
    use crate::{interpreter::Value, schema::Constant};
    let expr = function("main", base(intt()), base(intt()), int(2)).func_with_arg_types();
    let expected = function(
        "main",
        base(intt()),
        base(intt()),
        in_context(infunc("main"), int(2)),
    )
    .func_with_arg_types();
    egglog_test(
        &format!("(AddFuncContext {expr})"),
        &format!(
            "
(check (= {expr} {expected}))",
        ),
        vec![
            expr.to_program(emptyt(), base(intt())),
            expected.to_program(emptyt(), base(intt())),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
        vec![],
    )
}
*/
