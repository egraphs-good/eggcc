#[cfg(test)]
use crate::{
    ast::*,
    egglog_test,
    interpreter::Value,
    schema::{RcExpr, Type},
};

#[cfg(test)]
fn type_test(inp: RcExpr, expected_ty: Type, arg: Value, expected_val: Value) -> crate::Result {
    type_test_with_log(inp, expected_ty, arg, expected_val, vec![])
}

#[cfg(test)]
fn type_test_with_log(
    inp: RcExpr,
    expected_ty: Type,
    arg: Value,
    expected_val: Value,
    expected_log: Vec<String>,
) -> crate::Result {
    let with_arg_types = inp.clone().with_arg_types(emptyt(), expected_ty.clone());
    let build = format!("{with_arg_types}");
    let check = format!("(check (HasType {with_arg_types} {expected_ty}))");
    egglog_test(
        &build,
        &check,
        vec![inp.to_program(emptyt(), expected_ty)],
        arg,
        expected_val,
        expected_log,
    )
}

#[cfg(test)]
#[allow(dead_code)]
fn type_error_test(inp: RcExpr) {
    let _ = egglog_test(&format!("{inp}"), "", vec![], emptyv(), emptyv(), vec![]);
}

#[cfg(test)]
fn _debug(inp: RcExpr, after: &str) -> crate::Result {
    egglog_test(&format!("{inp}"), after, vec![], emptyv(), emptyv(), vec![])
}

#[test]
fn primitives() -> crate::Result {
    type_test(int(3), base(intt()), intv(0), intv(3))?;
    type_test(int(12), base(intt()), intv(0), intv(12))?;
    type_test(ttrue(), base(boolt()), intv(0), val_bool(true))?;
    type_test(tfalse(), base(boolt()), intv(0), val_bool(false))?;
    type_test(empty(), emptyt(), intv(0), emptyv())
}

