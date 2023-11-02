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

// Tracks what Exprs/Operands/Bodies a body contains in its context.
// Notably, if a Gamma/Theta contains X, then Args in X are bound by that
// Gamma/Theta's inputs.
fn region_contains_rules() -> Vec<String> {
    let mut res: Vec<String> = vec!["
        (relation Body-contains-Expr (Body Expr))
        (relation Body-contains-Operand (Body Operand))
        (relation Body-contains-Body (Body Body))
    "
    .into()];

    // Bodies contain their immediate children
    res.push(
        "
        (rule ((= f (PureOp e)))
              ((Body-contains-Expr f e))
              :ruleset fast-analyses)
        ; A Gamma only contains its outputs
        (rule ((= f (Gamma pred inputs outputs))
               (= outputs-i (VecVecOperand-get outputs i))
               (= x (VecOperand-get outputs-i j)))
              ((Body-contains-Operand f x))
              :ruleset fast-analyses)
        ; A Theta contains its pred and outputs
        (rule ((= f (Theta pred inputs outputs)))
              ((Body-contains-Operand f pred))
              :ruleset fast-analyses)
        (rule ((= f (Theta pred inputs outputs))
               (= x (VecOperand-get outputs i)))
              ((Body-contains-Operand f x))
              :ruleset fast-analyses)
        (rule ((= f (OperandGroup vec))
               (= x (VecOperand-get vec i)))
              ((Body-contains-Operand f x))
              :ruleset fast-analyses)
    "
        .into(),
    );

    // Transitivity - Body
    res.push(
        "
        (rule ((Body-contains-Body f (PureOp e)))
              ((Body-contains-Expr f e))
              :ruleset fast-analyses)
        ; A Gamma's pred and inputs are in the outer context
        (rule ((Body-contains-Body f (Gamma pred inputs outputs)))
              ((Body-contains-Operand f pred))
              :ruleset fast-analyses)
        (rule ((Body-contains-Body f (Gamma pred inputs outputs))
               (= x (VecOperand-get inputs i)))
              ((Body-contains-Operand f x))
              :ruleset fast-analyses)
        ; A Theta's inputs are in the outer context
        (rule ((Body-contains-Body f (Theta pred inputs outputs))
                (= x (VecOperand-get inputs i)))
              ((Body-contains-Operand f x))
              :ruleset fast-analyses)
        (rule ((Body-contains-Body f (OperandGroup vec))
               (= x (VecOperand-get vec i)))
              ((Body-contains-Operand f x))
              :ruleset fast-analyses)
    "
        .into(),
    );

    // Transitivity - Expr
    res.push(
        "
        (rule ((Body-contains-Expr f (Call ty name args n-outs))
               (= x (VecOperand-get args i)))
              ((Body-contains-Operand f x))
              :ruleset fast-analyses)
        (rule ((Body-contains-Expr f (PRINT e1 e2)))
              ((Body-contains-Operand f e1)
               (Body-contains-Operand f e2))
              :ruleset fast-analyses)
    "
        .into(),
    );
    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
                (rule ((Body-contains-Expr f ({op} type e1 e2)))
                      ((Body-contains-Operand f e1)
                       (Body-contains-Operand f e2))
                      :ruleset fast-analyses)
                "
            )),
            _ => unimplemented!(),
        };
    }

    // Transitivity - Operand
    res.push(
        "
        (rule ((Body-contains-Operand f (Node body)))
              ((Body-contains-Body f body))
              :ruleset fast-analyses)
        (rule ((Body-contains-Operand f (Project i body)))
              ((Body-contains-Body f body))
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
    res.extend(region_contains_rules());
    res.join("\n")
}
