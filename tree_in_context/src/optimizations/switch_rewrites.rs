#[cfg(test)]
use crate::egglog_test;

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
        val_int(2),
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
        val_int(1),
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
        val_int(2),
        vec![],
    )?;

    let impure =
        get(concat_par(tprint(int(1)), single(ttrue())), 0).with_arg_types(emptyt(), base(boolt()));

    let build =
        tif(and(tfalse(), impure.clone()), int(1), int(2)).with_arg_types(emptyt(), base(intt()));

    let check =
        tif(tfalse(), tif(impure, int(1), int(2)), int(2)).with_arg_types(emptyt(), base(intt()));

    egglog_test(
        &format!("{build}"),
        &format!("(fail (check (= {build} {check})))"),
        vec![build.to_program(emptyt(), base(intt()))],
        val_empty(),
        val_int(2),
        vec!["1".to_string()],
    )
}