/* Fix type tests after dag semantics
#[test]
fn uops() -> crate::Result {
    let m = int(3);
    let x = ttrue();
    let y = tfalse();
    type_test(not(x), base(boolt()), val_int(0), val_bool(false))?;
    type_test(not(y), base(boolt()), val_int(0), val_bool(true))?;
    type_test_with_log(
        tprint(m),
        emptyt(),
        val_int(0),
        val_empty(),
        vec!["3".to_string()],
    )
}

#[test]
#[should_panic]
fn not_error() {
    type_error_test(not(int_ty(4, emptyt())));
}

#[test]
#[should_panic]
fn load_error() {
    type_error_test(load(int_ty(4, emptyt())));
}

#[test]
fn bops() -> crate::Result {
    let m = int(3);
    let n = int(12);
    type_test(
        add(m.clone(), n.clone()),
        base(intt()),
        val_int(0),
        val_int(15),
    )?;
    type_test(
        sub(m.clone(), n.clone()),
        base(intt()),
        val_int(0),
        val_int(-9),
    )?;
    type_test(
        mul(
            add(m.clone(), m.clone()),
            sub(add(n.clone(), n.clone()), m.clone()),
        ),
        base(intt()),
        val_int(0),
        val_int(126),
    )
}

#[test]
#[should_panic]
fn add_error() {
    type_error_test(add(int_ty(4, emptyt()), ttrue_ty(emptyt())));
}

#[test]
#[should_panic]
fn sub_error() {
    type_error_test(sub(tfalse_ty(emptyt()), ttrue_ty(emptyt())));
}

#[test]
#[should_panic]
fn mul_error() {
    type_error_test(mul(
        less_than(int_ty(4, emptyt()), int_ty(5, emptyt())),
        int_ty(3, emptyt()),
    ));
}

#[test]
#[should_panic]
fn less_than_error() {
    type_error_test(less_than(
        less_than(int_ty(4, emptyt()), int_ty(5, emptyt())),
        int_ty(3, emptyt()),
    ));
}

#[test]
#[should_panic]
fn and_error() {
    type_error_test(and(
        ttrue_ty(emptyt()),
        and(tfalse_ty(emptyt()), int_ty(2, emptyt())),
    ));
}

#[test]
#[should_panic]
fn or_error() {
    type_error_test(or(tfalse_ty(emptyt()), int_ty(2, emptyt())));
}

#[test]
fn pointers() -> crate::Result {
    let ptr = alloc(
        int_ty(12, emptyt()),
        arg_ty(base(statet())),
        pointert(intt()),
    );
    type_test(
        ptr.clone(),
        pointert(intt()),
        val_int(0),
        Value::Ptr(Pointer::new(0, 12, 0)),
    )?;
    type_test(
        write(ptr.clone(), int_ty(1, emptyt())),
        emptyt(),
        val_int(0),
        val_empty(),
    )?;
    type_test(
        ptradd(
            alloc(int(1), arg_ty(base(statet())), pointert(boolt())),
            add(int(1), int(2)),
        )
        .with_arg_types(emptyt(), pointert(boolt())),
        pointert(boolt()),
        val_int(0),
        Value::Ptr(Pointer::new(0, 1, 3)),
    )
}

#[test]
#[should_panic]
fn pointer_write_error() {
    let ptr = alloc(
        int_ty(12, emptyt()),
        arg_ty(base(statet())),
        pointert(intt()),
    );
    type_error_test(write(ptr.clone(), ttrue_ty(emptyt())));
}

#[test]
#[should_panic]
fn pointer_type_error() {
    type_error_test(alloc(
        less_than(int_ty(1, emptyt()), int_ty(2, emptyt())),
        arg_ty(base(statet())),
        base(boolt()),
    ));
}

#[test]
fn tuple() -> crate::Result {
    type_test(
        single(int_ty(30, emptyt())),
        tuplet!(intt()),
        val_int(0),
        val_vec(vec![val_int(30)]),
    )?;

    type_test(
        concat_par(single(int(20)), single(ttrue()))
            .with_arg_types(emptyt(), tuplet!(intt(), boolt())),
        tuplet!(intt(), boolt()),
        val_int(0),
        val_vec(vec![val_int(20), val_bool(true)]),
    )
}

#[test]
fn tuple_get() -> crate::Result {
    let t = concat_par(single(int(2)), concat_par(single(ttrue()), single(int(4))))
        .with_arg_types(emptyt(), tuplet!(intt(), boolt(), intt()));
    type_test(get(t.clone(), 0), base(intt()), val_int(0), val_int(2))?;
    type_test(get(t.clone(), 1), base(boolt()), val_int(0), val_bool(true))?;
    type_test(get(t, 2), base(intt()), val_int(0), val_int(4))?;
    let t2 = concat_seq(
        single(tfalse()),
        single(add(get(single(int(2)), 0), int(1))),
    )
    .with_arg_types(emptyt(), tuplet!(boolt(), intt()));
    type_test(get(t2, 0), base(boolt()), val_int(0), val_bool(false))
}

#[test]
fn ifs() -> crate::Result {
    type_test(
        tif(ttrue(), int(1), int(2)),
        base(intt()),
        val_int(0),
        val_int(1),
    )?;

    type_test(
        tif(
            less_than(int(2), int(3)),
            and(ttrue(), tfalse()),
            or(less_than(int(3), int(4)), ttrue()),
        ),
        base(boolt()),
        val_int(0),
        val_bool(false),
    )
}

#[test]
#[should_panic]
fn if_pred() {
    type_error_test(tif(
        int_ty(1, emptyt()),
        int_ty(2, emptyt()),
        int_ty(3, emptyt()),
    ));
}

#[test]
#[should_panic]
fn if_branches() {
    type_error_test(tif(
        ttrue_ty(emptyt()),
        int_ty(2, emptyt()),
        tfalse_ty(emptyt()),
    ));
}

#[test]
fn switches() -> crate::Result {
    type_test(
        switch_vec(int(1), vec![int(0), int(21)]).with_arg_types(emptyt(), base(intt())),
        base(intt()),
        val_int(0),
        val_int(21),
    )?;
    type_test(
        switch_vec(int(0), vec![ttrue()]).with_arg_types(emptyt(), base(boolt())),
        base(boolt()),
        val_int(0),
        val_bool(true),
    )?;
    type_test(
        switch_vec(int(2), vec![int(1), int(2), int(3), int(4)])
            .with_arg_types(emptyt(), base(intt())),
        base(intt()),
        val_int(0),
        val_int(3),
    )
}

#[test]
#[should_panic]
fn switch_pred() {
    type_error_test(switch_vec(
        ttrue_ty(emptyt()),
        vec![int_ty(1, emptyt()), int_ty(2, emptyt())],
    ));
}

#[test]
#[should_panic]
fn switch_branches() {
    type_error_test(switch_vec(
        int_ty(1, emptyt()),
        vec![ttrue_ty(emptyt()), int_ty(1, emptyt())],
    ));
}

#[test]
fn lets() -> crate::Result {
    let inp = tlet(int(4), add(iarg(), iarg()));
    type_test(inp, base(intt()), val_int(0), val_int(8))
}

#[test]
#[should_panic]
fn let_type_error() {
    type_error_test(tlet(
        int_ty(1, emptyt()),
        and(barg(), ttrue_ty(base(intt()))),
    ));
}

#[test]
fn let_arg_types() -> crate::Result {
    let expr = and(barg(), ttrue_ty(base(boolt())));
    let build = format!("{expr}");
    let expected_ty = base(boolt());
    let check = format!(
        "(check (HasType {expr} {expected_ty}))
(check (HasArgType {expr} {expected_ty}))"
    );
    crate::egglog_test(
        &build,
        &check,
        vec![expr.to_program(base(boolt()), base(boolt()))],
        val_bool(true),
        val_bool(true),
        vec![],
    )
}

#[test]
fn loops() -> crate::Result {
    let l1 = dowhile(single(int(1)), concat_seq(single(tfalse()), single(int(3))));
    type_test(l1, tuplet!(intt()), val_int(0), val_vec(vec![val_int(3)]))?;

    let l15 = dowhile(
        single(int(1)),
        concat_seq(single(tfalse()), single(add(getat(0), int(1)))),
    );
    type_test(l15, tuplet!(intt()), val_int(0), val_vec(vec![val_int(2)]))?;

    // while x < 4, x++
    let pred = single(less_than(getat(0), int(4)));
    let body = single(add(getat(0), int(1)));
    let l2 = dowhile(single(int(1)), concat_seq(pred, body));
    type_test(l2, tuplet!(intt()), val_int(0), val_vec(vec![val_int(5)]))?;

    // x = 1, y = 2
    // do (x = x + 1, y = x * 2)
    // while (x < 5)
    let l2 = dowhile(
        concat_par(single(int(1)), single(int(2))),
        concat_par(
            single(less_than(getat(0), int(5))),
            concat_par(single(add(getat(0), int(1))), single(mul(getat(0), int(2)))),
        ),
    );

    type_test(
        l2,
        tuplet!(intt(), intt()),
        val_int(0),
        val_vec(vec![val_int(6), val_int(10)]),
    )
}

#[test]
#[should_panic]
fn loop_input_error() {
    // input is not a tuple
    type_error_test(dowhile(
        int_ty(4, emptyt()),
        concat_par(
            single(ttrue_ty(tuplet!(intt()))),
            single(int_ty(5, tuplet!(intt()))),
        ),
    ))
}

#[test]
#[should_panic]
fn loop_predbody_error() {
    // pred-body is not a tuple
    type_error_test(dowhile(
        single(int_ty(4, emptyt())),
        ttrue_ty(tuplet!(intt())),
    ))
}

#[test]
#[should_panic]
fn loop_pred_error() {
    // pred is not a bool
    type_error_test(dowhile(
        single(int_ty(1, emptyt())),
        concat_par(
            single(int_ty(2, tuplet!(intt()))),
            single(int_ty(3, tuplet!(intt()))),
        ),
    ))
}

#[test]
#[should_panic]
fn loop_inputs_outputs_error1() {
    // input is bool, output is int
    type_error_test(dowhile(
        single(ttrue_ty(emptyt())),
        concat_par(
            single(tfalse_ty(tuplet!(boolt()))),
            single(int_ty(2, tuplet!(boolt()))),
        ),
    ))
}

#[test]
#[should_panic]
fn loop_inputs_outputs_error2() {
    // input is (int, bool), output is (int)
    type_error_test(dowhile(
        concat_seq(single(int_ty(2, emptyt())), single(ttrue_ty(emptyt()))),
        concat_par(
            single(tfalse_ty(tuplet!(intt(), boolt()))),
            single(int_ty(2, tuplet!(intt(), boolt()))),
        ),
    ))
}

#[test]
fn funcs_and_calls() -> crate::Result {
    let body = add(arg(), int(2)).with_arg_types(base(intt()), base(intt()));
    let f = function("f", base(intt()), base(intt()), body.clone()).func_with_arg_types();
    let c = call("f", int_ty(4, emptyt()));
    egglog_test(
        &format!("{f}{c}"),
        &format!(
            "
        (check (HasType {body} (Base (IntT))))
        (check (HasType {c} (Base (IntT))))"
        ),
        vec![f.to_program(base(intt()), base(intt()))],
        val_int(4),
        val_int(6),
        vec![],
    )
}

#[test]
fn repro_argtype_bug() -> crate::Result {
    type_test(
        concat_par(tlet(int(1), empty()), tlet(ttrue(), empty()))
            .with_arg_types(emptyt(), emptyt()),
        emptyt(),
        val_empty(),
        val_empty(),
    )
}
 */
