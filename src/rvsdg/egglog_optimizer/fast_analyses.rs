use super::BRIL_OPS;

fn reify_vec_rules() -> Vec<String> {
    let mut res = vec![];
    for (vectype, ctor, eltype) in &[
        ("VecOperand", "VO", "Operand"),
        ("VecOperandCtx", "VOC", "Operand"),
        ("VecVecOperandCtx", "VVO", "VecOperandCtx"),
    ] {
        // Reify vec-get to be able to be able to match on vec contents
        res.push(format!(
            "
          (function {vectype}-get ({vectype} i64) {eltype} :unextractable)
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
        (relation VecOperandCtx-is-pure (VecOperandCtx))
        (function VecOperandCtx-pure-prefix (VecOperandCtx) i64 :merge (max old new))
        (relation VecVecOperandCtx-is-pure (VecVecOperandCtx))
        (function VecVecOperandCtx-pure-prefix (VecVecOperandCtx) i64 :merge (max old new))
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
            [Some(_), None] => res.push(format!(
                "
                (rule ((= f ({op} type e1))
                       (Operand-is-pure e1))
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
               (VecVecOperandCtx-is-pure outputs))
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

    // {VecOperand,VecVecOperandCtx}-is-pure
    for (vectype, ctor, eltype) in [
        ("VecOperand", "VO", "Operand"),
        ("VecOperandCtx", "VOC", "Operand"),
        ("VecVecOperandCtx", "VVO", "VecOperandCtx"),
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

// Tracks what Exprs/Operands/Bodies an operand contains in its context,
// similar to Body-contains-*.
// A very important difference is that it does not look into the inputs/outputs of Theta
// and outputs of Gamma. Instead, it only looks at their inputs.
//
// Because there are so many operand, the user of this analysis need
// to explicitly demand annotate operands they are interested in.
fn operand_contains_rules() -> Vec<String> {
    let mut res: Vec<String> = vec!["
        (relation Operand-contains-Expr (Operand Expr))
        (relation Operand-contains-Operand (Operand Operand))
        (relation Operand-contains-Body (Operand Body))
        (relation Operand-contains-demand (Operand))
    "
    .into()];

    // Body and operand rules
    res.push(
        "
        (rule ((Operand-contains-demand f))
              ((Operand-contains-Operand f f))
              :ruleset fast-analyses)
              
        (rule ((Operand-contains-Operand f c)
               (= c (Node body)))
              ((Operand-contains-Body f body))
                :ruleset fast-analyses)
        (rule ((Operand-contains-Operand f c)
               (= c (Project i body)))
              ((Operand-contains-Body f body))
              :ruleset fast-analyses)

        (rule ((Operand-contains-Body f (PureOp e)))
              ((Operand-contains-Expr f e))
              :ruleset fast-analyses)
        ;; A Gamma contains both its predicate and inputs
        (rule ((Operand-contains-Body f (Gamma pred inputs outputs)))
              ((Operand-contains-Operand f pred))
              :ruleset fast-analyses)
        (rule ((Operand-contains-Body f (Gamma pred inputs outputs))
               (= x (VecOperand-get inputs any)))
              ((Operand-contains-Operand f x))
              :ruleset fast-analyses)
        ;; A Theta contains only its predicate
        (rule ((Operand-contains-Body f (Theta pred inputs outputs)))
              ((Operand-contains-Operand f pred))
              :ruleset fast-analyses)
        ;; OperandGroup contains its operands
        (rule ((Operand-contains-Body f (OperandGroup vec))
               (= x (VecOperand-get vec any)))
              ((Operand-contains-Operand f x))
              :ruleset fast-analyses)
    "
        .into(),
    );

    // Expr rules
    res.push(
        "
        (rule ((Operand-contains-Expr f (Call ty name args n-outs))
               (= x (VecOperand-get args any)))
              ((Operand-contains-Operand f x))
              :ruleset fast-analyses)
        (rule ((Operand-contains-Expr f (PRINT e1 e2)))
              ((Operand-contains-Operand f e1)
               (Operand-contains-Operand f e2))
              :ruleset fast-analyses)
    "
        .into(),
    );
    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
                (rule ((Operand-contains-Expr f ({op} type e1 e2)))
                      ((Operand-contains-Operand f e1)
                       (Operand-contains-Operand f e2))
                      :ruleset fast-analyses)
                "
            )),
            [Some(_), None] => res.push(format!(
                "
                (rule ((Operand-contains-Expr f ({op} type e1)))
                      ((Operand-contains-Operand f e1))
                      :ruleset fast-analyses)
                "
            )),
            _ => unimplemented!(),
        };
    }

    res
}

// Tracks what Exprs/Operands/Bodies a body contains in its context, in its
// ith output (-1 indicates it's contained in a Theta predicate).
// Notably, if a Gamma/Theta contains X, then Args in X are bound by that
// Gamma/Theta's inputs.
fn region_contains_rules() -> Vec<String> {
    let mut res: Vec<String> = vec!["
        (relation Body-contains-Expr (Body i64 Expr))
        (relation Body-contains-Operand (Body i64 Operand))
        (relation Body-contains-Body (Body i64 Body))
    "
    .into()];

    // Bodies contain their immediate children
    res.push(
        "
        (rule ((= f (PureOp e)))
              ((Body-contains-Expr f 0 e))
              :ruleset fast-analyses)
        ; A Gamma only contains its outputs
        (rule ((= f (Gamma pred inputs outputs))
               (= outputs-i (VecVecOperandCtx-get outputs i))
               (= x (VecOperandCtx-get outputs-i j)))
              ((Body-contains-Operand f i x))
              :ruleset fast-analyses)
        ; A Theta contains its pred and outputs
        (rule ((= f (Theta pred inputs outputs)))
              ((Body-contains-Operand f -1 pred))
              :ruleset fast-analyses)
        (rule ((= f (Theta pred inputs outputs))
               (= x (VecOperand-get outputs i)))
              ((Body-contains-Operand f i x))
              :ruleset fast-analyses)
        (rule ((= f (OperandGroup vec))
               (= x (VecOperand-get vec i)))
              ((Body-contains-Operand f i x))
              :ruleset fast-analyses)
    "
        .into(),
    );

    // Transitivity - Body
    res.push(
        "
        (rule ((Body-contains-Body f i (PureOp e)))
              ((Body-contains-Expr f i e))
              :ruleset fast-analyses)
        ; A Gamma's pred and inputs are in the outer context
        (rule ((Body-contains-Body f i (Gamma pred inputs outputs)))
              ((Body-contains-Operand f i pred))
              :ruleset fast-analyses)
        (rule ((Body-contains-Body f i (Gamma pred inputs outputs))
               (= x (VecOperand-get inputs any)))
              ((Body-contains-Operand f i x))
              :ruleset fast-analyses)
        ; A Theta's inputs are in the outer context
        (rule ((Body-contains-Body f i (Theta pred inputs outputs))
               (= x (VecOperand-get inputs any)))
              ((Body-contains-Operand f i x))
              :ruleset fast-analyses)
        (rule ((Body-contains-Body f i (OperandGroup vec))
               (= x (VecOperand-get vec any)))
              ((Body-contains-Operand f i x))
              :ruleset fast-analyses)
    "
        .into(),
    );

    // Transitivity - Expr
    res.push(
        "
        (rule ((Body-contains-Expr f i (Call ty name args n-outs))
               (= x (VecOperand-get args any)))
              ((Body-contains-Operand f i x))
              :ruleset fast-analyses)
        (rule ((Body-contains-Expr f i (PRINT e1 e2)))
              ((Body-contains-Operand f i e1)
               (Body-contains-Operand f i e2))
              :ruleset fast-analyses)
    "
        .into(),
    );
    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
                (rule ((Body-contains-Expr f i ({op} type e1 e2)))
                      ((Body-contains-Operand f i e1)
                       (Body-contains-Operand f i e2))
                      :ruleset fast-analyses)
                "
            )),
            [Some(_), None] => res.push(format!(
                "
                (rule ((Body-contains-Expr f i ({op} type e1)))
                      ((Body-contains-Operand f i e1))
                      :ruleset fast-analyses)
                "
            )),
            _ => unimplemented!(),
        };
    }

    // Transitivity - Operand
    res.push(
        "
        (rule ((Body-contains-Operand f i (Node body)))
              ((Body-contains-Body f i body))
              :ruleset fast-analyses)
        (rule ((Body-contains-Operand f i (Project i body)))
              ((Body-contains-Body f i body))
              :ruleset fast-analyses)
        (rule ((Body-contains-Body f i (PureOp e))
               (Body-contains-Body (PureOp e) j body))
              ((Body-contains-Body f i body)) :ruleset fast-analyses)
    "
        .into(),
    );

    res
}

