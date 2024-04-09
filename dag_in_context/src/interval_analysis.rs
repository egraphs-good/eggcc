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
    let check = format!("
    (check (lo-bound {with_arg_types}) (IntB {lo}))
    (check (hi-bound {with_arg_types}) (IntB {hi}))
    ");
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
    let check = format!("
    (check (lo-bound {with_arg_types}) (BoolB {lo}))
    (check (hi-bound {with_arg_types}) (BoolB {hi}))
    ");
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
    let expr = add(int(1), int(2)).with_arg_types(emptyt(), base(intt()));
    let expr2 = int_ty(3, emptyt());

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
    let e = tif(c, int_ty(3, emptyt()), int_ty(4, emptyt()));

    int_interval_test(e, base(intt()), val_empty(), intv(3), 3, 3)
}

#[test]
fn if_interval() -> crate::Result {
    let e = tif(
        less_than(iarg(), int_ty(3, base(intt()))),
        int_ty(4, base(intt())),
        int_ty(5, base(intt())),
    );
    let f = function("main", base(intt()), base(intt()), e.clone()).func_with_arg_types();

    egglog_test(
        &format!("{f}"),
        &format!("
        (check (lo-bound {e}) (IntB 4))
        (check (hi-bound {e}) (IntB 5))
        "),
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
        int_ty(4, base(intt())),
        int_ty(5, base(intt())),
    );
    let outer = tif(
        less_eq(inner.clone(), int_ty(10, base(intt()))),
        int_ty(20, base(intt())),
        int_ty(30, base(intt())),
    );
    let f = function("main", base(intt()), base(intt()), outer.clone()).func_with_arg_types();

    egglog_test(
        &format!("{f}"),
        &format!("
        (check (lo-bound {inner}) (IntB 4))
        (check (hi-bound {inner}) (IntB 5))
        (check (lo-bound {outer}) (IntB 20))
        (check (hi-bound {outer}) (IntB 20))"),
        vec![f.to_program(base(intt()), base(intt()))],
        intv(2),
        intv(20),
        vec![],
    )
}
