#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_in_context_in_func() -> crate::Result {
    use crate::ast::*;
    let expr = function("main", intt(), intt(), int(2));
    let expected = function("main", intt(), intt(), in_context(infunc("main"), int(2)));
    egglog_test(
        &format!("{expr}"),
        &format!("(check (= {expr} {expected}))"),
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
        &format!("{expr}"),
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
fn test_switch_contexts() -> crate::Result {
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
        &format!("{expr}"),
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
fn test_dowhile_cycle_in_context() -> crate::Result {
    use crate::ast::*;
    // loop runs one iteration and returns 3
    let myloop = dowhile(single(int(2)), parallel!(tfalse(), int(3)));
    let expr = function("main", intt(), tuplet!(intt()), myloop);

    let int2 = single(in_context(infunc("main"), int(2)));
    let inner_in_context = inloop(int2.clone(), parallel!(tfalse(), int(3)));
    let expr2 = function(
        "main",
        intt(),
        tuplet!(intt()),
        dowhile(
            int2.clone(),
            parallel!(
                in_context(inner_in_context.clone(), tfalse()),
                in_context(inner_in_context.clone(), int(3)),
            ),
        ),
    );

    egglog_test(
        &format!(
            "{expr}
(union {} {expr})",
            single(int(3))
        ),
        &format!("(check (= {expr} {expr2}))"),
        vec![
            expr.to_program(emptyt(), tuplet!(intt())),
            expr2.to_program(emptyt(), tuplet!(intt())),
        ],
        Value::Tuple(vec![]),
        Value::Tuple(vec![Value::Const(Constant::Int(3))]),
        vec![],
    )
}
