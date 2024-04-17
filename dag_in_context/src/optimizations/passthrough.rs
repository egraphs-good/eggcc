#[cfg(test)]
use crate::egglog_test;

#[test]
fn passthrough_if_predicate() -> crate::Result {
    use crate::ast::*;

    let build = get(
        tif(
            less_than(arg(), int(5)),
            empty(),
            single(ttrue()),
            single(tfalse()),
        ),
        0,
    );
    let check = less_than(arg(), int(5));

    let build = build.to_program(base(intt()), base(boolt())).add_context();
    let check = check.to_program(base(intt()), base(boolt())).add_context();
    egglog_test(
        &format!("(let b {build})"),
        &format!("(let c {check}) (check (= b c))"),
        vec![build, check],
        intv(3),
        val_bool(true),
        vec![],
    )
}
