#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_add_constant_fold() -> crate::Result {
    use crate::ast::*;
    let expr = add(int(1), int(2));
    let expr2 = int(3);

    egglog_test(
        &format!("{expr}"),
        &format!("(check (= {expr} {expr2}))"),
        vec![
            expr.to_program(emptyt(), intt()),
            expr2.to_program(emptyt(), intt()),
        ],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(3)),
    )
}
