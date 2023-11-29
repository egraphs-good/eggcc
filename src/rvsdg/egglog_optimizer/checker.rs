use super::AST_SORTS;

use bril_rs::Type;
use super::subst::functions_modifying_args;
use super::BRIL_OPS;
use super::type_to_literal_constructor;
#[cfg(test)]
use crate::rvsdg::egglog_optimizer::build_egglog_test;
use egglog::EGraph;

pub(crate) fn checker_code() -> String {
    let mut res = vec!["
(ruleset checker)
(sort Env (Vec Literal))

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

    res.extend(functions_modifying_args(
        "{}EvalsToDemand",
        vec!["Env"],
        "checker",
        ""));

    for sort in &AST_SORTS {
        res.push(format!("
;; if a node evaluates to a single value, we wrap it in a vector
(function {sort}EvalsTo ({sort} Env) Env)
"));
    }

res.push(format!("
(rule ((LiteralEvalsToDemand (Num a) env))
      ((set (LiteralEvalsTo (Num a) env) (vec-of (Num a)))))
(rule ((LiteralEvalsToDemand (Float a) env))
      ((set (LiteralEvalsTo (Float a) env) (vec-of (Float a)))))
(rule ((LiteralEvalsToDemand (Char a) env))
      ((set (LiteralEvalsTo (Char a) env) (vec-of (Char a)))))
(rule ((LiteralEvalsToDemand (Bool a) env))
      ((set (LiteralEvalsTo (Bool a) env) (vec-of (Bool a)))))

(rule ((= lhs (Const t (const) lit))
       (ExprEvalsToDemand lhs env)
       (= result (LiteralEvalsTo lit env)))
      ((set (ExprEvalsTo lhs env) result)))
      
(rule ((OperandEvalsToDemand (Arg i) env))
      ((set (OperandEvalsTo (Arg i) env) (vec-get env i))))

(rule (
        (= lhs (Node body))
        (OperandEvalsToDemand lhs env)
        (= result (BodyEvalsTo body env))
      )
      ((set (OperandEvalsTo lhs env) result)))
(rule (
        (= lhs (Project i body))
        (OperandEvalsToDemand lhs env)
        (= result (BodyEvalsTo body env))
      )
      ((set (OperandEvalsTo lhs env) (vec-of (vec-get result i)))))

(rule (
        (= lhs (PureOp expr))
        (BodyEvalsToDemand lhs env)
        (= result (ExprEvalsTo expr env))
      )
      ((set (BodyEvalsTo lhs env) result)))
      "));

    for op in BRIL_OPS {
        let name = op.op;
        let egglog_name = op.egglog_op;
        let constructor = type_to_literal_constructor(&op.output_type);
        match op.input_types {
            [Some(Type::Int), Some(Type::Int)] => {
                res.push(format!("
(rule ((= lhs ({name} (IntT) a b))
       (ExprEvalsToDemand lhs env)
       (= (Num childa) (vec-get (OperandEvalsTo a env) 0))
       (= (Num childb) (vec-get (OperandEvalsTo b env) 0)))
      ((set (ExprEvalsTo lhs env)
            (vec-of ({constructor}
                      ({egglog_name}
                           childa
                           childb))))))
        "));
            }

            _ => (),
        }
    }
        


// bodies dont do them ill paste them back


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

(ExprEvalsToDemand testadd (vec-of))
    "#;

    const footer: &str = r#"
(check (= (vec-of (Num 3)) (ExprEvalsTo testadd (vec-of))))
    "#;

    let mut egraph = build_egglog_test(PROGRAM);
    egraph.parse_and_run_program(&footer).unwrap();
}