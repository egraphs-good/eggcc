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
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(noctx());

    let check = tif(
        tfalse(),
        parallel!(ttrue()),
        int(1),
        tif(get(arg(), 0), empty(), int(1), int(2)),
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_ctx(noctx());

    egglog_test(
        &format!("(let build_ {build})"),
        &format!("(let check_ {check}) (check (= build_ check_))"),
        vec![],
        val_empty(),
        intv(1),
        vec![],
    )
}
