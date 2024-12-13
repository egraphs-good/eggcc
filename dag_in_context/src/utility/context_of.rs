#[test]
fn test_context_of() -> crate::Result {
    use crate::ast::*;
    use crate::schema::Assumption;
    let ctx = Assumption::InFunc("main".to_string());

    // fn main(x): if x = 5 then x else 4
    let pred = eq(arg(), int(5));
    let body = tif(pred, arg(), arg(), int(4))
        .with_arg_types(base(intt()), base(intt()))
        .with_arg_types(base(intt()), base(intt()));
    let (body_with_context, body_cache) = body.clone().add_ctx(ctx.clone());
    let (build, build_cache) = function("main", base(intt()), base(intt()), body.clone())
        .func_with_arg_types()
        .func_add_ctx();
    let ctx = format!("{}", ctx);

    // If statement should have the context of its predicate
    let check = format!(
        "
(let pred-ctx {ctx})
(let pred (Bop (Eq) (Arg (Base (IntT)) {ctx}) (Const (Int 5) (Base (IntT)) {ctx})))
(check (ContextOf pred pred-ctx))
(let if {body_with_context})
{}
(check (ContextOf if pred-ctx))",
        body_cache.get_unions(),
    );

    crate::egglog_test(
        &format!("(let build {build})\n{}", build_cache.get_unions()),
        &check,
        vec![build.func_to_program()],
        intv(5),
        intv(5),
        vec![],
    )
}

// Check that a constant has ContextOf
#[test]
fn test_context_of_base_case() -> crate::Result {
    use crate::schema::Assumption;
    let ctx = format!("{}", Assumption::dummy());
    let build = format!("(Const (Int 5) (Base (IntT)) {ctx})");
    let check = format!("(ContextOf (Const (Int 5) (Base (IntT)) {ctx}) {ctx})");

    crate::egglog_test(
        &format!("(let build {build})"),
        &check,
        vec![],
        crate::ast::emptyv(),
        crate::ast::emptyv(),
        vec![],
    )
}

#[test]
#[should_panic]
fn test_context_of_panics_if_two() {
    use crate::ast::*;
    use crate::schema::Assumption;
    let ctx1 = format!("{}", Assumption::dummy());
    let ctx2 = inif(
        true,
        ttrue().with_arg_types(tuplet!(), base(boolt())),
        arg_ty(tuplet!()),
    );
    let build = format!("
        (let ctx1 {ctx1})
        (let ctx2 {ctx2})
        (let conflict-expr (Bop (And) (Const (Bool false) (Base (BoolT)) ctx1) (Const (Bool true) (Base (BoolT)) ctx2)))");
    let check = "";

    let _ = crate::egglog_test(
        &build,
        check,
        vec![],
        crate::ast::emptyv(),
        crate::ast::emptyv(),
        vec![],
    );
}

// Functions should not have a context
#[test]
fn test_context_of_no_func_context() -> crate::Result {
    use crate::ast::*;

    let (build, cache) = function(
        "main",
        emptyt(),
        base(intt()),
        get(
            dowhile(
                single(int(5)),
                parallel!(tfalse(), add(get(arg(), 0), int(4))),
            ),
            0,
        ),
    )
    .func_with_arg_types()
    .func_add_ctx();

    let check = format!("(fail (check (ContextOf {build} ctx)))");

    crate::egglog_test(
        &format!("{build}\n{}", cache.get_unions()),
        &check,
        vec![build.to_program(base(intt()), base(intt()))],
        emptyv(),
        intv(9),
        vec![],
    )
}
