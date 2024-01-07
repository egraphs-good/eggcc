use bril_rs::{ConstOps, Literal, Type};
use egglog::{Term, TermDag};
use hashbrown::HashMap;

use crate::conversions::egglog_op_to_bril;

use super::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgType};

struct RvsdgFromEgglog<'a> {
    termdag: &'a TermDag,
    body_cache: HashMap<Term, Id>,
    bodies: Vec<RvsdgBody>,
}

impl<'a> RvsdgFromEgglog<'a> {
    fn egglog_term_to_operand(&mut self, op: Term) -> Operand {
        use egglog::ast::Literal::*;
        if let Term::App(func, args) = &op {
            let args = args
                .iter()
                .map(|t| self.termdag.get(*t))
                .collect::<Vec<_>>();
            let mut project_body = |n: &i64, body: &Term| {
                let Term::App(bfunc, bargs) = body else {
                    panic!(
                        "expected a body after project, got {}",
                        self.termdag.to_string(body)
                    )
                };
                let bargs = bargs
                    .iter()
                    .map(|t| self.termdag.get(*t))
                    .collect::<Vec<_>>();

                // Every `OperandGroup` should be used by a
                // `Project`, so check that here and
                // optimize them away.
                match (bfunc.as_str(), &bargs.as_slice()) {
                    ("OperandGroup", [groupbody]) => {
                        let results = self.expr_to_vec_operand(groupbody.clone(), "VO");
                        assert!(results.len() > *n as usize);
                        results[*n as usize]
                    }
                    _ => Operand::Project(*n as usize, self.egglog_term_to_body(body.clone())),
                }
            };
            match (func.as_str(), &args.as_slice()) {
                ("Arg", [Term::Lit(Int(n))]) => Operand::Arg(*n as usize),
                ("Node", [body]) => project_body(&0, body),
                ("Project", [Term::Lit(Int(n)), body]) => project_body(n, body),
                _ => panic!("expected an operand, got {}", self.termdag.to_string(&op)),
            }
        } else {
            panic!("expected an operand, got {}", self.termdag.to_string(&op))
        }
    }

    fn expr_to_vec_operand(&mut self, vec: Term, constructor: &str) -> Vec<Operand> {
        let Term::App(func, args) = vec else {
            panic!("Expected a VO, got {}", self.termdag.to_string(&vec))
        };
        let args = args
            .iter()
            .map(|t| self.termdag.get(*t))
            .collect::<Vec<_>>();
        assert_eq!(func.as_str(), constructor);
        assert_eq!(args.len(), 1);
        let vec = &args[0];
        vec_map(vec.clone(), self.termdag, |term| {
            self.egglog_term_to_operand(term)
        })
    }

    fn expr_to_vec_vec_operand(&mut self, vec_vec: Term) -> Vec<Vec<Operand>> {
        let Term::App(func, args) = vec_vec else {
            panic!("Expected a VVO, got {}", self.termdag.to_string(&vec_vec))
        };
        let args = args
            .iter()
            .map(|t| self.termdag.get(*t))
            .collect::<Vec<_>>();

        assert_eq!(func.as_str(), "VVO");
        assert_eq!(args.len(), 1);
        let vec_vec = &args[0];
        vec_map(vec_vec.clone(), self.termdag, |vec| {
            self.expr_to_vec_operand(vec, "VOC")
        })
    }

    fn egglog_term_to_body(&mut self, body: Term) -> Id {
        if let Some(id) = self.body_cache.get(&body) {
            return *id;
        }
        let res = if let Term::App(func, args) = &body {
            let body = match (func.as_str(), &args.as_slice()) {
                ("PureOp", [expr]) => {
                    let expr = self.termdag.get(*expr);
                    RvsdgBody::BasicOp(self.egglog_expr_to_expr(expr))
                }

                ("Gamma", [pred, inputs, outputs]) => {
                    // TODO make match_term_app macro
                    // better and do conversion to term for us
                    let pred = self.termdag.get(*pred);
                    let inputs = self.termdag.get(*inputs);
                    let outputs = self.termdag.get(*outputs);

                    let pred = self.egglog_term_to_operand(pred);
                    let inputs = self.expr_to_vec_operand(inputs, "VO");
                    let outputs = self.expr_to_vec_vec_operand(outputs);
                    RvsdgBody::Gamma {
                        pred,
                        inputs,
                        outputs,
                    }
                }
                ("Theta", [pred, inputs, outputs]) => {
                    let pred = self.termdag.get(*pred);
                    let inputs = self.termdag.get(*inputs);
                    let outputs = self.termdag.get(*outputs);

                    let pred = self.egglog_term_to_operand(pred);
                    let inputs = self.expr_to_vec_operand(inputs, "VO");
                    let outputs = self.expr_to_vec_operand(outputs, "VO");
                    RvsdgBody::Theta {
                        pred,
                        inputs,
                        outputs,
                    }
                }
                _ => panic!("expect an operand, got {}", self.termdag.to_string(&body)),
            };
            self.bodies.push(body);
            self.bodies.len() - 1
        } else {
            panic!("expect an operand, got {}", self.termdag.to_string(&body))
        };

        self.body_cache.insert(body, res);
        res
    }

