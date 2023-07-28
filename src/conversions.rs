use std::{collections::HashMap, iter::once};

use crate::Optimizer;
use bril_rs::{Code, EffectOps, Function, Instruction, Literal, Type, ValueOps};
use egglog::ast::{Expr, Symbol};
use ordered_float::OrderedFloat;

impl Optimizer {
    pub(crate) fn expr_to_func(&mut self, expr: Expr) -> Function {
        if let Expr::Call(func, args) = expr {
            assert_eq!(func.to_string(), "Func");
            match &args.as_slice() {
                [func_name, body] => {
                    if let Expr::Lit(egglog::ast::Literal::String(fname)) = func_name {
                        Function {
                            name: fname.to_string(),
                            args: vec![],
                            instrs: self.body_to_code(body),
                            pos: None,
                            return_type: None,
                        }
                    } else {
                        panic!("expected string literal for func name");
                    }
                }
                _ => panic!("expected 2 args in expr_to_func"),
            }
        } else {
            panic!("expected call in expr_to_func");
        }
    }

    pub(crate) fn body_to_code(&mut self, expr: &Expr) -> Vec<Code> {
        eprintln!("body_to_code: {}", expr);

        if let Expr::Call(op, args) = expr {
            let mut res = vec![];
            match op.to_string().as_str() {
                "Print" => {
                    assert!(args.len() == 2);
                    let arg = self.expr_to_code(&args[0], &mut res);

                    res.push(Code::Instruction(Instruction::Effect {
                        op: EffectOps::Print,
                        args: vec![arg],
                        funcs: vec![],
                        labels: vec![],
                        pos: None,
                    }));
                    res.extend(self.body_to_code(&args[1]));
                }
                "End" => {
                    assert!(args.is_empty());
                }
                "Ret" => {
                    assert_eq!(args.len(), 2);

                    let arg = self.expr_to_code(&args[0], &mut res);

                    res.push(Code::Instruction(Instruction::Effect {
                        op: EffectOps::Return,
                        args: vec![arg],
                        funcs: vec![],
                        labels: vec![],
                        pos: None,
                    }));

                    res.extend(self.body_to_code(&args[1]));
                }
                _ => panic!("unknown effect in body_to_code {}", op),
            }

            res
        } else {
            panic!("expected call in body_to_code");
        }
    }

    // TODO memoize exprs for common subexpression elimination
    pub(crate) fn expr_to_code(&mut self, expr: &Expr, res: &mut Vec<Code>) -> String {
        match expr {
            Expr::Lit(literal) => {
                let fresh = self.fresh();
                res.push(Code::Instruction(Instruction::Constant {
                    dest: fresh.clone(),
                    op: bril_rs::ConstOps::Const,
                    value: self.literal_to_bril(literal),
                    pos: None,
                    const_type: self.literal_to_type(literal),
                }));
                fresh
            }
            Expr::Var(var) => var.to_string(),
            Expr::Call(op, args) => match op.to_string().as_str() {
                "ReturnValue" => self.expr_to_code(&args[0], res),
                "Int" | "True" | "False" => {
                    let fresh = self.fresh();
                    let literal = match op.to_string().as_str() {
                        "Int" => Literal::Int(args[1].to_string().parse().unwrap()),
                        "True" => Literal::Bool(true),
                        "False" => Literal::Bool(false),
                        "Char" => {
                            assert_eq!(args[0].to_string().len(), 1);
                            Literal::Char(args[0].to_string().chars().next().unwrap())
                        }
                        _ => panic!("unknown literal"),
                    };
                    res.push(Code::Instruction(Instruction::Constant {
                        dest: fresh.clone(),
                        op: bril_rs::ConstOps::Const,
                        value: literal,
                        pos: None,
                        const_type: self.expr_to_type(&args[0]),
                    }));
                    fresh
                }
                _ => {
                    let etype = self.expr_to_type(&args[0]);
                    let args_vars = args
                        .iter()
                        .skip(1)
                        .map(|arg| self.expr_to_code(arg, res))
                        .collect::<Vec<String>>();
                    let fresh = self.fresh();
                    res.push(Code::Instruction(Instruction::Value {
                        dest: fresh.clone(),
                        args: args_vars,
                        funcs: vec![],
                        op: self.egglog_op_to_bril(*op),
                        labels: vec![],
                        pos: None,
                        op_type: etype,
                    }));
                    fresh
                }
            },
        }
    }

    pub(crate) fn func_to_expr(&mut self, func: &Function) -> Expr {
        eprintln!("func_to_expr: {}", func.name);

        // leave prints in order
        // leave any effects in order
        // leave assignments to variables used outside
        // of this function
        // otherwise inline
        let mut res = Expr::Call("End".into(), vec![]);
        let mut env = HashMap::<String, Expr>::new();
        for code in &func.instrs {
            if let Code::Instruction(instr) = code {
                self.add_instr_to_env(instr, &mut env);
            }
        }

        // reverse order to build the linked list
        for code in func.instrs.iter().rev() {
            match code {
                Code::Instruction(instr) => {
                    res = self.add_instr_effect(instr, &res, &env);
                }
                Code::Label { pos, label } => {
                    panic!("labels not supported");
                }
            }
        }

        Expr::Call(
            "Func".into(),
            vec![
                Expr::Lit(egglog::ast::Literal::String(func.name.clone().into())),
                res,
            ],
        )
    }

