// TODO enable this test once saturation issue with peeling is resolved
/*#[test]
fn loop_peel_once() -> crate::Result {
    use crate::ast::*;
    use crate::egglog_test;
    let prog = dowhile(
        parallel!(int(1)),
        parallel!(tfalse(), add(getat(0), int(1))),
    )
    .with_arg_types(base(intt()), tuplet!(intt()));

    let expected = parallel!(int(2)).with_arg_types(base(intt()), tuplet!(intt()));

    egglog_test(
        &format!("{prog}"),
        &format!(
            "
(check (= {prog} {expected}))"
        ),
        vec![
            prog.to_program(base(intt()), tuplet!(intt())),
            expected.to_program(base(intt()), tuplet!(intt())),
        ],
        intv(0),
        tuplev!(intv(2)),
        vec![],
    )
}*/

#[test]
fn loop_unroll_simple() -> crate::Result {
    use crate::ast::*;
    use crate::egglog_test;
    let prog = dowhile(
        parallel!(int(0)),
        parallel!(
            less_than(add(getat(0), int(1)), int(8)),
            add(getat(0), int(1))
        ),
    )
    .add_arg_type(base(intt()));

    let unrolled_add = add(add(add(add(getat(0), int(1)), int(1)), int(1)), int(1));
    let expected = dowhile(
        parallel!(int(0)),
        parallel!(less_than(unrolled_add.clone(), int(8)), unrolled_add),
    )
    .add_arg_type(base(intt()))
    .add_symbolic_ctx();

    egglog_test(
        &format!("{prog}"),
        &format!("(check (= {prog} {expected}))"),
        vec![prog.to_program(base(intt()), tuplet!(intt()))],
        intv(0),
        tuplev!(intv(8)),
        vec![],
    )
}
