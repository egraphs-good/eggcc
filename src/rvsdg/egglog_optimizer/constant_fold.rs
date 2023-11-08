use bril_rs::Type;

use super::BRIL_OPS;

pub(crate) fn constant_fold_egglog() -> String {
    let mut res = vec![include_str!("constant_fold.egg").to_string()];

    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        let egglog_op = bril_op.egglog_op;

        match (bril_op.input_types.as_ref(), bril_op.output_type) {
            ([Some(Type::Int), Some(Type::Int)], Type::Int) => res.push(format!(
                "(rewrite ({op} output_type
                    (Node (PureOp (Const ty2 (const) (Num n1))))
                    (Node (PureOp (Const ty3 (const) (Num n2)))))
                  (Const output_type (const) (Num ({egglog_op} n1 n2))))",
            )),
            // egglog partial functions
            ([Some(Type::Int), Some(Type::Int)], Type::Bool) => res.push(format!(
                "(rewrite ({op} output_type
                  (Node (PureOp (Const ty2 (const) (Num n1))))
                  (Node (PureOp (Const ty3 (const) (Num n2)))))
                (Const output_type (const) (Bool ({egglog_op} n1 n2))))",
            )),
            ([Some(Type::Bool), Some(Type::Bool)], Type::Bool) => res.push(format!(
                "(rewrite ({op} output_type
                  (Node (PureOp (Const ty2 (const) (Bool n1))))
                  (Node (PureOp (Const ty3 (const) (Bool n2)))))
                (Const output_type (const) (Bool ({egglog_op} n1 n2))))",
            )),
            ([Some(Type::Bool), None], Type::Bool) => res.push(format!(
                "(rewrite ({op} output_type
                  (Node (PureOp (Const ty2 (const) (Bool n1)))))
                (Const output_type (const) (Bool ({egglog_op} n1))))",
            )),
            _ => unimplemented!(),
        };
    }

    res.join("\n")
}
