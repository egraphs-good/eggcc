#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_assume_two_lets() -> crate::Result {
    use crate::ast::*;
    let expr = tlet(int(1), tlet(add(arg(), arg()), mul(arg(), int(2))));
    let arg1 = assume(inlet(int(1)), arg());
    let addarg1 = add(arg1.clone(), arg1.clone());
    let arg2 = assume(inlet(addarg1), arg());
    let expr2 = tlet(int(1), tlet(add(arg1.clone(), arg1), mul(arg2, int(2))));

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
