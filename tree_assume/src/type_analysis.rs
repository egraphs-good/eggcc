#[cfg(test)]
use crate::{interpreter::Value, schema::Constant};

#[test]
fn primitive_types() -> Result<(), egglog::Error> {
    let build = "
        (let m (Const (Int 3)))
        (let n (Const (Int 12)))
        (let x (Const (Bool true)))
        (let y (Const (Bool false)))
        (let t (Empty))
        ";
    let check = format!(
        "
    (check (HasType n (Base (IntT))))
    (check (HasType m (Base (IntT))))
    (check (HasType x (Base (BoolT))))
    (check (HasType y (Base (BoolT))))
    (check (HasType t (TupleT (TNil))))
    "
    );
    let progs = vec![];
    let input = Value::Const(Constant::Int(0));
    let expected = Value::Const(Constant::Int(0));
    crate::egglog_test(build, &check, progs, input, expected)
}

#[test]
fn uops() -> Result<(), egglog::Error> {
    let build = "
        (let m (Const (Int 3)))
        (let n (Const (Int 12)))
        (let x (Const (Bool true)))
        (let y (Const (Bool false)))
        (let e1 (Uop (Not) x))
        (let e2 (Uop (Not) y))
        (let e3 (Uop (Print) m))
        ";
    let check = format!(
        "
    (check (HasType e1 (Base (BoolT))))
    (check (HasType e2 (Base (BoolT))))
    (check (HasType e3 (TupleT (TNil))))
    "
    );
    let progs = vec![];
    let input = Value::Const(Constant::Int(0));
    let expected = Value::Const(Constant::Int(0));
    crate::egglog_test(build, &check, progs, input, expected)
}
