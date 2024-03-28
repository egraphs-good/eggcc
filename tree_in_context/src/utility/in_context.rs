/* TODO fix in context with DAG semantics

#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_in_context_two_lets() -> crate::Result {
    use crate::ast::*;
    let expr = function(
        "main",
        base(intt()),
        base(intt()),
        tlet(int(1), tlet(add(arg(), arg()), mul(arg(), int(2)))),
    )
    .func_with_arg_types();
    let int1 = in_context(infunc("main"), int_ty(1, base(intt())));
    let arg1 = in_context(inlet(int1.clone()), arg_ty(base(intt())));
    let addarg1 = add(arg1.clone(), arg1.clone());
    let int2 = in_context(inlet(addarg1.clone()), int(2));
    let arg2 = in_context(inlet(addarg1.clone()), arg());
    let expr2 = function(
        "main",
        base(intt()),
        base(intt()),
        tlet(
            int1.clone(),
            tlet(
                add(arg1.clone(), arg1.clone()),
                mul(arg2.clone(), int2.clone()),
            ),
        ),
    )
    .func_with_arg_types();

    egglog_test(
        &format!("(AddFuncContext {expr})"),
        &format!(
            "
(check (Let (Const (Int 1) (Base (IntT))) whatever))
(check (DoAddContext something (InFunc \"main\") (Full) (Let (Const (Int 1) (Base (IntT))) bsdfody)))
(check (DoAddContext somethingelse (InFunc \"main\") (Full) (Const (Int 1) (Base (IntT)))))
(check (= {expr} {expr2}))"
        ),
        vec![
            expr.to_program(emptyt(), base(intt())),
            expr2.to_program(emptyt(), base(intt())),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(4)),
        vec![],
    )
}

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
