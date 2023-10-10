use bril_rs::{ConstOps, Literal, Type};
use egglog::ast::Expr;

use crate::{cfg::Identifier, conversions::egglog_op_to_bril};

use super::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgType};

impl RvsdgFunction {
    fn egglog_expr_to_operand(op: &Expr, bodies: &mut Vec<RvsdgBody>) -> Operand {
        use egglog::ast::{Expr::*, Literal::*};
        if let Call(func, args) = op {
            match (func.as_str(), &args.as_slice()) {
                ("Arg", [Lit(Int(n))]) => Operand::Arg(*n as usize),
                ("Node", [body]) => Operand::Id(Self::egglog_expr_to_body(body, bodies)),
                ("Project", [Lit(Int(n)), body]) => {
                    Operand::Project(*n as usize, Self::egglog_expr_to_body(body, bodies))
                }
                _ => panic!("expect an operand, got {op}"),
            }
        } else {
            panic!("expect an operand, got {op}")
        }
    }

    fn expr_to_vec_operand(vec: &Expr, bodies: &mut Vec<RvsdgBody>) -> Vec<Operand> {
        let Expr::Call(func, args) = vec else {
            panic!("Expected a VO, got {vec}")
        };
        assert_eq!(func.as_str(), "VO");
        assert_eq!(args.len(), 1);
        let vec = &args[0];
        vec_map(vec, |e| Self::egglog_expr_to_operand(e, bodies))
    }

    fn expr_to_vec_vec_operand(vec_vec: &Expr, bodies: &mut Vec<RvsdgBody>) -> Vec<Vec<Operand>> {
        let Expr::Call(func, args) = vec_vec else {
        panic!("Expected a VVO, got {vec_vec}")
      };
        assert_eq!(func.as_str(), "VVO");
        assert_eq!(args.len(), 1);
        let vec_vec = &args[0];
        vec_map(vec_vec, |vec| Self::expr_to_vec_operand(vec, bodies))
    }

    fn egglog_expr_to_body(body: &Expr, bodies: &mut Vec<RvsdgBody>) -> Id {
        use Expr::*;
        if let Call(func, args) = body {
            let body = match (func.as_str(), &args.as_slice()) {
                ("PureOp", [expr]) => RvsdgBody::BasicOp(Self::egglog_expr_to_expr(expr, bodies)),
                ("Gamma", [pred, inputs, outputs]) => {
                    let pred = Self::egglog_expr_to_operand(pred, bodies);
                    let inputs = Self::expr_to_vec_operand(inputs, bodies);
                    let outputs = Self::expr_to_vec_vec_operand(outputs, bodies);
                    RvsdgBody::Gamma {
                        pred,
                        inputs,
                        outputs,
                    }
                }
                ("Theta", [pred, inputs, outputs]) => {
                    let pred = Self::egglog_expr_to_operand(pred, bodies);
                    let inputs = Self::expr_to_vec_operand(inputs, bodies);
                    let outputs = Self::expr_to_vec_operand(outputs, bodies);
                    RvsdgBody::Theta {
                        pred,
                        inputs,
                        outputs,
                    }
                }
                ("VO", _operands) => RvsdgBody::Operands {
                    operands: Self::expr_to_vec_operand(body, bodies),
                },
                _ => panic!("expected a body, got {body}"),
            };
            bodies.push(body);
            bodies.len() - 1
        } else {
            panic!("expected a body, got {body}")
        }
    }

    fn egglog_expr_to_expr(expr: &Expr, bodies: &mut Vec<RvsdgBody>) -> BasicExpr<Operand> {
        use egglog::ast::Literal;
        if let Expr::Call(func, args) = expr {
            match (func.as_str(), &args.as_slice()) {
                ("Call", [ty, Expr::Lit(Literal::String(ident)), args]) => {
                    let args = vec_map(args, |e| Self::egglog_expr_to_operand(e, bodies));
                    // TODO: this is imprecise, we don't know if the number of outputs is 1 or 2.
                    BasicExpr::Call(
                        Identifier::Name(ident.to_string()),
                        args,
                        1,
                        Some(Self::egglog_expr_to_ty(ty)),
                    )
                }
                ("Const", [ty, _const_op, lit]) => BasicExpr::Const(
                    // todo remove the const op from the encoding because it is always ConstOps::Const
                    ConstOps::Const,
                    Self::egglog_expr_to_literal(lit),
                    Self::egglog_expr_to_ty(ty),
                ),
                ("PRINT", [opr1, opr2]) => {
                    let opr1 = Self::egglog_expr_to_operand(opr1, bodies);
                    let opr2 = Self::egglog_expr_to_operand(opr2, bodies);
                    BasicExpr::Print(vec![opr1, opr2])
                }
                (binop, [ty, opr1, opr2]) => {
                    let opr1 = Self::egglog_expr_to_operand(opr1, bodies);
                    let opr2 = Self::egglog_expr_to_operand(opr2, bodies);
                    BasicExpr::Op(
                        egglog_op_to_bril(binop.into()),
                        vec![opr1, opr2],
                        Self::egglog_expr_to_ty(ty),
                    )
                }
                _ => panic!("expected an expression, got {expr}"),
            }
        } else {
            panic!("expect an expression, got {expr}")
        }
    }