    fn egglog_expr_to_expr(&mut self, expr: Term) -> BasicExpr<Operand> {
        use egglog::ast::Literal;
        if let Term::App(func, args) = &expr {
            let args = args
                .iter()
                .map(|t| self.termdag.get(*t))
                .collect::<Vec<_>>();
            match (func.as_str(), &args.as_slice()) {
                (
                    "Call",
                    [ty, Term::Lit(Literal::String(ident)), args, Term::Lit(Literal::Int(n_outs))],
                ) => {
                    let args = self.expr_to_vec_operand(args.clone(), "VO");
                    let ty = self.egglog_expr_to_option_ty(ty.clone());
                    BasicExpr::Call(ident.to_string(), args, *n_outs as usize, ty)
                }
                ("Const", [ty, _const_op, lit]) => BasicExpr::Const(
                    // todo remove the const op from the encoding because it is always ConstOps::Const
                    ConstOps::Const,
                    self.egglog_expr_to_literal(lit.clone()),
                    self.egglog_expr_to_ty(ty.clone()),
                ),
                ("PRINT", [opr1, opr2]) => {
                    let opr1 = self.egglog_term_to_operand(opr1.clone());
                    let opr2 = self.egglog_term_to_operand(opr2.clone());
                    BasicExpr::Print(vec![opr1, opr2])
                }
                (binop, [ty, opr1, opr2]) => {
                    let opr1 = self.egglog_term_to_operand(opr1.clone());
                    let opr2 = self.egglog_term_to_operand(opr2.clone());
                    BasicExpr::Op(
                        egglog_op_to_bril(binop.into()),
                        vec![opr1, opr2],
                        self.egglog_expr_to_ty(ty.clone()),
                    )
                }
                (unop, [ty, opr]) => {
                    let opr = self.egglog_term_to_operand(opr.clone());
                    BasicExpr::Op(
                        egglog_op_to_bril(unop.into()),
                        vec![opr],
                        self.egglog_expr_to_ty(ty.clone()),
                    )
                }
                _ => panic!(
                    "expected an expression, got {}",
                    self.termdag.to_string(&expr)
                ),
            }
        } else {
            panic!(
                "expect an expression, got {}",
                self.termdag.to_string(&expr)
            )
        }
    }

    fn egglog_expr_to_ty(&self, ty: Term) -> Type {
        if let Term::App(func, args) = &ty {
            let args = args
                .iter()
                .map(|t| self.termdag.get(*t))
                .collect::<Vec<_>>();
            match (func.as_str(), &args.as_slice()) {
                ("IntT", []) => Type::Int,
                ("BoolT", []) => Type::Bool,
                ("FloatT", []) => Type::Float,
                ("CharT", []) => Type::Char,
                ("PointerT", [inner]) => {
                    Type::Pointer(Box::new(self.egglog_expr_to_ty(inner.clone())))
                }
                _ => panic!("expect a list, got {}", self.termdag.to_string(&ty)),
            }
        } else {
            panic!("expect a list, got {}", self.termdag.to_string(&ty))
        }
    }

    fn egglog_expr_to_option_ty(&self, ty: Term) -> Option<Type> {
        if let Term::App(func, args) = &ty {
            let args = args
                .iter()
                .map(|t| self.termdag.get(*t))
                .collect::<Vec<_>>();
            match (func.as_str(), &args.as_slice()) {
                ("SomeType", [ty]) => Some(self.egglog_expr_to_ty(ty.clone())),
                ("NoneType", []) => None,
                _ => panic!("expect an option type, got {}", self.termdag.to_string(&ty)),
            }
        } else {
            panic!("expect an option type, got {}", self.termdag.to_string(&ty))
        }
    }

