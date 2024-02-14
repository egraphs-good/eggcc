#[cfg(test)]
use crate::{
    ast::*,
    egglog_test,
    interpreter::{Pointer, Value},
    schema::{RcExpr, Type},
};

#[cfg(test)]
fn type_test(inp: RcExpr, expected_ty: Type, arg: Value, expected_val: Value) -> crate::Result {
    let build = format!("{inp}");
    let check = format!("(check (HasType {inp} {expected_ty}))");
    egglog_test(
        &build,
        &check,
        vec![inp.to_program(emptyt(), expected_ty)],
        arg,
        expected_val,
    )
}

#[cfg(test)]
fn type_error_test(inp: RcExpr) {
    let _ = egglog_test(&format!("{inp}"), "", vec![], val_empty(), val_empty());
}

#[cfg(test)]
fn _debug(inp: RcExpr, after: &str) -> crate::Result {
    egglog_test(&format!("{inp}"), after, vec![], val_empty(), val_empty())
}

#[test]
fn primitives() -> crate::Result {
    type_test(int(3), intt(), val_int(0), val_int(3))?;
    type_test(int(12), intt(), val_int(0), val_int(12))?;
    type_test(ttrue(), boolt(), val_int(0), val_bool(true))?;
    type_test(tfalse(), boolt(), val_int(0), val_bool(false))?;
    type_test(empty(), emptyt(), val_int(0), val_empty())
}

#[test]
fn uops() -> crate::Result {
    let m = int(3);
    let x = ttrue();
    let y = tfalse();
    type_test(not(x), boolt(), val_int(0), val_bool(false))?;
    type_test(not(y), boolt(), val_int(0), val_bool(true))?;
    type_test(tprint(m), emptyt(), val_int(0), val_empty())
}

#[test]
#[should_panic]
fn not_error() {
    type_error_test(not(int(4)));
}

#[test]
#[should_panic]
fn load_error() {
    type_error_test(load(int(4)));
}

#[test]
fn bops() -> crate::Result {
    let m = int(3);
    let n = int(12);
    type_test(add(m.clone(), n.clone()), intt(), val_int(0), val_int(15))?;
    type_test(sub(m.clone(), n.clone()), intt(), val_int(0), val_int(-9))?;
    type_test(
        mul(
            add(m.clone(), m.clone()),
            sub(add(n.clone(), n.clone()), m.clone()),
        ),
        intt(),
        val_int(0),
        val_int(126),
    )
}

#[test]
#[should_panic]
fn add_error() {
    type_error_test(add(int(4), ttrue()));
}

#[test]
#[should_panic]
fn sub_error() {
    type_error_test(sub(tfalse(), ttrue()));
}

#[test]
#[should_panic]
fn mul_error() {
    type_error_test(mul(less_than(int(4), int(5)), int(3)));
}

#[test]
#[should_panic]
fn less_than_error() {
    type_error_test(less_than(less_than(int(4), int(5)), int(3)));
}

#[test]
#[should_panic]
fn and_error() {
    type_error_test(and(ttrue(), and(tfalse(), int(2))));
}

#[test]
#[should_panic]
fn or_error() {
    type_error_test(or(tfalse(), int(2)));
}

#[test]
fn pointers() -> crate::Result {
    let ptr = alloc(int(12), intt());
    type_test(
        ptr.clone(),
        pointert(intt()),
        val_int(0),
        Value::Ptr(Pointer::new(0, 12, 0)),
    )?;
    type_test(
        write(ptr.clone(), int(1)),
        emptyt(),
        val_int(0),
        val_empty(),
    )?;
    type_test(
        ptradd(alloc(int(1), boolt()), add(int(1), int(2))),
        pointert(boolt()),
        val_int(0),
        Value::Ptr(Pointer::new(0, 1, 3)),
    )
}

#[test]
#[should_panic]
fn pointer_write_error() {
    let ptr = alloc(int(12), intt());
    type_error_test(write(ptr.clone(), ttrue()));
}

#[test]
#[should_panic]
fn pointer_type_error() {
    type_error_test(alloc(less_than(int(1), int(2)), boolt()));
}

#[test]
fn tuple() -> crate::Result {
    type_test(
        single(int(30)),
        tuplet_vec(vec![intt()]),
        val_int(0),
        val_vec(vec![val_int(30)]),
    )?;

    type_test(
        concat_par(single(int(20)), single(ttrue())),
        tuplet_vec(vec![intt(), boolt()]),
        val_int(0),
        val_vec(vec![val_int(20), val_bool(true)]),
    )
}

#[test]
fn tuple_get() -> crate::Result {
    let t = concat_par(single(int(2)), concat_par(single(ttrue()), single(int(4))));
    type_test(get(t.clone(), 0), intt(), val_int(0), val_int(2))?;
    type_test(get(t.clone(), 1), boolt(), val_int(0), val_bool(true))?;
    type_test(get(t, 2), intt(), val_int(0), val_int(4))?;
    let t2 = concat_seq(
        single(tfalse()),
        single(add(get(single(int(2)), 0), int(1))),
    );
    type_test(get(t2, 0), boolt(), val_int(0), val_bool(false))
}

