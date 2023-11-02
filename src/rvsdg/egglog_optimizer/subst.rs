use super::BRIL_OPS;

/// "subst-beneath" rules support replacing a specific {Expr, Body, Operand,
///  VecOperand, VecVecOperand} with another.
///
/// - The key relations are (relation can-subst-TYPE-beneath (Body TYPE TYPE)),
///   where TYPE is one of {Expr, Body, Operand, VecOperand, VecVecOperand}.
/// - Add (can-subst-TYPE-beneath above from to) if it is sound to replace zero
///   or more occurrences of `from` with `to`, strictly within `above`.
/// - Then, saturate the subst ruleset for the appropriate unions to be made.
///
/// Some intuition behind what is actually happening:
///   If it's safe to replace `(badd a b)` with `to` in some context, then it's
///   also safe to replace `(other-op (badd a b) c)` with `(other-op to c)` in
///   the same context.
///
///   In this way, `can-subst-*-beneath` learns more possible replacements
///   "bottom-up", then, when we reach the top of a context (a body), we union
///   only at the top.
///
/// See [src/rvsdg/tests.rs] for examples.
///
/// rtjoa: We could parameterize on `above`'s type (currently, always Body) to
/// support substituting under a Function as well
fn subst_beneath_rules() -> Vec<String> {
    let mut res = vec!["
        (relation can-subst-Expr-beneath (Body Expr Expr))
        (relation can-subst-Operand-beneath (Body Operand Operand))
        (relation can-subst-Body-beneath (Body Body Body))
        (relation can-subst-VecVecOperand-beneath (Body VecVecOperand VecVecOperand))
        (relation can-subst-VecOperand-beneath (Body VecOperand VecOperand))

        ;; Base case 'do the substitution' rules
        (rule ((can-subst-Operand-beneath above from to)
               (= above     (Theta from inputs outputs)))
              ((union above (Theta to   inputs outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= above     (Theta pred inputs from)))
              ((union above (Theta pred inputs to)))
              :ruleset subst)
        (rule ((can-subst-Operand-beneath above pred-from pred-to)
               (can-subst-VecOperand-beneath above outputs-from outputs-to)
               (= above     (Theta pred-from inputs outputs-from)))
              ((union above (Theta pred-from inputs outputs-to)))
              :ruleset subst)
        (rule ((can-subst-VecVecOperand-beneath above from to)
               (= above     (Gamma pred inputs from)))
              ((union above (Gamma pred inputs to)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= above     (OperandGroup from)))
              ((union above (OperandGroup to)))
              :ruleset subst)

        ;; Learn can-subst-Operand-beneath
        (rule ((can-subst-Body-beneath above from to)
               (= new-from (Node from)))
              ((can-subst-Operand-beneath above new-from (Node to)))
              :ruleset subst)
        (rule ((can-subst-Body-beneath above from to)
               (= new-from (Project i from)))
              ((can-subst-Operand-beneath above new-from (Project i to)))
              :ruleset subst)

        ;; Learn can-subst-body-beneath
        (rule ((can-subst-Expr-beneath above from to)
               (= new-from (PureOp from)))
              ((can-subst-Body-beneath above new-from (PureOp to)))
              :ruleset subst)
        ;; Propagates up same context (Gamma: pred & inputs, Theta: inputs)
        ;; rtjoa: Is it sound to propagate up outputs if we renumber args?
        (rule ((can-subst-Operand-beneath above from to)
               (= new-from (Gamma from inputs outputs)))
              ((can-subst-Body-beneath above new-from (Gamma to inputs outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (Gamma pred from outputs)))
              ((can-subst-Body-beneath above new-from (Gamma pred to outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (Theta pred from outputs)))
              ((can-subst-Body-beneath above new-from (Theta pred to outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (OperandGroup from)))
              ((can-subst-Body-beneath above new-from (OperandGroup to)))
              :ruleset subst)
        "
    .into()];

    // Learn can-subst-Expr-beneath
    res.push(
        "
      (rule ((can-subst-VecOperand-beneath above from to)
              (= new-from (Call ty f from n-outs)))
             ((can-subst-Expr-beneath above new-from (Call ty f to n-outs)))
            :ruleset subst)"
            .into(),
    );
    for bril_op in BRIL_OPS {
        let op = bril_op.op;

        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
              (rule ((can-subst-Operand-beneath above from to)
                      (= new-from ({op} type from e2)))
                     ((can-subst-Expr-beneath above new-from ({op} type to e2)))
                    :ruleset subst)
              (rule ((can-subst-Operand-beneath above from to)
                      (= new-from ({op} type e1 from)))
                     ((can-subst-Expr-beneath above new-from ({op} type e1 to)))
                    :ruleset subst)
                     ",
            )),
            _ => unimplemented!(),
        };
    }

    // Learn can-subst-{VecOperand, VecVecOperand}-beneath
    for (vectype, ctor, eltype) in &[
        ("VecOperand", "VO", "Operand"),
        ("VecVecOperand", "VVO", "VecOperand"),
    ] {
        // Propagate info bottom-up
        res.push(format!(
            "
          (rule ((can-subst-{eltype}-beneath above from to)
                 (= from ({vectype}-get ({ctor} vec) i)))
                ((can-subst-{vectype}-beneath
                    above
                    ({ctor} vec)
                    ({ctor} (vec-set vec i to))))
                :ruleset subst)"
        ));
    }

    res
}

pub(crate) fn subst_rules() -> String {
    let mut res = vec![include_str!("subst.egg").to_string()];
    res.extend(subst_beneath_rules());
    res.join("\n")
}
