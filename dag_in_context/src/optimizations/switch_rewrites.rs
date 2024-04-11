//#[cfg(test)]
//use crate::egglog_test;
/*
TODO rewrite when ifs have regions
#[test]
fn switch_rewrite_three_quarters_and() -> crate::Result {
    use crate::ast::*;

    let build = tif(and(tfalse(), ttrue()), int(1), int(2)).with_arg_types(emptyt(), base(intt()));

    let check =
        tif(tfalse(), tif(ttrue(), int(1), int(2)), int(2)).with_arg_types(emptyt(), base(intt()));

    egglog_test(
        &format!("{build}"),
        &format!("(check (= {build} {check}))"),
        vec![
            build.to_program(emptyt(), base(intt())),
            check.to_program(emptyt(), base(intt())),
        ],
        val_empty(),
        intv(2),
        vec![],
    )
}

#[test]
fn switch_rewrite_three_quarters_or() -> crate::Result {
    use crate::ast::*;

    let build = tif(or(tfalse(), ttrue()), int(1), int(2)).with_arg_types(emptyt(), base(intt()));

    let check =
        tif(tfalse(), int(1), tif(ttrue(), int(1), int(2))).with_arg_types(emptyt(), base(intt()));

    egglog_test(
        &format!("{build}"),
        &format!("(check (= {build} {check}))"),
        vec![
            build.to_program(emptyt(), base(intt())),
            check.to_program(emptyt(), base(intt())),
        ],
        val_empty(),
        intv(1),
        vec![],
    )
}

#[test]
fn switch_rewrite_three_quarters_purity() -> crate::Result {
    use crate::ast::*;

    let pure = get(single(ttrue()), 0).with_arg_types(emptyt(), base(boolt()));

    let build =
        tif(and(tfalse(), pure.clone()), int(1), int(2)).with_arg_types(emptyt(), base(intt()));

    let check =
        tif(tfalse(), tif(pure, int(1), int(2)), int(2)).with_arg_types(emptyt(), base(intt()));

    egglog_test(
        &format!("{build}"),
        &format!("(check (= {build} {check}))"),
        vec![build.to_program(emptyt(), base(intt()))],
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
    )
    .with_arg_types(base(statet()), base(boolt()));

    let build = tif(and(tfalse(), impure.clone()), int(1), int(2))
        .with_arg_types(base(statet()), base(intt()));

    let check = tif(tfalse(), tif(impure, int(1), int(2)), int(2))
        .with_arg_types(base(statet()), base(intt()));

    egglog_test(
        &format!("{build}"),
        &format!("(fail (check (= {build} {check})))"),
        vec![build.to_program(base(statet()), base(intt()))],
        statev(),
        intv(2),
        vec!["1".to_string()],
    )
}
 */
