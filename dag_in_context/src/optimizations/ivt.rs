#[test]
fn ivt_if_more_outputs_than_inputs() -> crate::Result {
    // Initialize the logger for this test
    let _ = env_logger::try_init();

    use crate::ast::*;
    use crate::egglog_test;

    let cond = less_than(getat(0), int(10));
    // if statement has 3 outputs but only 1 input
    let if_in_loop = tif(
        cond.clone(),
        parallel!(add(getat(0), getat(1))),
        parallel!(add(getat(0), int(1)), int(2)),
        parallel!(getat(0), int(3)),
    );

    // 1 is passed through the loop
    let my_loop = dowhile(
        parallel!(int(0), int(0), int(1)),
        parallel!(
            cond,
            get(if_in_loop.clone(), 0),
            get(if_in_loop, 1),
            getat(2)
        ),
    )
    .add_arg_type(tuplet!())
    .add_ctx(infunc("main"))
    .0;

    let added = add(getat(0), int(1));
    let inner_loop_new = dowhile(
        arg(),
        parallel!(
            less_than(added.clone(), int(10)),
            add(added, int(2)),
            getat(1)
        ),
    );
    let expected_if = tif(ttrue(), parallel!(int(0), int(1)), inner_loop_new, arg());

    let expected = parallel!(get(expected_if.clone(), 0), int(3), get(expected_if, 1))
        .add_arg_type(tuplet!())
        .add_symbolic_ctx();

    egglog_test(
        &format!("(let myloop {my_loop})"),
        &format!(
            "
        (check (= myloop {expected}))"
        ),
        vec![
            my_loop.to_program(emptyt(), tuplet!(intt(), intt(), intt())),
            expected
                .add_ctx(infunc("main"))
                .0
                .to_program(emptyt(), tuplet!(intt(), intt(), intt())),
        ],
        tuplev!(),
        tuplev!(intv(12), intv(3), intv(1)),
        vec![],
    )
}