    fn egglog_expr_to_rvsdg_ty(&self, ty: Term) -> RvsdgType {
        if let Term::App(func, args) = &ty {
            let args = args
                .iter()
                .map(|t| self.termdag.get(*t))
                .collect::<Vec<_>>();
            match (func.as_str(), &args.as_slice()) {
                ("PrintState", []) => RvsdgType::PrintState,
                ("Bril", [ty]) => RvsdgType::Bril(self.egglog_expr_to_ty(ty.clone())),
                _ => panic!("expect an expression, got {}", self.termdag.to_string(&ty)),
            }
        } else {
            panic!("expect an expression, got {}", self.termdag.to_string(&ty))
        }
    }

    fn egglog_expr_to_literal(&self, lit: Term) -> Literal {
        use egglog::ast::Literal::*;
        if let Term::App(func, args) = &lit {
            let args = args
                .iter()
                .map(|t| self.termdag.get(*t))
                .collect::<Vec<_>>();
            match (func.as_str(), &args.as_slice()) {
                ("Num", [Term::Lit(Int(n))]) => Literal::Int(*n),
                ("Float", [Term::Lit(F64(n))]) => Literal::Float(f64::from(*n)),
                ("Char", [Term::Lit(String(s))]) => {
                    assert_eq!(s.as_str().len(), 1);
                    Literal::Char(s.as_str().chars().next().unwrap())
                }
                ("Bool", [Term::Lit(Bool(bool))]) => Literal::Bool(*bool),
                _ => panic!("expect a lit, got {}", self.termdag.to_string(&lit)),
            }
        } else {
            panic!("expect a lit, got {}", self.termdag.to_string(&lit))
        }
    }
}

impl RvsdgFunction {
    pub fn egglog_term_to_function(term: Term, termdag: &TermDag) -> RvsdgFunction {
        use egglog::ast::Literal::*;

        let mut convert = RvsdgFromEgglog {
            termdag,
            body_cache: HashMap::new(),
            bodies: vec![],
        };

        if let Term::App(func, args) = &term {
            let arg_terms = args.iter().map(|t| termdag.get(*t)).collect::<Vec<_>>();
            match (func.as_str(), &arg_terms.as_slice()) {
                ("Func", [Term::Lit(String(name)), input_sig, output_sig, body]) => {
                    let args: Vec<RvsdgType> = vec_map(input_sig.clone(), termdag, |ty| {
                        convert.egglog_expr_to_rvsdg_ty(ty)
                    });

                    let result_types = vec_map(output_sig.clone(), termdag, |ty| {
                        convert.egglog_expr_to_rvsdg_ty(ty)
                    });

                    let result_values = convert.expr_to_vec_operand(body.clone(), "VO");

                    let results = result_types.into_iter().zip(result_values).collect();
                    RvsdgFunction {
                        name: name.to_string(),
                        args,
                        nodes: convert.bodies,
                        results,
                    }
                }
                _ => panic!("expect a function, got {}", termdag.to_string(&term)),
            }
        } else {
            panic!("expect a function, got {}", termdag.to_string(&term))
        }
    }
}

/// Call `f` on each element of `inputs`, which should be a fully
/// expanded egglog expression representing a vector.
/// Returns the result of `f` for each element.
fn vec_map<T>(mut inputs: Term, termdag: &TermDag, mut f: impl FnMut(Term) -> T) -> Vec<T> {
    let mut results = vec![];
    if let Term::App(func, args) = &inputs {
        if func.as_str() == "vec-of" {
            return args.iter().map(|t| f(termdag.get(*t))).collect();
        }
    }
    loop {
        if let Term::App(func, args) = &inputs {
            match (func.as_str(), &args.as_slice()) {
                ("vec-push", [head, tail]) => {
                    results.push(f(termdag.get(*head)));
                    inputs = termdag.get(*tail);
                }
                ("vec-empty", []) => {
                    break;
                }
                _ => panic!("expect a list, got {}", termdag.to_string(&inputs)),
            }
        } else {
            panic!("expect a list, got {}", termdag.to_string(&inputs))
        }
    }
    results.reverse();
    results
}
