#[cfg(test)]
use crate::egglog_test;

#[test]
fn switch_rewrite_three_quarters_and() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let (build, build_cache) = tif(and(tfalse(), ttrue()), empty(), int(1), int(2))
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(Assumption::dummy());

    let (check, check_cache) = tif(
        tfalse(),
        parallel!(ttrue()),
        tif(get(arg(), 0), empty(), int(1), int(2)),
        int(2),
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_ctx(Assumption::dummy());

    egglog_test(
        &format!("(let build_ {build})\n{}", build_cache.get_unions()),
        &format!(
            "(let check_ {check})\n{}\n(check (= build_ check_))",
            check_cache.get_unions()
        ),
        vec![],
        emptyv(),
        intv(2),
        vec![],
    )
}

#[test]
fn switch_rewrite_three_quarters_or() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;

    let (build, build_cache) = tif(or(tfalse(), ttrue()), empty(), int(1), int(2))
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(Assumption::dummy());

    let (check, check_cache) = tif(
        tfalse(),
        parallel!(ttrue()),
        int(1),
        tif(get(arg(), 0), empty(), int(1), int(2)),
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_ctx(Assumption::dummy());

    egglog_test(
        &format!("(let build_ {build})\n{}", build_cache.get_unions()),
        &format!(
            "(let check_ {check})\n{}\n(check (= build_ check_))",
            check_cache.get_unions()
        ),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}

#[test]
fn switch_rewrite_forward_pred() -> crate::Result {
    use crate::ast::*;

    let ctx_ty = tuplet!(boolt());

    let arg = get(arg_ty(ctx_ty.clone()), 0);

    let (build, build_cache) = get(
        tif(arg.clone(), empty(), single(ttrue()), single(tfalse())),
        0,
    )
    .add_arg_type(ctx_ty.clone())
    .add_dummy_ctx();

    let (check, check_cache) = arg.clone().add_arg_type(ctx_ty.clone()).add_dummy_ctx();

    egglog_test(
        &format!("(let build_ {build})\n{}", build_cache.get_unions()),
        &format!(
            "(let check_ {check})\n{}\n(check (= build_ check_))",
            check_cache.get_unions()
        ),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}

#[test]
fn switch_rewrite_negate_pred() -> crate::Result {
    use crate::ast::*;

    let ctx_ty = tuplet!(boolt());

    let arg = get(arg_ty(ctx_ty.clone()), 0);

    let (build, build_cache) = get(
        tif(arg.clone(), empty(), single(tfalse()), single(ttrue())),
        0,
    )
    .add_arg_type(ctx_ty.clone())
    .add_dummy_ctx();

    let (check, check_cache) = not(arg.clone())
        .add_arg_type(ctx_ty.clone())
        .add_dummy_ctx();

    egglog_test(
        &format!("(let build_ {build})\n{}", build_cache.get_unions()),
        &format!(
            "(let check_ {check})\n{}\n(check (= build_ check_))",
            check_cache.get_unions()
        ),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}

#[test]
fn single_branch_switch() -> crate::Result {
    use crate::ast::*;

    let (build, build_cache) = switch_vec(
        int(1),
        empty(),
        vec![
            switch!(int(0), empty(); int(12)),
            switch!(int(0), empty(); int(12)),
        ],
    )
    .with_arg_types(emptyt(), base(intt()))
    .add_dummy_ctx();

    let (check, check_cache) = int(1)
        .with_arg_types(emptyt(), base(intt()))
        .add_dummy_ctx();

    egglog_test(
        &format!("(let build_ {build})\n{}", build_cache.get_unions()),
        &format!(
            "(let check_ {check})\n{}\n(check (!= build_ check_))",
            check_cache.get_unions()
        ),
        vec![],
        emptyv(),
        intv(1),
        vec![],
    )
}
