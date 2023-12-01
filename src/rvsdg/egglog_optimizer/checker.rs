use super::AST_SORTS;

use super::subst::functions_modifying_args;
use super::type_to_literal_constructor;
use super::BRIL_OPS;
#[cfg(test)]
use crate::rvsdg::egglog_optimizer::build_egglog_test;
use bril_rs::Type;
use egglog::EGraph;

pub(crate) fn checker_code() -> String {
    let mut res = vec!["
(ruleset checker)
(sort EnvVec (Vec Literal))
(datatype Env
  (E EnvVec))

(rule ((= (E vec1) (E vec2))
       (!= vec1 vec2))
      ((panic \"two envs with different values were unioned\"))
      :ruleset checker)

;; Sanity checks: make sure no constants are equal in the database
(rule ((= (Num a) (Num b)) (!= a b)) ((panic \"unioned two numbers with different values\"))
      :ruleset checker)
(rule ((= (Num a) (Float b))) ((panic \"num and float cannot be equal\"))
:ruleset checker)
(rule ((= (Num a) (Char b))) ((panic \"num and char cannot be equal\"))
:ruleset checker)
(rule ((= (Num a) (Bool b))) ((panic \"num and bool cannot be equal\"))
:ruleset checker)
(rule ((= (Float a) (Float b)) (!= a b)) ((panic \"unioned two floats with different values\"))
:ruleset checker)
(rule ((= (Float a) (Char b))) ((panic \"float and char cannot be equal\"))
:ruleset checker)
(rule ((= (Float a) (Bool b))) ((panic \"float and bool cannot be equal\"))
:ruleset checker)
(rule ((= (Char a) (Char b)) (!= a b)) ((panic \"unioned two chars with different values\"))
:ruleset checker)
(rule ((= (Char a) (Bool b))) ((panic \"char and bool cannot be equal\"))
:ruleset checker)
(rule ((= (Bool a) (Bool b)) (!= a b)) ((panic \"unioned two bools with different values\"))
:ruleset checker)
    "
    .to_string()];

    for sort in &AST_SORTS {
        res.push(format!(
            "
;; if a node evaluates to a single value, we wrap it in a vector
(function {sort}EvalsTo ({sort} Env) Env)
"
        ));
    }

    res.push(format!(
        "
(rewrite (ExprEvalsTo (Const t (const) lit) (E env))
         (E (vec-of lit))
         :ruleset checker)

(rewrite (OperandEvalsTo (Arg i) (E env)) (E (vec-of (vec-get env i)))
         :ruleset checker)
         
(rewrite (OperandEvalsTo (Node body) (E env))
         (BodyEvalsTo body (E env))
         :ruleset checker)

(rewrite (OperandEvalsTo (Project i body) (E env))
         (E (vec-of (vec-get body-vals i)))
         :when ((= (BodyEvalsTo body (E env)) (E body-vals)))
         :ruleset checker)

(function VecGet (Env i64) Literal)
(rewrite (VecGet (E vec) i) (vec-get vec i)
         :ruleset checker)
         
         
(relation VecOperandInProgress
    ;; vector, environment, in-progress result
    (VecOperand Env Env))

(rule ((VecOperandEvalsTo vec env))
      ((VecOperandInProgress vec env (E (vec-of)))))

(rule ((VecOperandInProgress vec env (E progress-vec))
       (< (vec-length progress-vec) (vec-length vec))
       (= l (vec-length progress-vec)))
      (
        (VecOperandInProgress vec env
          (E (vec-push
               (VecGet
                 (OperandEvalsTo (vec-get vec l))
                 0)
               progress-vec)))))
;;;;

(function prefix-of-VecOperand-evals-to (VecOperand i64 Env) Env)
 
; the empty prefix evaluates to an empty vec
(rule ((VecOperandEvalsTo vec env))
      ((set (prefix-of-VecOperand-evals-to vec 0 env) (vec-empty))))

; grow prefix
(rule ((VecOperandEvalsTo vec env)
       (= (E prefix-vals) (prefix-of-VecOperand-evals-to vec i env))
       (= next-op (VecOperand-get vec i))
       (= next-op-val (OperandEvalsTo next-op env)))
      ((set (prefix-of-VecOperand-evals-to vec (+ i 1) env)
            (E (vec-push prefix-vals next-op-val)))))

; if prefix of length of the vec, save final result
(rule ((VecOperandEvalsTo vec env)
       (= l (VecOperand-length vec))
       (= all-vals (prefix-of-VecOperand-evals-to vec l env)))
      ((set (VecOperandEvalsTo vec env) all-vals)))
;;;


; Evaluate the individual element
(rule ((VecOperandEvalsTo e env)
       (VecOperand-get e i el))
      ((OperandEvalsTo el env))
       :ruleset checker)

(rewrite (BodyEvalsTo (PureOp expr) (E env))
         (ExprEvalsTo expr (E env))
         :ruleset checker)
      "
    ));

    for op in BRIL_OPS {
        let name = op.op;
        let egglog_name = op.egglog_op;
        let constructor = type_to_literal_constructor(&op.output_type);
        match op.input_types {
            [Some(Type::Int), Some(Type::Int)] => {
                res.push(format!(
                    "
                
;; demand rule

; (function Lit{name} (VecLiteral VecLiteral) VecLiteral))
; if num:
    ; (rewrite (Lit{name} (vec-of (Num x)) (vec-of (Num y)) (vec-of ({egglog_name} x y))))
; if bool:
    ; (rewrite (Lit{name} (vec-of (Bool x)) (vec-of (Bool y)) (vec-of ({egglog_name} x y))))
; (rewrite (ExprEvalsTo ({name} ty a b) (E env)) (Lit{name} ))

(rule ((ExprEvalsTo ({name} ignored-type a b) (E env)))
      ((OperandEvalsTo a (E env))
       (OperandEvalsTo b (E env)))
       :ruleset checker)

(rule ((= lhs ({name} (IntT) a b))
       (ExprEvalsTo lhs (E env))
       (= (OperandEvalsTo a (E env)) (E a-vals))
       (= (OperandEvalsTo b (E env)) (E b-vals))
       (= (Num childa) (vec-get a-vals 0))
       (= (Num childb) (vec-get b-vals 0)))
      ((union (ExprEvalsTo lhs (E env))
              (E (vec-of ({constructor}
                         ({egglog_name}
                           childa
                           childb))))))
     :ruleset checker)
        "
                ));
            }

            _ => (),
        }
    }

    // Theta
    res.push(format!(
        "
        (function ThetaOutputsEvalToAtIter (Body Env i64) Env)
        (function ThetaPredEvalsToAtIter (Body Env i64) Env)

        ; demand inputs get evaluated
        (rule ((BodyEvalsTo (Theta pred inputs outputs) env))
              ((VecOperandEvalsTo inputs env))
              :ruleset checker)

        ; hack: at iter -1, set the pred to true and outputs to inputs
        (rule ((= theta (Theta pred inputs outputs))
               (BodyEvalsTo (Theta pred inputs outputs) env)
               (= inputs-vals (VecOperandEvalsTo inputs env)))
              ((union (ThetaOutputsEvalToAtIter theta env -1) inputs-vals)
               (union (ThetaPredEvalsToAtIter theta env -1) (E (vec-of (Bool true)))))
              :ruleset checker)

        ; if pred is false at the end of some iter, its outputs are the overall result
        (rule ((= theta (Theta pred inputs outputs))
               (BodyEvalsTo theta env)
               (= output-vals (ThetaOutputsEvalToAtIter theta env i))
               (= (E (vec-of (Bool false))) (ThetaPredEvalsToAtIter theta env i)))
              ((union (BodyEvalsTo theta env) output-vals))
              :ruleset checker)

        ; if pred is true, demand next pred and env...
        (rule ((= theta (Theta pred inputs outputs))
               (BodyEvalsTo theta env)
               (= next-env (ThetaOutputsEvalToAtIter theta env i))
               (= (E (vec-of (Bool true))) (ThetaPredEvalsToAtIter theta env i)))
              ((OperandEvalsTo pred next-env)
               (VecOperandEvalsTo outputs next-env))
              :ruleset checker)
        
        ; ...then set what the outputs/preds eval to at the next iter
        (rule ((= theta (Theta pred inputs outputs))
               (BodyEvalsTo theta env)
               (= next-env (ThetaOutputsEvalToAtIter theta env i))
               (= (E (vec-of (Bool true))) (ThetaPredEvalsToAtIter theta env i))
               (= next-pred (OperandEvalsTo pred next-env))
               (= next-outputs (VecOperandEvalsTo outputs next-env)))
              ((union (ThetaOutputsEvalToAtIter theta env (+ i 1)) next-outputs)
               (union (ThetaPredEvalsToAtIter theta env (+ i 1)) next-outputs))
              :ruleset checker)
    "
    ));

    // Gamma
    res.push(format!(
        "
        ; demand pred gets evaluated
        (rule ((BodyEvalsTo (Gamma pred inputs outputs) env))
              ((OperandEvalsTo pred env)
               (VecOperandEvalsTo inputs env))
              :ruleset checker)

        ; demand correct branch gets evaluated
        (rule ((BodyEvalsTo (Gamma pred inputs outputs) env)
               (= (E (vec-of (Num i))) (OperandEvalsTo pred env))
               (= new-env (VecOperandEvalsTo inputs env))
               (= outputs-i (VecVecOperandCtx-get outputs i)))
              ((VecOperandCtxEvalsTo outputs-i new-env))
              :ruleset checker)

        (rule ((BodyEvalsTo (Gamma pred inputs outputs) env)
               (= (E (vec-of (Num i))) (OperandEvalsTo pred env))
               (= new-env (VecOperandEvalsTo inputs env))
               (= outputs-i (VecVecOperandCtx-get outputs i))
               (= outputs-i-vals (VecOperandCtxEvalsTo outputs-i new-env)))
              ((union (BodyEvalsTo (Gamma pred inputs outputs) env) outputs-i-vals))
              :ruleset checker)
    ; "
    ));

    res.join("\n")
}

#[test]
fn test_evaluate_add() {
    const PROGRAM: &str = r#"
(let testadd (badd (IntT)
                   (Node (PureOp (Const (IntT) (const)
                                        (Num 1))))
                   (Node (PureOp (Const (IntT) (const)
                                        (Num 2))))))
(let empty-env (E (vec-pop (vec-of (Num 3)))))
(let vec3 (E (vec-of (Num 3))))
(ExprEvalsTo testadd empty-env)
    "#;

    const FOOTER: &str = r#"
(check (= vec3 (ExprEvalsTo testadd empty-env)))
    "#;

    let mut egraph = EGraph::default();
    let code = build_egglog_test(PROGRAM);
    let code_and_footer = format!("{}\n{}", code, FOOTER);
    println!("{}", code_and_footer);
    match egraph.parse_and_run_program(&code_and_footer) {
        Ok(_) => (),
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_evaluate_gamma() {
    const PROGRAM: &str = r#"

(let empty-env (E (vec-pop (vec-of (Num 3)))))

(let testgamma
    (Gamma
        (Arg 0)
        (VO (vec-of
            (Arg 1)
            (Arg 2)
        ))
        (VVO (vec-of
            (VOC (vec-of
                (Arg 0)
            ))
            (VOC (vec-of
                (Node (PureOp (badd (IntT) (Arg 0) (Arg 1))))
            ))
        ))
    )
)
(let myenv (E (vec-of (Num 1) (Num 10) (Num 20))))
(let vec30 (E (vec-of (Num 30))))

(BodyEvalsTo testgamma myenv)

    "#;

    const FOOTER: &str = r#"
(check (= vec30 (BodyEvalsTo testgamma myenv)))
    "#;

    let mut egraph = EGraph::default();
    let code = build_egglog_test(PROGRAM);
    let code_and_footer = format!("{}\n{}", code, FOOTER);
    println!("{}", code_and_footer);
    match egraph.parse_and_run_program(&code_and_footer) {
        Ok(_) => (),
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_evaluate_theta() {
    const PROGRAM: &str = r#"

(let empty-env (E (vec-pop (vec-of (Num 3)))))

(let one (Node (PureOp (Const (IntT) (const) (Num 1)))))
(let ten (Node (PureOp (Const (IntT) (const) (Num 10)))))
(let testtheta
    (Theta
        ;; stop when i is 10
        (Node (PureOp (blt (BoolT) (Arg 0) ten)))
        (VO (vec-of
            (Arg 0)
        ))
        (VO (vec-of
            ;; add one each iteration
            (Node (PureOp (badd (IntT) (Arg 0)
                                       one
                                       )))
        ))
     ))

(let myenv (E (vec-of (Num 1))))
(let expected (E (vec-of (Num 10))))

(BodyEvalsTo testtheta myenv)

    "#;

    const FOOTER: &str = r#"
(check (= expected (BodyEvalsTo testtheta myenv)))
    "#;

    let mut egraph = EGraph::default();
    let code = build_egglog_test(PROGRAM);
    match egraph.parse_and_run_program(&code) {
        Ok(_) => (),
        Err(e) => panic!("Error: {}", e),
    }
    match egraph.parse_and_run_program(&FOOTER) {
        Ok(_) => (),
        Err(e) => panic!("Error: {}", e),
    }
}
