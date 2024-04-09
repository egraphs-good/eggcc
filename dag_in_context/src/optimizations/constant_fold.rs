#[cfg(test)]
use crate::egglog_test;

#[test]
fn test_add_constant_fold() -> crate::Result {
    use crate::ast::*;
    let expr = add(int(1), int(2)).with_arg_types(emptyt(), base(intt()));
    let expr2 = int_ty(3, emptyt());

    egglog_test(
        &format!("{expr}"),
        &format!("(check (= {expr} {expr2}))"),
        vec![
            expr.to_program(emptyt(), base(intt())),
            expr2.to_program(emptyt(), base(intt())),
        ],
        val_empty(),
        intv(3),
        vec![],
    )
}
