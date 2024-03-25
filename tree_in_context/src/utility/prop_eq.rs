#[cfg(test)]
use crate::egglog_test;

#[test]
fn test_simple_prop_eq() -> crate::Result {
    use crate::ast::*;
    let prog = function(
        "main",
        base(intt()),
        base(intt()),
        tlet(parallel!(arg(), arg()), add(getat(0), getat(1)))
            .with_arg_types(base(intt()), base(intt())),
    )
    .func_with_arg_types();
    let expected = function(
        "main",
        base(intt()),
        base(intt()),
        tlet(parallel!(arg(), arg()), mul(getat(0), int(2))),
    )
    .func_with_arg_types();
    egglog_test(
        &format!(
            "
(let expr (AddFuncContext {}))
(let expr2 (AddFuncContext {}))",
            prog, expected
        ),
        "(check (= expr expr2))",
        vec![
            prog.to_program(base(intt()), base(intt())),
            expected.to_program(base(intt()), base(intt())),
        ],
        val_int(2),
        val_int(4),
        vec![],
    )
}
