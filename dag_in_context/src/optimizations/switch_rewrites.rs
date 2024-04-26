#[cfg(test)]
use crate::egglog_test;

#[test]
fn switch_rewrite_three_quarters_and() -> crate::Result {
    use crate::ast::*;

    let build = tif(and(tfalse(), ttrue()), empty(), int(1), int(2))
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(noctx());

    let check = tif(
        tfalse(),
        parallel!(ttrue()),
        tif(get(arg(), 0), empty(), int(1), int(2)),
        int(2),
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_ctx(noctx());

    egglog_test(
        &format!("(let build_ {build})"),
        &format!("(let check_ {check}) (check (= build_ check_))"),
        vec![],
        val_empty(),
        intv(2),
        vec![],
    )
}

#[test]
fn switch_rewrite_three_quarters_or() -> crate::Result {
    use crate::ast::*;

    let build = tif(or(tfalse(), ttrue()), empty(), int(1), int(2))
        .to_program(emptyt(), base(intt()))
        .add_context();

    let check = tif(
        tfalse(),
        empty(),
        int(1),
        tif(ttrue(), empty(), int(1), int(2)),
    )
    .to_program(emptyt(), base(intt()))
    .add_context();

    egglog_test(
        &format!("(let b {build})"),
        &format!("(let c {check}) (check (= b c))"),
        vec![build, check],
        val_empty(),
        intv(1),
        vec![],
    )
}

#[test]
fn switch_rewrite_three_quarters_purity() -> crate::Result {
    use crate::ast::*;

    let pure = get(single(ttrue()), 0);

    let build = tif(and(tfalse(), pure.clone()), empty(), int(1), int(2))
        .to_program(emptyt(), base(intt()))
        .add_context();

    let check = tif(
        tfalse(),
        empty(),
        tif(pure, empty(), int(1), int(2)),
        int(2),
    )
    .to_program(emptyt(), base(intt()))
    .add_context();

    egglog_test(
        &format!("(let b {build})"),
        &format!("(let c {check}) (check (= b c))"),
        vec![build, check],
        val_empty(),
        intv(2),
        vec![],
    )?;

    let impure = get(
        dowhile(
            parallel![arg(), tfalse()],
            parallel![tfalse(), tprint(int(1), getat(0)), ttrue(),],
        ),
        1,
    );

    let build = tif(and(tfalse(), impure.clone()), empty(), int(1), int(2))
        .to_program(base(statet()), base(intt()));

    let check = tif(
        tfalse(),
        empty(),
        tif(impure, empty(), int(1), int(2)),
        int(2),
    )
    .to_program(base(statet()), base(intt()));

    egglog_test(
        &format!("(let b {build})"),
        &format!("(let c {check}) (fail (check (= b c)))"),
        vec![build],
        statev(),
        intv(2),
        vec!["1".to_string()],
    )
}
