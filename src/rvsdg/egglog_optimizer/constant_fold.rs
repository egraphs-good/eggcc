use super::BRIL_OPS;

pub(crate) fn constant_fold_egglog() -> String {
    let mut res = vec![include_str!("constant_fold.egg").to_string()];

    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        let egglog_op = bril_op.egglog_op;
        if bril_op.num_inputs() == 2 {
            res.push(format!(
                "(rewrite ({op} ty
                      (Node (PureOp (Const ty (const) (Num n1))))
                      (Node (PureOp (Const ty (const) (Num n2)))))
                    (Const ty (const) (Num ({egglog_op} n1 n2))))",
            ));
        }
    }

    res.join("\n")
}
