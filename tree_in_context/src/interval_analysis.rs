#[cfg(test)]
use crate::{
  ast::*,
  egglog_test,
  interpreter::*,
  schema::*
};

#[cfg(test)]
fn int_interval_test(inp: RcExpr, expected_ty: Type, arg: Value, expected_val: Value, lo: i64, hi: i64) -> crate::Result {
    let with_arg_types = inp.clone().with_arg_types(emptyt(), expected_ty.clone());
    let check = format!("(check (ival {with_arg_types}) (IntI {lo} {hi}))");
    interval_test(with_arg_types, expected_ty, arg, expected_val, check)
}

#[cfg(test)]
fn bool_interval_test(inp: RcExpr, expected_ty: Type, arg: Value, expected_val: Value, lo: bool, hi: bool) -> crate::Result {
    let with_arg_types = inp.clone().with_arg_types(emptyt(), expected_ty.clone());
    let check = format!("(check (ival {with_arg_types}) (BoolI {lo} {hi}))");
    interval_test(with_arg_types, expected_ty, arg, expected_val, check)
}

#[cfg(test)]
fn interval_test(
    prog: RcExpr,
    expected_ty: Type,
    arg: Value,
    expected_val: Value,
    check: String

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
  int_interval_test(e, base(intt()), val_empty(), val_int(3), 3, 3)
}

#[test]
fn constant_interval_test2() -> crate::Result {
  let e = ttrue();
  bool_interval_test(e, base(boolt()), val_empty(), val_bool(true), true, true)
}