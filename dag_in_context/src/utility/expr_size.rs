#[cfg(test)]
use crate::{egglog_test, interpreter::Value};

#[test]
fn test_expr_size() -> crate::Result {
    use crate::ast::*;
    let pureloop = dowhile(
        single(int(1)),
        parallel!(
            less_than(getat(0), int(3)),
            get(switch!(int(2), arg(); parallel!(int(4), int(5))), 0)
        ),
    )
    .with_arg_types(emptyt(), tuplet!(intt()));
    let build: String = format!("(let loop {})", pureloop);

    let check = "(check (= 10 (Expr-size loop)))";
    egglog_test(
        build.as_str(),
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
