#[cfg(test)]
use crate::{
    ast::*,
    egglog_test,
    interpreter::Value,
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
fn type_error_test(inp: RcExpr) -> crate::Result {
    egglog_test(&format!("{inp}"), "", vec![], val_empty(), val_empty())
}

#[cfg(test)]
fn _debug(before: &str, after: &str) -> crate::Result {
    egglog_test(before, after, vec![], val_empty(), val_empty())
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
    let _ = type_error_test(not(int(4)));
}

#[test]
#[should_panic]
fn load_error() {
    let _ = type_error_test(load(int(4)));
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
    let _ = type_error_test(add(int(4), ttrue()));
}

#[test]
#[should_panic]
fn sub_error() {
    let _ = type_error_test(sub(tfalse(), ttrue()));
}

#[test]
#[should_panic]
fn mul_error() {
    let _ = type_error_test(mul(less_than(int(4), int(5)), int(3)));
}

#[test]
#[should_panic]
fn less_than_error() {
    let _ = type_error_test(less_than(less_than(int(4), int(5)), int(3)));
}

#[test]
#[should_panic]
fn and_error() {
    let _ = type_error_test(and(ttrue(), and(tfalse(), int(2))));
}

#[test]
#[should_panic]
fn or_error() {
    let _ = type_error_test(or(tfalse(), int(2)));
}
