#[cfg(test)]
use crate::egglog_test;

#[test]
fn test_simple_prop_eq() -> crate::Result {
    use crate::ast::*;
    // (let (Concat (Parallel) (Arg) (Arg))
    //      (Add (Get (Arg) 0) (Get (Arg) 1)))
    let prog = function(
        "main",
        base(intt()),
        base(intt()),
        tlet(parallel!(arg(), arg()), add(getat(0), getat(1)))
            .with_arg_types(base(intt()), base(intt())),
    )
    .func_with_arg_types();
    // (let (Concat (Parallel) (Arg) (Arg))
    //      (Mul (Get (Arg) 0) 2))
    let expected = function(
        "main",
        base(intt()),
        base(intt()),
        tlet(parallel!(arg(), arg()), mul(getat(0), int(2))),
    )
    .func_with_arg_types();
    egglog_test(
        &format!(
            "
(let expr (AddFuncContext {}))
(let expr2 (AddFuncContext {}))",
            prog, expected
        ),
        "(check (= expr expr2))",
        vec![
            prog.to_program(base(intt()), base(intt())),
            expected.to_program(base(intt()), base(intt())),
        ],
        val_int(2),
        val_int(4),
        vec![],
    )
}

#[test]
fn test_complex_prop_eq() -> crate::Result {
    use crate::ast::*;
    let prog = function(
        "main",
        tuplet!(intt(), intt()),
        tuplet!(intt(), intt()),
        tlet(
            parallel!(getat(1), add(getat(1), getat(0))),
            parallel!(sub(getat(1), int(0)), getat(0)),
        ),
    )
    .func_with_arg_types();
    let expected = function(
        "main",
        tuplet!(intt(), intt()),
        tuplet!(intt(), intt()),
        tlet(
            parallel!(getat(1), add(getat(1), getat(0))),
            parallel!(int(20), int(21)),
        ),
    )
    .func_with_arg_types();
    let ty = tuplet!(intt(), intt());
    let ctx = infunc("main");
    let arg_top = in_context(ctx.clone(), arg_ty(ty.clone()));
    let top_level_term = parallel!(
        sub(
            add(get(arg_top.clone(), 1), get(arg_top.clone(), 0)),
            in_context(ctx.clone(), int(0))
        ),
        get(arg_top, 1)
    )
    .with_arg_types(ty.clone(), ty.clone());
    let other_top_level = parallel!(in_context(ctx.clone(), int(20)), in_context(ctx, int(21)))
        .with_arg_types(ty.clone(), ty.clone());
    egglog_test(
        &format!(
            "
(let expr (AddFuncContext {prog}))
(let expr2 (AddFuncContext {expected}))
;; equality we want propagated down
(union {top_level_term} {other_top_level})
"
        ),
        "(check (= expr expr2))",
        vec![
            prog.to_program(ty.clone(), ty.clone()),
            expected.to_program(ty.clone(), ty.clone()),
        ],
        tuple!(int(20), int(21)),
        val_int(4),
        vec![],
    )
}
