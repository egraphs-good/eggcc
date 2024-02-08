#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_assume_in_func() -> crate::Result {
    use crate::ast::*;
    let expr = function("main", intt(), intt(), int(2));
    let expected = function("main", intt(), intt(), assume(infunc("main"), int(2)));
    egglog_test(
        &format!("{expr}"),
        &format!("(check (= {expr} {expected}))"),
        vec![
            expr.to_program(emptyt(), intt()),
            expected.to_program(emptyt(), intt()),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
    )
}

#[test]
fn test_assume_two_lets() -> crate::Result {
    use crate::ast::*;
    let expr = function(
        "main",
        intt(),
        intt(),
        tlet(int(1), tlet(add(arg(), arg()), mul(arg(), int(2)))),
    );
    let int1 = assume(infunc("main"), int(1));
    let arg1 = assume(inlet(int1.clone()), arg());
    let addarg1 = add(arg1.clone(), arg1.clone());
    let int2 = assume(inlet(addarg1.clone()), int(2));
    let arg2 = assume(inlet(addarg1.clone()), arg());
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
    )
}

#[test]
fn test_switch_contexts() -> crate::Result {
    use crate::ast::*;
    let expr = function("main", intt(), intt(), tif(ttrue(), int(1), int(2)));
    let pred = assume(infunc("main"), ttrue());
    let expr2 = function(
        "main",
        intt(),
        intt(),
        tif(
            pred.clone(),
            assume(inif(true, pred.clone()), int(1)),
            assume(inif(false, pred.clone()), int(2)),
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
    )
}
