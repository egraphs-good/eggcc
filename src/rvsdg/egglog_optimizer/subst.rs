use super::BRIL_OPS;

/// "subst-beneath" rules support replacing a specific {Expr, Body, Operand,
///  VecOperand, VecVecOperand} with another.
///
/// - The key relations are (relation can-subst-TYPE-beneath (Context TYPE TYPE)),
///   where TYPE is one of {Expr, Body, Operand, VecOperand, VecVecOperand}.
/// - A context is a (GammaCtx inputs) or a (ThetaCtx inputs).
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
        (datatype Context
            (GammaCtx VecOperand)
            (ThetaCtx VecOperand))

        (relation can-subst-Expr-beneath (Context Expr Expr))
        (relation can-subst-Operand-beneath (Context Operand Operand))
        (relation can-subst-Body-beneath (Context Body Body))
        (relation can-subst-VecVecOperand-beneath (Context VecVecOperand VecVecOperand))
        (relation can-subst-VecOperand-beneath (Context VecOperand VecOperand))

        ;; Base case 'do the substitution' rules
        (rule ((can-subst-Operand-beneath above from to)
               (= above (ThetaCtx inputs))
               (= theta     (Theta from inputs outputs)))
              ((union theta (Theta to   inputs outputs)))
              :ruleset subst-beneath)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= above (ThetaCtx inputs))
               (= theta     (Theta pred inputs from)))
              ((union theta (Theta pred inputs to)))
              :ruleset subst-beneath)
        (rule ((can-subst-Operand-beneath above pred-from pred-to)
               (can-subst-VecOperand-beneath above outputs-from outputs-to)
               (= above (ThetaCtx inputs))
               (= theta     (Theta pred-from inputs outputs-from)))
              ((union theta (Theta pred-from inputs outputs-to)))
              :ruleset subst-beneath)
        (rule ((can-subst-VecVecOperand-beneath above from to)
               (= above (GammaCtx inputs))
               (= gamma     (Gamma pred inputs from)))
              ((union gamma (Gamma pred inputs to)))
              :ruleset subst-beneath)

        ;; Learn can-subst-Operand-beneath
        (rule ((can-subst-Body-beneath above from to)
               (= new-from (Node from)))
              ((can-subst-Operand-beneath above new-from (Node to)))
              :ruleset subst-beneath)
        (rule ((can-subst-Body-beneath above from to)
               (= new-from (Project i from)))
              ((can-subst-Operand-beneath above new-from (Project i to)))
              :ruleset subst-beneath)

        ;; Learn can-subst-body-beneath
        (rule ((can-subst-Expr-beneath above from to)
               (= new-from (PureOp from)))
              ((can-subst-Body-beneath above new-from (PureOp to)))
              :ruleset subst-beneath)
        ;; Propagates up same context (Gamma: pred & inputs, Theta: inputs)
        ;; rtjoa: Is it sound to propagate up outputs if we renumber args?
        (rule ((can-subst-Operand-beneath above from to)
               (= new-from (Gamma from inputs outputs)))
              ((can-subst-Body-beneath above new-from (Gamma to inputs outputs)))
              :ruleset subst-beneath)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (Gamma pred from outputs)))
              ((can-subst-Body-beneath above new-from (Gamma pred to outputs)))
              :ruleset subst-beneath)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (Theta pred from outputs)))
              ((can-subst-Body-beneath above new-from (Theta pred to outputs)))
              :ruleset subst-beneath)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (OperandGroup from)))
              ((can-subst-Body-beneath above new-from (OperandGroup to)))
              :ruleset subst-beneath)
        "
    .into()];

    // Learn can-subst-Expr-beneath
    res.push(
        "
      (rule ((can-subst-VecOperand-beneath above from to)
              (= new-from (Call ty f from n-outs)))
             ((can-subst-Expr-beneath above new-from (Call ty f to n-outs)))
            :ruleset subst-beneath)"
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
                    :ruleset subst-beneath)
              (rule ((can-subst-Operand-beneath above from to)
                      (= new-from ({op} type e1 from)))
                     ((can-subst-Expr-beneath above new-from ({op} type e1 to)))
                    :ruleset subst-beneath)
                     ",
            )),
            [Some(_), None] => res.push(format!(
                "
              (rule ((can-subst-Operand-beneath above from to)
                      (= new-from ({op} type from)))
                     ((can-subst-Expr-beneath above new-from ({op} type to)))
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
                :ruleset subst-beneath)"
        ));
    }

    res.push(
        "
        (rule ((can-subst-Operand-beneath above from to)
               (= new-from (PRINT from state)))
               ((can-subst-Expr-beneath above new-from (PRINT to state)))
               :ruleset subst-beneath)
        (rule ((can-subst-Operand-beneath above from to)
               (= new-from (PRINT op from)))
               ((can-subst-Expr-beneath above new-from (PRINT op to)))
               :ruleset subst-beneath)"
            .into(),
    );

    res
}

