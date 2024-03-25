#[cfg(test)]
use crate::egglog_test;

#[test]
fn test_simple_prop_eq() -> crate::Result {
    use crate::ast::*;
    let prog = tlet(parallel!(arg(), arg()), add(getat(0), getat(1)))
        .with_arg_types(base(intt()), base(intt()));
    let expected = tlet(parallel!(arg(), arg()), mul(getat(0), int(2)))
        .with_arg_types(base(intt()), base(intt()));
    egglog_test(
        &format!("(let expr {})", prog),
        &format!("(check (= expr {}))", expected),
        vec![
            prog.to_program(base(intt()), base(intt())),
            expected.to_program(base(intt()), base(intt())),
        ],
        val_int(2),
        val_int(4),
        vec![],
    )
}