pub(crate) fn vo_conversion_rules() -> String {
    "
(function VO-to-VOC (VecOperand) VecOperandCtx :unextractable)
(function VOC-to-VO (VecOperandCtx) VecOperand :unextractable)

(rewrite (VO-to-VOC (VO vo)) (VOC vo) :ruleset fast-analyses)
(rewrite (VOC-to-VO (VOC voc)) (VO voc) :ruleset fast-analyses)
    "
    .to_string()
}

pub(crate) fn all_rules() -> String {
    let mut res = vec!["(ruleset fast-analyses)".to_string()];
    res.extend(reify_vec_rules());
    res.extend(is_pure_rules());
    res.extend(region_contains_rules());
    res.push(vo_conversion_rules());
    res.extend(operand_contains_rules());
    res.push(arg_used_rules());
    res.join("\n")
}

fn arg_used_rules() -> String {
    "
    ; Note that because the merge function is set-union 
    ;; instead of set-intersect (which is sound but requires a 
    ;; different set of rules for Node and Project),
    ;; we cannot handle cases like
    ;; if (...) x else x - x even accompanied with rules like x - x => 0

    (function arg-used-Operand (Operand) SetIntBase :merge (set-union old new))
    (rule ((Operand-contains-Operand operand (Arg arg)))
        ((set (arg-used-Operand operand) (set-of arg)))
        :ruleset fast-analyses)

    (function arg-used-VecOperandCtx (VecOperandCtx) SetIntBase :merge (set-union old new))
    (rule ((= operand (VecOperandCtx-get operands i))
        (= arg-set (arg-used-Operand operand)))
        ((set (arg-used-VecOperandCtx operands) arg-set))
        :ruleset fast-analyses)

    ;; Right now, we should only fire this analysis on children operands of VecOperandCtx and VecOperand
    (rule ((= operand (VecOperandCtx-get operands i)))
        ((Operand-contains-demand operand))
        :ruleset fast-analyses)
    (rule ((= operand (VecOperand-get operands i)))
        ((Operand-contains-demand operand))
        :ruleset fast-analyses)
        ".into()
}