    fn egglog_expr_to_ty(ty: &Expr) -> Type {
        use Expr::*;
        if let Call(func, args) = ty {
            match (func.as_str(), &args.as_slice()) {
                ("IntT", []) => Type::Int,
                ("BoolT", []) => Type::Bool,
                ("FloatT", []) => Type::Float,
                ("CharT", []) => Type::Char,
                ("PointerT", [inner]) => Type::Pointer(Box::new(Self::egglog_expr_to_ty(inner))),
                _ => panic!("expect a list, got {ty}"),
            }
        } else {
            panic!("expect a list, got {ty}")
        }
    }

    fn egglog_expr_to_rvsdg_ty(ty: &Expr) -> RvsdgType {
        use Expr::*;
        if let Call(func, args) = ty {
            match (func.as_str(), &args.as_slice()) {
                ("PrintState", []) => RvsdgType::PrintState,
                ("Bril", [ty]) => RvsdgType::Bril(Self::egglog_expr_to_ty(ty)),
                _ => panic!("expect an expression, got {ty}"),
            }
        } else {
            panic!("expect an expression, got {ty}")
        }
    }

    fn egglog_expr_to_literal(lit: &Expr) -> Literal {
        use egglog::ast::{Expr::*, Literal::*};
        if let Call(func, args) = lit {
            match (func.as_str(), &args.as_slice()) {
                ("Num", [Lit(Int(n))]) => Literal::Int(*n),
                ("Float", [Lit(F64(n))]) => Literal::Float(f64::from(*n)),
                ("Char", [Lit(String(s))]) => {
                    assert_eq!(s.as_str().len(), 1);
                    Literal::Char(s.as_str().chars().next().unwrap())
                }
                ("Bool", [Lit(Bool(bool))]) => Literal::Bool(*bool),
                _ => panic!("expected a literal, got {lit}"),
            }
        } else {
            panic!("expect a literal, got {lit}")
        }
    }

    pub fn egglog_expr_to_function(expr: &Expr) -> RvsdgFunction {
        eprintln!("expr: {}", expr);
        use egglog::ast::{Expr::*, Literal::*};
        if let Call(func, args) = expr {
            match (func.as_str(), &args.as_slice()) {
                ("Func", [Lit(String(name)), sig, Call(func_output, func_args)]) => {
                    let args: Vec<RvsdgType> = vec_map(sig, Self::egglog_expr_to_rvsdg_ty);
                    let n_args = args.len() - 1;

                    let mut nodes = vec![];
                    let (state, result) = match (func_output.as_str(), &func_args.as_slice()) {
                        ("StateOnly", [state]) => {
                            (Self::egglog_expr_to_operand(state, &mut nodes), None)
                        }
                        ("StateAndValue", [state, ty, result]) => {
                            let state = Self::egglog_expr_to_operand(state, &mut nodes);
                            let result = Self::egglog_expr_to_operand(result, &mut nodes);
                            let ty = Self::egglog_expr_to_ty(ty);
                            (state, Some((ty, result)))
                        }
                        _ => panic!("expect a function, got {expr}"),
                    };
                    RvsdgFunction {
                        name: name.to_string(),
                        n_args,
                        args,
                        nodes,
                        result,
                        state,
                    }
                }
                _ => panic!("expect a function, got {expr}"),
            }
        } else {
            panic!("expect a function, got {expr}")
        }
    }
}

/// Call `f` on each element of `inputs`, which should be a fully
/// expanded egglog expression representing a vector.
/// Returns the result of `f` for each element.
fn vec_map<T>(mut inputs: &Expr, mut f: impl FnMut(&Expr) -> T) -> Vec<T> {
    use Expr::*;
    let mut results = vec![];
    if let Call(func, args) = inputs {
        if func.as_str() == "vec-of" {
            return args.iter().map(&mut f).collect();
        }
    }
    loop {
        if let Call(func, args) = inputs {
            match (func.as_str(), &args.as_slice()) {
                ("vec-push", [head, tail]) => {
                    results.push(f(head));
                    inputs = tail;
                }
                ("vec-empty", []) => {
                    break;
                }
                _ => panic!("expect a list, got {inputs}"),
            }
        } else {
            panic!("expect a list, got {inputs}")
        }
    }
    results.reverse();
    results
}
