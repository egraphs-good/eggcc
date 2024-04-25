#[test]
fn test_context_of() -> crate::Result {
    use crate::ast::*;

    // fn main(x): if x = 5 then x else 4
    let pred = eq(arg(), int(5));
    let body = tif(pred, arg(), arg(), int(4))
        .with_arg_types(base(intt()), base(intt()))
        .with_arg_types(base(intt()), base(intt()));
    let body_with_context = body.clone().add_ctx(noctx());
    let build = function("main", base(intt()), base(intt()), body.clone())
        .func_with_arg_types()
        .func_add_ctx();

    // If statement should have the context of its predicate
    let check = format!("
        (let pred-ctx (NoContext))
        (let pred (Bop (Eq) (Arg (Base (IntT)) (NoContext)) (Const (Int 5) (Base (IntT)) (NoContext))))
        (check (ContextOf pred pred-ctx))
        (let if {body_with_context})
        (check (ContextOf if pred-ctx))
        ");

    crate::egglog_test(
        &format!("(let build {build})"),
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
    let build = "(Const (Int 5) (Base (IntT)) (NoContext))";
    let check = "(ContextOf (Const (Int 5) (Base (IntT)) (NoContext)) (NoContext))";

    crate::egglog_test(
        &format!("(let build {build})"),
        check,
        vec![],
        crate::ast::val_empty(),
        crate::ast::val_empty(),
        vec![],
    )
}

#[test]
#[should_panic]
fn test_context_of_panics_if_two() {
    let build = "
        (let ctx1 (NoContext))
        (let ctx2 (NoContext))
        (let conflict-expr (Bop (And) (Const (Bool false) (Base (BoolT)) ctx1) (Const (Bool true) (Base (BoolT)) ctx2)))";
    let check = "";

    let _ = crate::egglog_test(
        build,
        check,
        vec![],
        crate::ast::val_empty(),
        crate::ast::val_empty(),
        vec![],
    );
}

// Functions should not have a context
#[test]
fn test_context_of_no_func_context() -> crate::Result {
    use crate::ast::*;

    let build = function(
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

    let check = format!("(fail (check (ContextOf {} ctx)))", build.clone());

    crate::egglog_test(
        &build.to_string(),
        &check,
        vec![build.to_program(base(intt()), base(intt()))],
        val_empty(),
        intv(9),
        vec![],
    )
}