// Below, TYPE is one of {Expr, Operand, Body, VecOperand, VecVecOperand}

// Generate rules to replace args via some procedure. See below for examples.
//
// Will generate functions named func_name_format.replace("{}", TYPE),
// with parameter types [TYPE] ++ aux_param_types
//
// arg_rules are hardcoded rules for the Arg case of the function for Operand
fn functions_modifying_args(
    func_name_fmt: &str,
    aux_param_types: Vec<&str>,
    ruleset: &str,
    arg_rules: &str,
) -> Vec<String> {
    let mut res = vec![];

    // Define functions
    let aux_params_str = aux_param_types.join(" ");
    for ty in ["Expr", "Operand", "Body", "VecOperand", "VecVecOperand"] {
        let fname = func_name_fmt.replace("{}", ty);
        res.push(format!(
            "(function {fname} ({ty} {aux_params_str}) {ty} :cost 10000)",
        ));
    }
    let fname_expr = func_name_fmt.replace("{}", "Expr");
    let fname_operand = func_name_fmt.replace("{}", "Operand");
    let fname_body = func_name_fmt.replace("{}", "Body");
    let fname_vec_operand = func_name_fmt.replace("{}", "VecOperand");

    // Rules to compute on Expr
    let aux_args_str = (0..aux_param_types.len())
        .map(|i| format!("x{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
                (rewrite
                    ({fname_expr} ({op} ty a b) {aux_args_str})
                    ({op}
                        ty
                        ({fname_operand} a {aux_args_str})
                        ({fname_operand} b {aux_args_str}))
                    :ruleset {ruleset})
                     ",
            )),
            [Some(_), None] => res.push(format!(
                "
                (rewrite
                    ({fname_expr} ({op} ty a) {aux_args_str})
                    ({op}
                        ty
                        ({fname_operand} a {aux_args_str})
                    )
                    :ruleset {ruleset})
                     ",
            )),
            _ => unimplemented!(),
        };
    }
    res.push(format!(
        "
        (rewrite
            ({fname_expr} (Const ty ops lit) {aux_args_str})
            (Const ty ops lit)
            :ruleset {ruleset})
        (rewrite
            ({fname_expr} (Call ty f args n-outs) {aux_args_str})
            (Call ty f ({fname_vec_operand} args {aux_args_str}) n-outs)
            :ruleset {ruleset})
        (rewrite
            ({fname_expr} (PRINT a b) {aux_args_str})
            (PRINT ({fname_operand} a {aux_args_str}) ({fname_operand} b {aux_args_str}))
            :ruleset {ruleset})",
    ));

    // Rules to compute on Operand
    res.push(arg_rules.into());
    res.push(format!(
        "
        (rewrite
            ({fname_operand} (Node b) {aux_args_str})
            (Node ({fname_body} b {aux_args_str}))
            :ruleset {ruleset})
        (rewrite
            ({fname_operand} (Project i b) {aux_args_str})
            (Project i ({fname_body} b {aux_args_str}))
            :ruleset {ruleset})"
    ));

    // Rules to compute on Body
    res.push(format!(
        "
        (rewrite
            ({fname_body} (PureOp e) {aux_args_str})
            (PureOp ({fname_expr} e {aux_args_str}))
            :ruleset {ruleset})
        ;; Don't cross regions, so so we shift into the inputs but not outputs
        ;; A Gamma's pred is on the outside, so it's affected, but not a Theta's
        (rewrite
            ({fname_body} (Gamma pred inputs outputs) {aux_args_str})
            (Gamma
                ({fname_operand} pred {aux_args_str})
                ({fname_vec_operand} inputs {aux_args_str})
                outputs)
            :ruleset {ruleset})
        (rewrite
            ({fname_body} (Theta pred inputs outputs) {aux_args_str})
            (Theta pred ({fname_vec_operand} inputs {aux_args_str}) outputs)
            :ruleset {ruleset})"
    ));

    // Rules to compute on VecOperand
    for (vectype, ctor, eltype) in [
        ("VecOperand", "VO", "Operand"),
        ("VecVecOperand", "VVO", "VecOperand"),
    ] {
        let fname_vec = func_name_fmt.replace("{}", vectype);
        let fname_eltype = func_name_fmt.replace("{}", eltype);
        // rtjoa: TODO: implement by mapping internally so they're not O(n^2) time
        res.push(format!(
            "
            (function {fname_vec}-helper ({vectype} {aux_params_str} i64) {vectype} :unextractable)
            (rewrite
                ({fname_vec} vec {aux_args_str})
                ({fname_vec}-helper vec {aux_args_str} 0)
                :ruleset {ruleset})
            (rule
                ((= f ({fname_vec}-helper ({ctor} vec) {aux_args_str} i))
                 (< i (vec-length vec)))
                ((union
                    ({fname_vec}-helper ({ctor} vec) {aux_args_str} i)
                    ({fname_vec}-helper
                        ({ctor} (vec-set vec i ({fname_eltype} (vec-get vec i) {aux_args_str})))
                        {aux_args_str} (+ i 1))))
                :ruleset {ruleset})
            (rule
                ((= f ({fname_vec}-helper ({ctor} vec) {aux_args_str} i))
                 (= i (vec-length vec)))
                ((union
                    ({fname_vec}-helper ({ctor} vec) {aux_args_str} i)
                    ({ctor} vec)))
                :ruleset {ruleset})"
        ));
    }
    res
}

