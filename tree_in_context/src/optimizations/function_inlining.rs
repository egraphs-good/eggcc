#[cfg(test)]
use crate::{egglog_test, interpreter::Value, schema::Constant};

#[test]
fn test_function_inlining_single_function() -> crate::Result {
    use crate::ast::*;
    // main := inc 5
    let main_body = call("inc", int_ty(5, emptyt()));
    let main = function("main", emptyt(), base(intt()), main_body.clone());

    // inc n := n + 1
    let inc_body = add(arg(), int(1)).with_arg_types(base(intt()), base(intt()));
    let inc = function("inc", base(intt()), base(intt()), inc_body.clone());

    let prog = program!(main.clone(), inc.clone());
    let result = tlet(int_ty(5, emptyt()), inc_body);

    // check that (inc 5) = 6
    egglog_test(
        &format!("{prog}"),
        &format!("(check (= {main_body} {result}))"),
        vec![prog],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(6)),
        vec![],
    )
}

#[test]
fn test_function_inlining_recursive_function() -> crate::Result {
    use crate::ast::*;
    // main := fact 5
    let main_body = call("fact", int(5));
    let main = function("main", emptyt(), base(intt()), main_body.clone());

    // fact n := if n > 1 then n * fact n-1 else 1    (don't bother with handling n < 1)
    let n = arg().with_arg_types(base(intt()), base(intt()));
    let fact_body = tif(
        greater_than(n.clone(), int(1)),
        mul(n.clone(), call("fact", sub(n.clone(), int(1)))),
        int(1),
    );
    let fact =
        function("fact", base(intt()), base(intt()), fact_body.clone());

    let prog = program!(main.clone(), fact.clone());

    // check that (fact 5) = (let 5 in (if arg > 1 then (arg * fact arg-1) else 1))
    let result = tlet(
        int(5),
        tif(
            greater_than(arg().with_arg_types(base(intt()), base(intt())), int(1)),
            mul(
                arg().with_arg_types(base(intt()), base(intt())),
                call(
                    "fact",
                    sub(arg().with_arg_types(base(intt()), base(intt())), int(1)),
                ),
            ),
            int(1),
        ),
    );

    // note that this program will not get extracted under our current cost model,
    // since it doesn't reduce the number of calls
    egglog_test(
        &format!("{prog}"),
        &format!("(check (= {main_body} {result}))"),
        vec![prog],
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(120)),
        vec![],
    )
}
