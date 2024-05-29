#[cfg(test)]
use crate::egglog_test;

#[test]
fn switch_rewrite_three_quarters_and() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let build = tif(and(tfalse(), ttrue()), empty(), int(1), int(2))
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(Assumption::dummy());

    let check = tif(
        tfalse(),
        parallel!(ttrue()),
        tif(get(arg(), 0), empty(), int(1), int(2)),
        int(2),
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_ctx(Assumption::dummy());

    egglog_test(
        &format!("(let build_ {})\n{}", build.value, build.get_unions()),
        &format!(
            "(let check_ {})\n{}\n(check (= build_ check_))",
            check.value,
            check.get_unions()
        ),
        vec![],
        val_empty(),
        intv(2),
        vec![],
    )
}

#[test]
fn switch_rewrite_three_quarters_or() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let build = tif(or(tfalse(), ttrue()), empty(), int(1), int(2))
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(Assumption::dummy());

    let check = tif(
        tfalse(),
        parallel!(ttrue()),
        int(1),
        tif(get(arg(), 0), empty(), int(1), int(2)),
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_ctx(Assumption::dummy());

    egglog_test(
        &format!("(let build_ {})\n{}", build.value, build.get_unions()),
        &format!(
            "(let check_ {})\n{}\n(check (= build_ check_))",
            check.value,
            check.get_unions()
        ),
        vec![],
        val_empty(),
        intv(1),
        vec![],
    )
}

#[test]
fn switch_rewrite_forward_pred() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let ctx_ty = tuplet!(boolt());

    let arg = get(arg_ty(ctx_ty.clone()), 0);

    let build = get(
        tif(arg.clone(), empty(), single(ttrue()), single(tfalse())),
        0,
    )
    .add_arg_type(ctx_ty.clone())
    .add_ctx(Assumption::dummy());

    let check = arg
        .clone()
        .add_arg_type(ctx_ty.clone())
        .add_ctx(Assumption::dummy());

    egglog_test(
        &format!("(let build_ {})\n{}", build.value, build.get_unions()),
        &format!(
            "(let check_ {})\n{}\n(check (= build_ check_))",
            check.value,
            check.get_unions()
        ),
        vec![],
        val_empty(),
        intv(1),
        vec![],
    )
}

#[test]
fn switch_rewrite_negate_pred() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let ctx_ty = tuplet!(boolt());

    let arg = get(arg_ty(ctx_ty.clone()), 0);

    let build = get(
        tif(arg.clone(), empty(), single(tfalse()), single(ttrue())),
        0,
    )
    .add_arg_type(ctx_ty.clone())
    .add_ctx(Assumption::dummy());

    let check = not(arg.clone())
        .add_arg_type(ctx_ty.clone())
        .add_ctx(Assumption::dummy());

    egglog_test(
        &format!("(let build_ {})\n{}", build.value, build.get_unions()),
        &format!(
            "(let check_ {})\n{}\n(check (= build_ check_))",
            check.value,
            check.get_unions()
        ),
        vec![],
        val_empty(),
        intv(1),
        vec![],
    )
}

#[test]
fn single_branch_switch() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let build = switch_vec(
        int(1),
        empty(),
        vec![
            switch!(int(0), empty(); int(12)),
            switch!(int(0), empty(); int(12)),
        ],
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_ctx(Assumption::dummy());

    let check = int(1)
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(Assumption::dummy());

    egglog_test(
        &format!("(let build_ {})\n{}", build.value, build.get_unions()),
        &format!(
            "(let check_ {})\n{}\n(check (!= build_ check_))",
            check.value,
            check.get_unions()
        ),
        vec![],
        val_empty(),
        intv(1),
        vec![],
    )
}