// Within e, replace (Args x) with v.
//                      e  [ x -> v ]
// (function SubstTYPE (TYPE i64  Operand) TYPE)
fn subst_rules() -> Vec<String> {
    functions_modifying_args(
        "Subst{}",
        vec!["i64", "Operand"],
        "subst",
        "
        (rewrite (SubstOperand (Arg x) x v) v :ruleset subst)
        (rule ((= f (SubstOperand (Arg y) x v)) (!= y x))
              ((union f (Arg y))) :ruleset subst)",
    )
}

// Within e, replace (Args x), where x > last-unshifted, with (Args (x + amt)).
//                      e    last-unshifted amt
// (function ShiftTYPE (TYPE i64            i64) TYPE)
fn shift_rules() -> Vec<String> {
    functions_modifying_args(
        "Shift{}",
        vec!["i64", "i64"],
        "shift",
        "
        (rule ((= f (ShiftOperand (Arg x) last-unshifted amt)) (<= x last-unshifted))
              ((union f (Arg x))) :ruleset shift)
        (rule ((= f (ShiftOperand (Arg x) last-unshifted amt)) (> x last-unshifted))
              ((union f (Arg (+ x amt)))) :ruleset shift)",
    )
}

// Within e, replace (Args x) with ops[x].
//                         e    ops
// (function SubstTYPEAll (TYPE VecOperand) TYPE)
fn subst_all_rules() -> Vec<String> {
    functions_modifying_args(
        "Subst{}All",
        vec!["VecOperand"],
        "subst",
        "
        (rule ((= f (SubstOperandAll (Arg x) (VO ops)))
               (< x (vec-length ops)))
              ((union f (vec-get ops x))) :ruleset subst)",
    )
}

// Within e, replace (Args x) with map[x].
//                         e    map
// (function SubstTYPEMap (TYPE MapI64Operand) TYPE)
fn subst_map_rules() -> Vec<String> {
    functions_modifying_args(
        "Subst{}Map",
        vec!["MapIntOperand"],
        "subst",
        "
        (rule ((= f (SubstOperandMap (Arg x) (MIO map)))
               (map-contains map x))
              ((union f (map-get map x))) :ruleset subst)
              
        (rule ((= f (SubstOperandMap (Arg x) (MIO map)))
               (map-not-contains map x))
              ((union f (Arg x))) :ruleset subst)",
    )
}

pub(crate) fn all_rules() -> String {
    let mut res = vec!["(ruleset subst) (ruleset subst-beneath) (ruleset shift)".into()];
    res.extend(subst_beneath_rules());
    res.extend(subst_rules());
    res.extend(subst_all_rules());
    res.extend(subst_map_rules());
    res.extend(shift_rules());
    res.join("\n")
}
