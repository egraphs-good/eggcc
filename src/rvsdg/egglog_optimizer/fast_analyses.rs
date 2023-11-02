use super::BRIL_OPS;

fn reify_vec_rules() -> Vec<String> {
    let mut res = vec![];
    for (vectype, ctor, eltype) in &[
        ("VecOperand", "VO", "Operand"),
        ("VecVecOperand", "VVO", "VecOperand"),
    ] {
        // Reify vec-get to be able to be able to match on vec contents
        res.push(format!(
            "
          (function {vectype}-get ({vectype} i64) {eltype})
          (rule (({ctor} x) (> (vec-length x) 0))
                ((union ({vectype}-get ({ctor} x) 0) (vec-get x 0)))
                :ruleset fast-analyses)
          (rule (({vectype}-get ({ctor} x) j)
                 (= i (+ j 1)) (< i (vec-length x)))
                ((union ({vectype}-get ({ctor} x) i) (vec-get x i)))
                :ruleset fast-analyses)"
        ));
        // Reify vec-length
        res.push(format!(
            "
            (function {vectype}-length ({vectype}) i64)
            (rule (({ctor} x))
                  ((set ({vectype}-length ({ctor} x)) (vec-length x)))
                  :ruleset fast-analyses)
        "
        ));
    }
    res
}

fn is_pure_rules() -> Vec<String> {
    let mut res: Vec<String> = vec!["
        (relation Expr-is-pure (Expr))
        (relation Operand-is-pure (Operand))
        (relation Body-is-pure (Body))
        (relation VecOperand-is-pure (VecOperand))
        (function VecOperand-pure-prefix (VecOperand) i64 :merge (max old new))
        (relation VecVecOperand-is-pure (VecVecOperand))
        (function VecVecOperand-pure-prefix (VecVecOperand) i64 :merge (max old new))
        (relation Function-is-pure (Function))
    "
    .into()];

    // Expr-is-pure
    res.push(
        "
        (rule ((= f (Const ty ops lit)))
              ((Expr-is-pure f))
              :ruleset fast-analyses)

        (rule ((= f (Call ty name args n-outs))
               (Function-is-pure (Func name input-types output-types body)))
              ((Expr-is-pure f))
              :ruleset fast-analyses)
    "
        .into(),
    );
    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
                (rule ((= f ({op} type e1 e2))
                       (Operand-is-pure e1)
                       (Operand-is-pure e2))
                      ((Expr-is-pure f))
                      :ruleset fast-analyses)
                "
            )),
            _ => unimplemented!(),
        };
    }

    // Operand-is-pure
    res.push(
        "
        (rule ((= f (Arg x)))
              ((Operand-is-pure f))
              :ruleset fast-analyses)
        (rule ((= f (Node body))
               (Body-is-pure body))
              ((Operand-is-pure f))
              :ruleset fast-analyses)
        (rule ((= f (Project i body))
               (Body-is-pure body))
              ((Operand-is-pure f))
              :ruleset fast-analyses)
    "
        .into(),
    );

    // Body-is-pure
    res.push(
        "
        (rule ((= f (PureOp e))
               (Expr-is-pure e))
              ((Body-is-pure f))
              :ruleset fast-analyses)
        (rule ((= f (Gamma pred inputs outputs))
               (Operand-is-pure pred)
               (VecOperand-is-pure inputs)
               (VecVecOperand-is-pure outputs))
              ((Body-is-pure f))
              :ruleset fast-analyses)
        (rule ((= f (Theta pred inputs outputs))
               (Operand-is-pure pred)
               (VecOperand-is-pure inputs)
               (VecOperand-is-pure outputs))
              ((Body-is-pure f))
              :ruleset fast-analyses)
        (rule ((= f (OperandGroup vec))
               (VecOperand-is-pure vec))
              ((Body-is-pure f))
              :ruleset fast-analyses)
    "
        .into(),
    );

    // {VecOperand,VecVecOperand}-is-pure
    for (vectype, ctor, eltype) in [
        ("VecOperand", "VO", "Operand"),
        ("VecVecOperand", "VVO", "VecOperand"),
    ] {
        res.push(format!(
            "
            (rule ((= f ({ctor} vec)))
                  ((set ({vectype}-pure-prefix f) 0))
                  :ruleset fast-analyses)
            (rule ((= i ({vectype}-pure-prefix f))
                   (< i ({vectype}-length f))
                   ({eltype}-is-pure ({vectype}-get f i)))
                  ((set ({vectype}-pure-prefix f) (+ i 1)))
                  :ruleset fast-analyses)
            (rule ((= ({vectype}-length f) ({vectype}-pure-prefix f)))
                  (({vectype}-is-pure f))
                  :ruleset fast-analyses)
        "
        ));
    }

    // Function-is-pure
    res.push(
        "
        (rule ((= f (Func name input-types output-types body))
               (VecOperand-is-pure body))
              ((Function-is-pure f))
              :ruleset fast-analyses)
    "
        .into(),
    );

    res
}

pub(crate) fn all_rules() -> String {
    let mut res = vec!["(ruleset fast-analyses)".to_string()];
    res.extend(reify_vec_rules());
    res.extend(is_pure_rules());
    res.join("\n")
}