    pub(crate) fn add_instr_effect(
        &mut self,
        instr: &Instruction,
        rest: &Expr,
        env: &HashMap<String, Expr>,
    ) -> Expr {
        match instr {
            Instruction::Effect {
                op,
                args,
                funcs: _funcs,
                labels: _labels,
                pos: _pos,
            } => {
                let arg_exprs = args
                    .iter()
                    .map(|arg| {
                        let arg = env.get(arg).unwrap_or(&Expr::Var(arg.into())).clone();
                        if op == &EffectOps::Return {
                            Expr::Call("ReturnValue".into(), vec![arg])
                        } else {
                            arg
                        }
                    })
                    .chain(std::iter::once(rest.clone()))
                    .collect::<Vec<Expr>>();
                Expr::Call(self.effect_op_to_egglog(*op), arg_exprs)
            }
            _ => rest.clone(),
        }
    }

    pub(crate) fn add_instr_to_env(
        &mut self,
        instr: &Instruction,
        env: &mut HashMap<String, Expr>,
    ) {
        match instr {
            Instruction::Constant {
                dest,
                op: _op,
                value,
                pos: _pos,
                const_type,
            } => match value {
                Literal::Int(int) => {
                    env.insert(
                        dest.clone(),
                        Expr::Call(
                            "Int".into(),
                            vec![
                                self.type_to_expr(const_type),
                                Expr::Lit(egglog::ast::Literal::Int(*int)),
                            ],
                        ),
                    );
                }
                Literal::Bool(bool) => {
                    let expr = if *bool {
                        Expr::Call("True".into(), vec![self.type_to_expr(const_type)])
                    } else {
                        Expr::Call("False".into(), vec![])
                    };
                    env.insert(dest.clone(), expr);
                }
                Literal::Char(char) => {
                    env.insert(
                        dest.clone(),
                        Expr::Call(
                            "Char".into(),
                            vec![
                                self.type_to_expr(const_type),
                                Expr::Lit(egglog::ast::Literal::String(char.to_string().into())),
                            ],
                        ),
                    );
                }
                Literal::Float(float) => {
                    env.insert(
                        dest.clone(),
                        Expr::Call(
                            "Float".into(),
                            vec![
                                self.type_to_expr(const_type),
                                Expr::Lit(egglog::ast::Literal::F64(OrderedFloat(*float))),
                            ],
                        ),
                    );
                }
            },
            Instruction::Value {
                dest,
                args,
                funcs,
                op,
                labels: _labels,
                pos: _pos,
                op_type,
            } => {
                assert!(funcs.is_empty());
                let arg_exprs = once(self.type_to_expr(op_type))
                    .chain(
                        args.iter()
                            .map(|arg| env.get(arg).unwrap_or(&Expr::Var(arg.into())).clone()),
                    )
                    .collect::<Vec<Expr>>();
                let expr = Expr::Call(self.op_to_egglog(*op), arg_exprs);
                env.insert(dest.clone(), expr);
            }
            Instruction::Effect { .. } => {
                // effect handled by next pass
            }
        }
    }

    pub(crate) fn egglog_op_to_bril(&mut self, op: Symbol) -> ValueOps {
        let with_quotes = "\"".to_owned() + &op.to_string() + "\"";
        serde_json::from_str(&with_quotes).unwrap()
    }

    pub(crate) fn effect_op_to_egglog(&mut self, op: EffectOps) -> Symbol {
        let opstr = op.to_string();
        if opstr == "print" {
            "Print".into()
        } else if opstr == "ret" {
            "Ret".into()
        } else {
            let with_quotes = serde_json::to_string(&op).unwrap();
            // remove the quotes around the json string
            let without_quotes = &with_quotes[1..with_quotes.len() - 1];
            without_quotes.into()
        }
    }

    pub(crate) fn op_to_egglog(&mut self, op: ValueOps) -> Symbol {
        let with_quotes = serde_json::to_string(&op).unwrap();
        // remove the quotes around the json string
        let without_quotes = &with_quotes[1..with_quotes.len() - 1];
        without_quotes.into()
    }

    pub(crate) fn literal_to_bril(&self, literal: &egglog::ast::Literal) -> Literal {
        match literal {
            egglog::ast::Literal::Int(int) => Literal::Int(*int),
            egglog::ast::Literal::String(string) => {
                assert!(string.to_string().len() == 1);
                Literal::Char(string.to_string().chars().next().unwrap())
            }
            egglog::ast::Literal::F64(float) => Literal::Float(float.into_inner()),
            egglog::ast::Literal::Unit => panic!("unit literal not supported"),
        }
    }

    pub(crate) fn literal_to_type(&self, literal: &egglog::ast::Literal) -> bril_rs::Type {
        match literal {
            egglog::ast::Literal::Int(_) => bril_rs::Type::Int,
            egglog::ast::Literal::String(_) => bril_rs::Type::Int,
            egglog::ast::Literal::F64(_) => bril_rs::Type::Float,
            egglog::ast::Literal::Unit => panic!("unit literal not supported"),
        }
    }

    pub(crate) fn type_to_expr(&self, ty: &Type) -> egglog::ast::Expr {
        let type_name = serde_json::to_string(&ty).unwrap();
        // remove the quotes around the json string
        let without_quotes = &type_name[1..type_name.len() - 1];
        Expr::Lit(egglog::ast::Literal::String(without_quotes.into()))
    }

    pub(crate) fn expr_to_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Lit(lit) => match lit {
                egglog::ast::Literal::String(string) => {
                    let with_quotes = "\"".to_owned() + &string.to_string() + "\"";
                    serde_json::from_str(&with_quotes).unwrap()
                }
                _ => panic!("expected type literal to be a string"),
            },
            _ => panic!("expected type literal. got: {}", expr),
        }
    }
}
