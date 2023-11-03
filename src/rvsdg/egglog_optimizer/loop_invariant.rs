fn inv_binary_ops(bop: String) -> String {
    format!(
        "(rule ((find_inv_expr theta ({bop} ty a b)))
            ((find_inv_operand theta a) (find_inv_operand theta b)) :ruleset fast-analyses)

(rule ((= true (is_inv_operand body a))
            (= true (is_inv_operand body b))
            (find_inv_expr body expr)
            (= expr ({bop} ty a b)))
        ((set (is_inv_expr body expr) true)) :ruleset fast-analyses)
    "
    )
}

pub(crate) fn loop_invariant_detection() -> String {
    let mut res = vec![include_str!("loop_invariant.egg").to_string()];

    for bop in &[
        "badd", "bsub", "bmul", "bfmul", "bdiv", "beq", "blt", "bgt", "ble", "bge", "bnot", "band",
        "bor",
    ] {
        res.push(inv_binary_ops(bop.to_string()));
    }

    res.join("\n")
}