#[test]
fn ifs() -> crate::Result {
    type_test(tif(ttrue(), int(1), int(2)), intt(), val_int(0), val_int(1))?;

    type_test(
        tif(
            less_than(int(2), int(3)),
            and(ttrue(), tfalse()),
            or(less_than(int(3), int(4)), ttrue()),
        ),
        boolt(),
        val_int(0),
        val_bool(false),
    )
}

#[test]
#[should_panic]
fn if_pred() {
    type_error_test(tif(int(1), int(2), int(3)));
}

#[test]
#[should_panic]
fn if_branches() {
    type_error_test(tif(ttrue(), int(2), tfalse()));
}

#[test]
fn switches() -> crate::Result {
    type_test(
        switch_vec(int(1), vec![int(0), int(21)]),
        intt(),
        val_int(0),
        val_int(21),
    )?;
    type_test(
        switch_vec(int(0), vec![ttrue()]),
        boolt(),
        val_int(0),
        val_bool(true),
    )?;
    type_test(
        switch_vec(int(2), vec![int(1), int(2), int(3), int(4)]),
        intt(),
        val_int(0),
        val_int(3),
    )
}

#[test]
#[should_panic]
fn switch_pred() {
    type_error_test(switch_vec(ttrue(), vec![int(1), int(2)]));
}

#[test]
#[should_panic]
fn switch_branches() {
    type_error_test(switch_vec(int(1), vec![ttrue(), int(1)]));
}

#[test]
fn lets() -> crate::Result {
    let inp = tlet(int(4), add(arg(), arg()));
    type_test(inp, intt(), val_int(0), val_int(8))
}

#[test]
#[should_panic]
fn let_type_error() {
    type_error_test(tlet(int(1), and(bool_arg(), ttrue())));
}

#[test]
fn loops() -> crate::Result {
    let l1 = dowhile(single(int(1)), concat_seq(single(tfalse()), single(int(3))));
    type_test(
        l1,
        tuplet_vec(vec![intt()]),
        val_int(0),
        val_vec(vec![val_int(3)]),
    )?;

    let l15 = dowhile(
        single(int(1)),
        concat_seq(
            single(tfalse()),
            single(add(get(arg(tuplet_vec(vec![intt()])), 0), int(1))),
        ),
    );
    type_test(
        l15,
        tuplet_vec(vec![intt()]),
        val_int(0),
        val_vec(vec![val_int(2)]),
    )?;

    // while x < 4, x++
    let pred = single(less_than(get(arg(tuplet_vec(vec![intt()])), 0), int(4)));
    let body = single(add(get(arg(tuplet_vec(vec![intt()])), 0), int(1)));
    let l2 = dowhile(single(int(1)), concat_seq(pred, body));
    type_test(
        l2,
        tuplet_vec(vec![intt()]),
        val_int(0),
        val_vec(vec![val_int(5)]),
    )?;

    // x = 1, y = 2
    // do (x = x + 1, y = x * 2)
    // while (x < 5)
    let l2 = dowhile(
        concat_par(single(int(1)), single(int(2))),
        concat_par(
            single(less_than(
                get(arg(tuplet_vec(vec![intt(), intt()])), 0),
                int(5),
            )),
            concat_par(
                single(add(get(arg(tuplet_vec(vec![intt(), intt()])), 0), int(1))),
                single(mul(get(arg(tuplet_vec(vec![intt(), intt()])), 0), int(2))),
            ),
        ),
    );

    type_test(
        l2,
        tuplet_vec(vec![intt(), intt()]),
        val_int(0),
        val_vec(vec![val_int(6), val_int(10)]),
    )
}

#[test]
#[should_panic]
fn loop_input_error() {
    // input is not a tuple
    type_error_test(dowhile(int(4), concat_par(single(ttrue()), single(int(5)))))
}

#[test]
#[should_panic]
fn loop_predbody_error() {
    // pred-body is not a tuple
    type_error_test(dowhile(single(int(4)), ttrue()))
}

#[test]
#[should_panic]
fn loop_pred_error() {
    // pred is not a bool
    type_error_test(dowhile(
        single(int(1)),
        concat_par(single(int(2)), single(int(3))),
    ))
}

#[test]
#[should_panic]
fn loop_inputs_outputs_error1() {
    // input is bool, output is int
    type_error_test(dowhile(
        single(ttrue()),
        concat_par(single(tfalse()), single(int(2))),
    ))
}

#[test]
#[should_panic]
fn loop_inputs_outputs_error2() {
    // input is (int, bool), output is (int)
    type_error_test(dowhile(
        concat_seq(single(int(2)), single(ttrue())),
        concat_par(single(tfalse()), single(int(2))),
    ))
}

#[test]
fn funcs_and_calls() -> crate::Result {
    let f = function("f", intt(), intt(), add(arg(intt()), int(2)));
    let c = call("f", int(4));
    egglog_test(
        &format!("{f}{c}"),
        &format!("(check (HasType {c} (Base (IntT))))"),
        vec![f.to_program(intt(), intt())],
        val_int(4),
        val_int(6),
    )
}
