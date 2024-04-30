#[cfg(test)]
use crate::{ast::*, egglog_test, interpreter::*, schema::*};

#[cfg(test)]
fn int_interval_test(
    inp: RcExpr,
    expected_ty: Type,
    arg: Value,
    expected_val: Value,
    lo: i64,
    hi: i64,
) -> crate::Result {
    let with_arg_types = inp.clone().with_arg_types(emptyt(), expected_ty.clone());
    let check = format!(
        "
    (check (lo-bound {with_arg_types}) (IntB {lo}))
    (check (hi-bound {with_arg_types}) (IntB {hi}))
    "
    );
    interval_test(with_arg_types, expected_ty, arg, expected_val, check)
}

#[cfg(test)]
fn bool_interval_test(
    inp: RcExpr,
    expected_ty: Type,
    arg: Value,
    expected_val: Value,
    lo: bool,
    hi: bool,
) -> crate::Result {
    let with_arg_types = inp.clone().with_arg_types(emptyt(), expected_ty.clone());
    let check = format!(
        "
    (check (lo-bound {with_arg_types}) (BoolB {lo}))
    (check (hi-bound {with_arg_types}) (BoolB {hi}))
    "
    );
    interval_test(with_arg_types, expected_ty, arg, expected_val, check)
}

#[cfg(test)]
fn interval_test(
    prog: RcExpr,
    expected_ty: Type,
    arg: Value,
    expected_val: Value,
    check: String,
) -> crate::Result {
    let build = format!("{prog}");
    egglog_test(
        &build,
        &check,
        vec![prog.to_program(emptyt(), expected_ty)],
        arg,
        expected_val,
        vec![],
    )
}

#[test]
fn constant_interval_test() -> crate::Result {
    let e = int(3);
    int_interval_test(e, base(intt()), val_empty(), intv(3), 3, 3)
}

#[test]
fn constant_interval_test2() -> crate::Result {
    let e = ttrue();
    bool_interval_test(e, base(boolt()), val_empty(), val_bool(true), true, true)
}

#[test]
fn constant_fold() -> crate::Result {
    let e = add(int(3), int(2));
    int_interval_test(e, base(intt()), val_empty(), intv(5), 5, 5)
}

#[test]
fn test_add_constant_fold() -> crate::Result {
    use crate::ast::*;
    let expr = add(int(1), int(2))
        .with_arg_types(emptyt(), base(intt()))
        .add_ctx(noctx());
    let expr2 = int_ty(3, emptyt()).add_ctx(noctx());

    egglog_test(
        &format!("{expr}"),
        &format!("(check (= {expr} {expr2}))"),
        vec![
            expr.to_program(emptyt(), base(intt())),
            expr2.to_program(emptyt(), base(intt())),
        ],
        val_empty(),
        intv(3),
        vec![],
    )
}

#[test]
fn test_add_interval() -> crate::Result {
    let e = add(int(3), int(4)).with_arg_types(emptyt(), base(intt()));
    int_interval_test(e, base(intt()), val_empty(), intv(7), 7, 7)
}

#[test]
fn test_lt_interval() -> crate::Result {
    let e = less_than(int(2), int(3)).with_arg_types(emptyt(), base(boolt()));
    bool_interval_test(e, base(boolt()), val_empty(), val_bool(true), true, true)
}

#[test]
fn test_if_constant_fold() -> crate::Result {
    let c = less_than(int(2), int(3)).with_arg_types(emptyt(), base(boolt()));
    let e = tif(c, arg(), int_ty(3, emptyt()), int_ty(4, emptyt()));

    int_interval_test(e, base(intt()), val_empty(), intv(3), 3, 3)
}

#[test]
fn if_interval() -> crate::Result {
    let e = tif(
        less_than(iarg(), int_ty(3, base(intt()))),
        arg(),
        int(4),
        int_ty(5, base(intt())),
    )
    .with_arg_types(base(intt()), base(intt()));
    let f = function("main", base(intt()), base(intt()), e.clone()).func_with_arg_types();

    egglog_test(
        &format!("{f}"),
        &format!(
            "
        (check (lo-bound {e}) (IntB 4))
        (check (hi-bound {e}) (IntB 5))
        "
        ),
        vec![f.to_program(base(intt()), base(intt()))],
        intv(1),
        intv(4),
        vec![],
    )
}

#[test]
fn nested_if() -> crate::Result {
    let inner = tif(
        less_than(iarg(), int_ty(3, base(intt()))),
        arg(),
        int_ty(4, base(intt())),
        int_ty(5, base(intt())),
    )
    .with_arg_types(base(intt()), base(intt()));
    let outer = tif(
        less_eq(inner.clone(), int_ty(10, base(intt()))),
        arg(),
        int_ty(20, base(intt())),
        int_ty(30, base(intt())),
    )
    .with_arg_types(base(intt()), base(intt()));
    let f = function("main", base(intt()), base(intt()), outer.clone()).func_with_arg_types();

    egglog_test(
        &format!("{f}"),
        &format!(
            "
        (check (lo-bound {inner}) (IntB 4))
        (check (hi-bound {inner}) (IntB 5))
        (check (lo-bound {outer}) (IntB 20))
        (check (hi-bound {outer}) (IntB 20))"
        ),
        vec![f.to_program(base(intt()), base(intt()))],
        intv(2),
        intv(20),
        vec![],
    )
}

