#[test]
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
}
