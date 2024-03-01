#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_in_context_two_lets() -> crate::Result {
    use crate::ast::*;
    let expr = function(
        "main",
        intt(),
        intt(),
        tlet(
            int(1),
            tlet(add(int_letarg(), int_letarg()), mul(int_letarg(), int(2))),
        ),
    );
    let int1 = in_context(infunc("main"), int(1));
    let arg1 = in_context(inlet(int1.clone()), int_letarg());
    let addarg1 = add(arg1.clone(), arg1.clone());
    let int2 = in_context(inlet(addarg1.clone()), int(2));
    let arg2 = in_context(inlet(addarg1.clone()), int_letarg());
    let expr2 = function(
        "main",
        intt(),
        intt(),
        tlet(
            int1,
            tlet(
                add(arg1.clone(), arg1.clone()),
                mul(arg2.clone(), int2.clone()),
            ),
        ),
    );

    egglog_test(
        &format!("(ExpandFuncContext {expr})"),
        &format!("(check (= {expr} {expr2}))"),
        vec![
            expr.to_program(emptyt(), intt()),
            expr2.to_program(emptyt(), intt()),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(4)),
        vec![],
    )
}

#[test]
fn test_if_contexts() -> crate::Result {
    use crate::ast::*;
    let expr = function("main", intt(), intt(), tif(ttrue(), int(1), int(2)));
    let pred = in_context(infunc("main"), ttrue());
    let expr2 = function(
        "main",
        intt(),
        intt(),
        tif(
            pred.clone(),
            in_context(inif(true, pred.clone()), int(1)),
            in_context(inif(false, pred.clone()), int(2)),
        ),
    );
    egglog_test(
        &format!("(ExpandFuncContext {expr})"),
        &format!("(check (= {expr} {expr2}))"),
        vec![
            expr.to_program(emptyt(), intt()),
            expr2.to_program(emptyt(), intt()),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(1)),
        vec![],
    )
}

#[test]
fn test_simple_subst_cycle() -> crate::Result {
    use crate::ast::*;
    let expr = dowhile(single(funcarg()), parallel!(tfalse(), int(3)))
        .with_arg_types(intt(), tuplet!(intt()));
    let inner = single(int(3));

    egglog_test(
        &format!(
            "
(union {expr} {inner})
(Subst (InFunc \"main\") (FuncScope) (Arg (FuncScope) (Base (IntT))) {expr})",
        ),
        &format!(
            "
(check (= {expr} {inner}))"
        ),
        vec![expr.to_program(intt(), tuplet!(intt()))],
        Value::Const(Constant::Int(3)),
        Value::Tuple(vec![Value::Const(Constant::Int(3))]),
        vec![],
    )
}

#[test]
fn test_dowhile_cycle_in_context() -> crate::Result {
    use crate::ast::*;
    // loop runs one iteration and returns 3
    let myloop = dowhile(funcarg(), parallel!(tfalse(), int(3)))
        .with_arg_types(tuplet!(intt()), tuplet!(intt()));
    let expr =
        function("main", tuplet!(intt()), tuplet!(intt()), myloop.clone()).func_with_arg_types();
    let int3func = function("main", tuplet!(intt()), tuplet!(intt()), single(int(3)));

    let fargincontext = in_context(
        infunc("main"),
        funcarg().with_arg_types(tuplet!(intt()), tuplet!(intt())),
    );
    let inner_in_context = inloop(fargincontext.clone(), parallel!(tfalse(), int(3)));
    let expr2 = function(
        "main",
        tuplet!(intt()),
        tuplet!(intt()),
        dowhile(
            fargincontext.clone(),
            parallel!(
                in_context(inner_in_context.clone(), tfalse()),
                in_context(inner_in_context.clone(), int(3)),
            ),
        ),
    )
    .func_with_arg_types();
    let ituple = tuplet!(intt());

    egglog_test(
        &format!(
            "
{expr}
(ExpandFuncContext {expr})
;(union {expr} (Function \"main\" {ituple} {ituple}
                 ;(Subst (InFunc \"main\") (FuncScope) (Arg (FuncScope) {ituple}) {myloop})))
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
fn simple_identity_subst() -> crate::Result {
    use crate::ast::*;
    use crate::egglog_test;
    use crate::{interpreter::Value, schema::Constant};
    let expr = function("main", intt(), intt(), int(2));
    let expected = function("main", intt(), intt(), in_context(infunc("main"), int(2)));
    egglog_test(
        &format!("(ExpandFuncContext {expr})"),
        &format!(
            "
(check (= {expr} {expected}))",
        ),
        vec![
            expr.to_program(emptyt(), intt()),
            expected.to_program(emptyt(), intt()),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
        vec![],
    )
}

#[test]
fn test_subst_nested() -> crate::Result {
    use crate::ast::*;
    use crate::{interpreter::Value, schema::Constant};
    let twoint = tuplet!(intt(), intt());
    let expr = tlet(
        parallel!(int(1), get(funcarg(), 1), tlet(int(2), get(funcarg(), 0))),
        int(0),
    )
    .with_arg_types(twoint.clone(), intt());
    let replace_with = parallel!(int(3), int(4));
    let replacement = in_context(infunc("main"), replace_with.clone());
    let replacement_2 = get(
        in_context(
            inlet(in_context(infunc("main"), int(2))),
            replace_with.clone(),
        ),
        0,
    );
    let new_inputs = parallel!(
        in_context(infunc("main"), int(1)),
        get(replacement.clone(), 1),
        tlet(in_context(infunc("main"), int(2)), replacement_2)
    );
    let expected = tlet(
        new_inputs.clone(),
        in_context(inlet(new_inputs.clone()), int(0)),
    )
    .with_arg_types(twoint, intt());

    let build = format!(
        "
(let substituted (Subst (InFunc \"main\")
                        (FuncScope)
                        {replace_with}
                        {expr}))"
    );
    let check = format!(
        "
(check (= substituted {expected}))",
    );

    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![
            expr.to_program(tuplet!(intt(), intt()), intt()),
            expected.to_program(tuplet!(intt(), intt()), intt()),
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
        in_context(infunc("otherfunc"), int(1)),
        in_context(infunc("otherfunc"), int_funcarg()),
    );
    let replace_with = int(2);
    let expected = add(
        in_context(infunc("main"), int(1)),
        in_context(infunc("main"), int(2)),
    );
    let build = format!(
        "
(let substituted (Subst (InFunc \"main\")
                        (FuncScope) 
                        {replace_with}
                        {expr}))"
    );
    let check = format!("(check (= substituted {expected}))");

    crate::egglog_test(
        &build.to_string(),
        &check.to_string(),
        vec![
            expr.to_program(intt(), intt()),
            expected.to_program(intt(), intt()),
        ],
        Value::Const(Constant::Int(2)),
        Value::Const(Constant::Int(3)),
        vec![],
    )
}