#[test]
fn context_if() -> crate::Result {
    // input <= 0
    let cond = less_eq(iarg(), int_ty(0, base(intt())));

    // y = if cond {-1 * input} else {input}
    // interval analysis should tell us that y is always positive (= (lo-bound y) (IntB 0))
    let y = tif(cond, parallel!(arg()), mul(getat(0), int(-1)), getat(0));

    // z = y < 0
    // interval analysis should tell us that z is false
    let z = less_than(y.clone(), int_ty(0, base(intt())));

    let f = function("main", base(intt()), base(boolt()), z.clone()).func_with_arg_types();
    let prog = f.to_program(base(intt()), base(boolt()));
    let with_context = prog.add_context();
    let term = with_context.entry.func_body().unwrap();

    egglog_test(
        &format!("{with_context}"),
        &format!("(check (= {term} (Const (Bool false) (Base (IntT)) somectx)))"),
        vec![with_context],
        intv(4),
        val_bool(false),
        vec![],
    )
}

#[test]
fn simple_less_than() -> crate::Result {
    // 0 <= input
    let cond = less_eq(int_ty(0, base(intt())), int(-1));
    let prog = program!(function("main", base(intt()), base(boolt()), cond.clone()),);
    let with_context = prog.add_context();
    let term = with_context.entry.func_body().unwrap();

    egglog_test(
        &format!("{with_context}"),
        &format!("(check (= {term} (Const (Bool false) (Base (IntT)) somectx)))"),
        vec![with_context],
        intv(4),
        val_bool(false),
        vec![],
    )
}

#[test]
fn context_if_rev() -> crate::Result {
    // 0 <= input
    let cond = less_eq(int_ty(0, base(intt())), iarg());

    // y = if cond {-1 * input} else {input}
    // interval analysis should tell us that y is always negative (= (hi-bound y) (IntB 0))
    let y = tif(cond, parallel!(iarg()), mul(getat(0), int(-1)), getat(0));

    // z = 0 < y
    // interval analysis should tell us that z is false
    let z = less_than(int_ty(0, base(intt())), y.clone());

    let f = function("main", base(intt()), base(boolt()), z.clone()).func_with_arg_types();
    let prog = f.to_program(base(intt()), base(boolt()));
    let with_context = prog.add_context();
    let term = with_context.entry.func_body().unwrap();

    egglog_test(
        &format!("{with_context}"),
        &format!(
            "
(check (= {term} (Const (Bool false) (Base (IntT)) (NoContext))))"
        ),
        vec![with_context],
        intv(4),
        val_bool(false),
        vec![],
    )
}

#[test]
fn context_if_with_state() -> crate::Result {
    let input_type = tuplet_vec(vec![intt(), statet()]);
    let output_type = tuplet_vec(vec![statet()]);
    let input_arg = arg_ty(input_type.clone());
    let pred = less_eq(get(input_arg.clone(), 0), int_ty(0, input_type.clone()));
    let inputs = concat(
        single(get(input_arg.clone(), 1)),
        concat(
            single(int_ty(0, input_type.clone())),
            single(get(input_arg.clone(), 0)),
        ),
    );

    let then = concat(single(getat(0)), concat(single(getat(2)), single(getat(1))));

    let els = concat(
        single(getat(0)),
        concat(
            single(mul(
                getat(2),
                int_ty(-1, tuplet_vec(vec![statet(), intt(), intt()])),
            )),
            single(getat(1)),
        ),
    );

    let body = single(tprint(
        less_eq(
            get(tif(pred.clone(), inputs.clone(), then, els), 1),
            int_ty(0, input_type.clone()),
        ),
        get(input_arg.clone(), 1),
    ))
    .with_arg_types(input_type.clone(), output_type.clone());

    let f = function(
        "main",
        input_type.clone(),
        output_type.clone(),
        body.clone(),
    )
    .func_with_arg_types();
    let prog = f
        .to_program(input_type.clone(), output_type.clone())
        .with_arg_types()
        .add_context();

    let body_with_ctx = prog.entry.func_body().unwrap();

    let expected = single(tprint(
        ttrue_ty(input_type.clone()),
        get(input_arg.clone(), 1),
    ));

    egglog_test(
        &format!("{prog}"),
        &format!(
            "
(check (= {expected} {body_with_ctx}))
"
        ),
        vec![prog],
        val_vec(vec![intv(4), statev()]),
        val_vec(vec![statev()]),
        vec!["true".to_string()],
    )
}
