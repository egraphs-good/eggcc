use std::{collections::HashMap, iter::once};

use crate::{
    cfg::{
        structured::{StructuredBlock, StructuredFunction},
        BasicBlock, BlockName,
    },
    EggCCError, Optimizer,
};
use bril_rs::{Argument, Code, EffectOps, Instruction, Literal, Program, Type, ValueOps};
use egglog::ast::{Expr, Symbol};
use egglog::{match_term_app, Term, TermDag, TermId};
use ordered_float::OrderedFloat;

pub(crate) struct TermConverter<'a> {
    optimizer: &'a mut Optimizer,
    termdag: &'a TermDag,
}

impl TermConverter<'_> {
    pub(crate) fn get(&self, id: &TermId) -> Term {
        self.termdag.get(*id)
    }

    pub(crate) fn term_to_structured_func(&mut self, id: &TermId) -> StructuredFunction {
        match_term_app!(self.get(id); {
            ("Func", [func_name, argslist, body]) => {
                let args = self.term_conslist_to_vec(argslist, "Arg")
                    .into_iter()
                    .map(|arg| self.term_to_argument(&arg))
                    .collect();

                let fname = self.string_term_to_string(func_name);
                StructuredFunction {
                    name: fname.to_string(),
                    args,
                    block: self.term_to_structured_block(body),
                }
            }
            (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
        })
    }

    fn term_to_argument(&self, id: &TermId) -> Argument {
        match_term_app!(self.get(id); {
            ("Arg", [name, ty]) => {
                let name = self.string_term_to_string(name);
                Argument {
                    name: name.to_string(),
                    arg_type: self.term_to_type(ty),
                }
            }
            (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
        })
    }

    pub(crate) fn term_to_structured_block(&mut self, id: &TermId) -> StructuredBlock {
        match_term_app!(self.get(id); {
            ("Block", [block]) => {
                StructuredBlock::Block(Box::new(self.term_to_structured_block(block)))
            },
            ("Basic", [basic_block]) => {
                StructuredBlock::Basic(Box::new(self.term_to_basic_block(basic_block)))
            },
            ("Ite", [name, then_branch, else_branch]) => {
                let string = self.string_term_to_string(name);
                StructuredBlock::Ite(
                    string.to_string(),
                    Box::new(self.term_to_structured_block(then_branch)),
                    Box::new(self.term_to_structured_block(else_branch)),
                )
            },
            ("Loop", [block]) => {
                StructuredBlock::Loop(Box::new(self.term_to_structured_block(block)))
            },
            ("Sequence", [block, rest]) => StructuredBlock::Sequence(vec![
                self.term_to_structured_block(block),
                self.term_to_structured_block(rest),
            ]),
            ("Break", [n]) => {
                if let Term::Lit(egglog::ast::Literal::Int(n)) = self.get(n) {
                    StructuredBlock::Break(n.try_into().unwrap())
                } else {
                    panic!("expected int literal for break");
                }
            },
            ("Return", [val]) => {
                match_term_app!(self.get(val); {
                    ("Void", _) => StructuredBlock::Return(None),
                    ("ReturnValue", [arg]) => {
                        match self.get(arg) {
                            Term::Lit(egglog::ast::Literal::String(s)) => {
                                StructuredBlock::Return(Some(s.to_string()))
                            }
                            _ => panic!("expected string literal for return value"),
                        }
                    }
                    (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
                })
            }
            (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
        })
    }

    pub(crate) fn term_to_basic_block(&mut self, id: &TermId) -> BasicBlock {
        match_term_app!(self.get(id); {
            ("BlockNamed", [name, code]) => {
                let name = self.string_term_to_string(name);
                let code_vec = self.term_conslist_to_vec(code, "Code");
                let mut instrs = vec![];
                let mut memo = HashMap::<TermId, String>::new();

                for t in code_vec {
                    self.term_to_instructions(&t, &mut instrs, &mut memo);
                }

                BasicBlock {
                    name: BlockName::Named(name.to_string()),
                    footer: Default::default(),
                    instrs,
                    pos: None,
                }
            }
            (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
        })
    }

    fn term_conslist_to_vec_helper(&self, id: &TermId, res: &mut Vec<TermId>, prefix: &str) {
        match_term_app!(self.get(id); {
            (op, [head, tail]) if op == prefix.to_string() + "Cons" => {
                res.push(*head);
                self.term_conslist_to_vec_helper(tail, res, prefix);
            },
            (op, []) if op == prefix.to_string() + "Nil" => {}
            (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
        })
    }

    fn term_conslist_to_vec(&self, id: &TermId, prefix: &str) -> Vec<TermId> {
        let mut res = vec![];
        self.term_conslist_to_vec_helper(id, &mut res, prefix);
        res
    }

    fn term_to_instructions(
        &mut self,
        id: &TermId,
        res: &mut Vec<Instruction>,
        memo: &mut HashMap<TermId, String>,
    ) {
        match_term_app!(self.get(id); {
            ("Print", [arg]) => {
                let arg = self.term_to_code(arg, res, None, memo);

                res.push(Instruction::Effect {
                    op: EffectOps::Print,
                    args: vec![arg],
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                });
            },
            ("End", []) => {},
            ("Assign", [dest, src]) => {
                let dest = self.string_term_to_string(dest);
                self.term_to_code(src, res, Some(dest.to_string()), memo);
            },
            (op @ ("store" | "free"), args) => {
                let args = args
                    .iter()
                    .map(|arg| self.term_to_code(arg, res, None, memo))
                    .collect::<Vec<String>>();


                res.push(Instruction::Effect {
                    op: serde_json::from_str(&format!("\"{}\"", op)).unwrap(),
                    args,
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                });
            },
            ("alloc", [atype, dest, arg]) => {
                let atype = self.term_to_type(atype);
                let dest = self.string_term_to_string(dest);
                let arg = self.term_to_code(arg, res, None, memo);
                res.push(Instruction::Value {
                    dest,
                    args: vec![arg],
                    funcs: vec![],
                    op: ValueOps::Alloc,
                    labels: vec![],
                    pos: None,
                    op_type: atype,
                });
            },
            (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
        })
    }

    pub(crate) fn term_to_code(
        &mut self,
        id: &TermId,
        res: &mut Vec<Instruction>,
        assign_to: Option<String>,
        memo: &mut HashMap<TermId, String>,
    ) -> String {
        if memo.contains_key(id) && assign_to.is_none() {
            return memo[id].clone();
        }

        let dest = match &assign_to {
            Some(dest) => dest.clone(),
            None => self.optimizer.fresh_var(),
        };

        let ret = match self.get(id) {
            Term::Lit(literal) => {
                res.push(Instruction::Constant {
                    dest: dest.clone(),
                    op: bril_rs::ConstOps::Const,
                    value: self.optimizer.literal_to_bril(&literal),
                    pos: None,
                    const_type: self.optimizer.literal_to_type(&literal),
                });
                dest
            }
            t => {
                match_term_app!(t; {
                    ("Var", [arg]) => {
                        match self.get(arg) {
                            Term::Lit(egglog::ast::Literal::String(var)) => var.to_string(),
                            _ => panic!("expected string literal for var"),
                        }
                    },
                    ("ReturnValue", [arg]) => self.term_to_code(arg, res, assign_to, memo),
                    (op @ ("True" | "False" | "Int" | "Float" | "Char"), [ty, args @ ..]) => {
                        let lit = match (op, args) {
                            ("True", []) => Literal::Bool(true),
                            ("False", []) => Literal::Bool(false),
                            ("Int", [arg]) => {
                                let arg = self.get(arg);
                                let arg_s = self.termdag.to_string(&arg);
                                Literal::Int(arg_s.parse::<i64>().unwrap())
                            }
                            ("Float", [arg]) => {
                                let arg = self.get(arg);
                                let arg_s = self.termdag.to_string(&arg);
                                Literal::Float(arg_s.parse::<f64>().unwrap())
                            }
                            ("Char", [arg]) => {
                                let arg = self.get(arg);
                                let arg_s = self.termdag.to_string(&arg);
                                assert_eq!(arg_s.len(), 1);
                                Literal::Char(arg_s.chars().next().unwrap())
                            }
                            _ => panic!("unexpected args to literal in term_to_code")
                        };
                        res.push(Instruction::Constant {
                            dest: dest.clone(),
                            op: bril_rs::ConstOps::Const,
                            value: lit,
                            pos: None,
                            const_type: self.term_to_type(ty),
                        });
                        dest
                    },
                    ("phi", [etype, arg1, arg2, label1, label2]) => {
                        let etype = self.term_to_type(etype);
                        let arg1 = self.term_to_code(arg1, res, None, memo);
                        let arg2 = self.term_to_code(arg2, res, None, memo);
                        let label1 = self.string_term_to_string(label1);
                        let label2 = self.string_term_to_string(label2);
                        res.push(Instruction::Value {
                            dest: dest.clone(),
                            args: vec![arg1, arg2],
                            funcs: vec![],
                            op: ValueOps::Phi,
                            labels: vec![label1, label2],
                            pos: None,
                            op_type: etype,
                        });
                        dest
                    },
                    (op, args) => {
                        assert!(op != "Void");
                        let etype = self.term_to_type(&args[0]);
                        let args_vars = args
                            .iter()
                            .skip(1)
                            .map(|arg| self.term_to_code(arg, res, None, memo))
                            .collect::<Vec<String>>();
                        res.push(Instruction::Value {
                            dest: dest.clone(),
                            args: args_vars,
                            funcs: vec![],
                            op: egglog_op_to_bril(op.into()),
                            labels: vec![],
                            pos: None,
                            op_type: etype,
                        });
                        dest
                    }
                })
            }
        };

        memo.insert(*id, ret.clone());
        ret
    }

    pub(crate) fn term_to_type(&self, id: &TermId) -> Type {
        match_term_app!(self.get(id); {
            ("IntT", []) => Type::Int,
            ("BoolT", []) => Type::Bool,
            ("FloatT", []) => Type::Float,
            ("CharT", []) => Type::Char,
            ("PointerT", [child]) => Type::Pointer(Box::new(self.term_to_type(child))),
            (head, _) => panic!("unexpected head {}, in {}:{}:{}", head, file!(), line!(), column!())
        })
    }

    fn string_term_to_string(&self, id: &TermId) -> String {
        if let Term::Lit(egglog::ast::Literal::String(string)) = self.get(id) {
            string.to_string()
        } else {
            panic!("expected string literal");
        }
    }
}

impl Optimizer {
    pub(crate) fn term_to_structured_func(
        &mut self,
        termdag: &TermDag,
        term: &Term,
    ) -> StructuredFunction {
        let mut converter = TermConverter {
            optimizer: self,
            termdag,
        };
        converter.term_to_structured_func(&termdag.lookup(term))
    }

    pub(crate) fn func_to_expr(&mut self, func: &StructuredFunction) -> Expr {
        let arg_exprs = func
            .args
            .iter()
            .map(Self::argument_to_expr)
            .collect::<Vec<Expr>>();
        let arg_expr = Self::vec_to_cons_list(arg_exprs, "Arg");
        Expr::Call(
            "Func".into(),
            vec![
                Expr::Lit(egglog::ast::Literal::String(func.name.clone().into())),
                arg_expr,
                self.structured_block_to_expr(&func.block),
            ],
        )
    }

    fn vec_to_cons_list(vec: Vec<Expr>, prefix: &str) -> Expr {
        let mut current = Expr::Call(format!("{prefix}Nil").into(), vec![]);
        for expr in vec.into_iter().rev() {
            current = Expr::Call(format!("{prefix}Cons").into(), vec![expr, current]);
        }
        current
    }

    fn argument_to_expr(arg: &Argument) -> Expr {
        Expr::Call(
            "Arg".into(),
            vec![
                Expr::Lit(egglog::ast::Literal::String(arg.name.clone().into())),
                Self::type_to_expr(&arg.arg_type),
            ],
        )
    }

    pub(crate) fn structured_block_to_expr(&mut self, structured_block: &StructuredBlock) -> Expr {
        match structured_block {
            StructuredBlock::Ite(var, then, els) => Expr::Call(
                "Ite".into(),
                vec![
                    self.string_to_expr(var.clone()),
                    self.structured_block_to_expr(then),
                    self.structured_block_to_expr(els),
                ],
            ),
            StructuredBlock::Loop(body) => {
                Expr::Call("Loop".into(), vec![self.structured_block_to_expr(body)])
            }
            StructuredBlock::Block(body) => {
                Expr::Call("Block".into(), vec![self.structured_block_to_expr(body)])
            }
            StructuredBlock::Sequence(blocks) => match &blocks.as_slice() {
                [] => panic!("empty sequence"),
                [a] => self.structured_block_to_expr(a),
                [a, ..] => Expr::Call(
                    "Sequence".into(),
                    vec![
                        self.structured_block_to_expr(a),
                        self.structured_block_to_expr(&StructuredBlock::Sequence(
                            blocks[1..].to_vec(),
                        )),
                    ],
                ),
            },
            StructuredBlock::Break(n) => Expr::Call(
                "Break".into(),
                vec![Expr::Lit(egglog::ast::Literal::Int(
                    (*n).try_into().unwrap(),
                ))],
            ),
            StructuredBlock::Return(val) => Expr::Call(
                "Return".into(),
                vec![match val {
                    Some(val) => {
                        Expr::Call("ReturnValue".into(), vec![self.string_to_expr(val.clone())])
                    }
                    None => Expr::Call("Void".into(), vec![]),
                }],
            ),
            StructuredBlock::Basic(block) => {
                Expr::Call("Basic".into(), vec![self.convert_basic_block(block)])
            }
        }
    }

    pub(crate) fn string_to_expr(&self, string: String) -> Expr {
        Expr::Lit(egglog::ast::Literal::String(string.into()))
    }

    pub(crate) fn string_to_var_encoding(&self, string: String) -> Expr {
        Expr::Call("Var".into(), vec![self.string_to_expr(string)])
    }

    pub(crate) fn convert_basic_block(&mut self, block: &BasicBlock) -> Expr {
        // leave prints in order
        // leave any effects in order
        // leave assignments to variables used outside
        // of this function
        // otherwise inline
        let mut env = HashMap::<String, Expr>::new();
        let codelist = block
            .instrs
            .iter()
            .map(|instr| self.instr_to_code_expr(instr, &mut env))
            .collect();

        Expr::Call(
            "BlockNamed".into(),
            vec![
                Expr::Lit(egglog::ast::Literal::String(block.name.to_string().into())),
                Self::vec_to_cons_list(codelist, "Code"),
            ],
        )
    }

    pub(crate) fn instr_to_code_expr(
        &mut self,
        instr: &Instruction,
        env: &mut HashMap<String, Expr>,
    ) -> Expr {
        let (dest, expr) = match instr {
            Instruction::Constant {
                dest,
                op: _op,
                value,
                pos: _pos,
                const_type,
            } => (
                dest.clone(),
                match value {
                    Literal::Int(int) => Expr::Call(
                        "Int".into(),
                        vec![
                            Self::type_to_expr(const_type),
                            Expr::Lit(egglog::ast::Literal::Int(*int)),
                        ],
                    ),
                    Literal::Bool(bool) => {
                        if *bool {
                            Expr::Call("True".into(), vec![Self::type_to_expr(const_type)])
                        } else {
                            Expr::Call("False".into(), vec![Self::type_to_expr(const_type)])
                        }
                    }
                    Literal::Char(char) => Expr::Call(
                        "Char".into(),
                        vec![
                            Self::type_to_expr(const_type),
                            Expr::Lit(egglog::ast::Literal::String(char.to_string().into())),
                        ],
                    ),
                    Literal::Float(float) => Expr::Call(
                        "Float".into(),
                        vec![
                            Self::type_to_expr(const_type),
                            Expr::Lit(egglog::ast::Literal::F64(OrderedFloat(*float))),
                        ],
                    ),
                },
            ),
            // Allocation is actually an effect, so handle it first
            Instruction::Value {
                op: ValueOps::Alloc,
                args,
                dest,
                op_type,
                ..
            } => {
                assert!(args.len() == 1);
                let arg = env
                    .get(&args[0])
                    .cloned()
                    .unwrap_or(self.string_to_var_encoding(args[0].clone()));
                let atype = Self::type_to_expr(op_type);
                return Expr::Call(
                    "alloc".into(),
                    vec![atype, self.string_to_expr(dest.to_string()), arg.clone()],
                );
            }
            Instruction::Effect {
                op,
                args,
                funcs: _funcs,
                labels: _labels,
                pos: _pos,
            } => {
                let arg_exprs = match op {
                    EffectOps::Return => match &args.as_slice() {
                        [] => vec![Expr::Call("Void".into(), vec![])],
                        [arg] => vec![Expr::Call(
                            "ReturnValue".into(),
                            vec![env.get(arg).cloned().unwrap_or(Expr::Var(arg.into()))],
                        )],
                        _ => panic!("expected 1 arg for return"),
                    },
                    _ => args
                        .iter()
                        .map(|arg| {
                            env.get(arg)
                                .unwrap_or(&self.string_to_var_encoding(arg.to_string()))
                                .clone()
                        })
                        .collect::<Vec<Expr>>(),
                }
                .into_iter()
                .collect::<Vec<Expr>>();
                return Expr::Call(self.effect_op_to_egglog(*op), arg_exprs);
            }
            Instruction::Value {
                dest,
                args,
                funcs,
                op,
                labels,
                pos: _pos,
                op_type,
            } => {
                // Funcs should be empty when it's a constant
                // in valid Bril code
                assert!(funcs.is_empty());
                let mut arg_exprs = once(Self::type_to_expr(op_type))
                    .chain(args.iter().map(|arg| {
                        env.get(arg)
                            .unwrap_or(&self.string_to_var_encoding(arg.to_string()))
                            .clone()
                    }))
                    .collect::<Vec<Expr>>();
                let label_exprs = labels
                    .iter()
                    .map(|label| self.string_to_expr(label.to_string()))
                    .collect::<Vec<Expr>>();
                assert!(label_exprs.is_empty() || op == &ValueOps::Phi);
                arg_exprs.extend(label_exprs);
                let expr = Expr::Call(self.op_to_egglog(*op), arg_exprs);
                (dest.clone(), expr)
            }
        };

        env.insert(dest.clone(), expr.clone());

        Expr::Call("Assign".into(), vec![self.string_to_expr(dest), expr])
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
            egglog::ast::Literal::Bool(bool) => Literal::Bool(*bool),
            egglog::ast::Literal::F64(float) => Literal::Float(float.into_inner()),
            egglog::ast::Literal::Unit => panic!("unit literal not supported"),
        }
    }

    pub(crate) fn literal_to_type(&self, literal: &egglog::ast::Literal) -> bril_rs::Type {
        match literal {
            egglog::ast::Literal::Int(_) => bril_rs::Type::Int,
            egglog::ast::Literal::String(_) => bril_rs::Type::Int,
            egglog::ast::Literal::F64(_) => bril_rs::Type::Float,
            egglog::ast::Literal::Bool(_) => bril_rs::Type::Bool,
            egglog::ast::Literal::Unit => panic!("unit literal not supported"),
        }
    }

    pub(crate) fn type_to_expr(ty: &Type) -> egglog::ast::Expr {
        match ty {
            Type::Int => Expr::Call("IntT".into(), vec![]),
            Type::Bool => Expr::Call("BoolT".into(), vec![]),
            Type::Float => Expr::Call("FloatT".into(), vec![]),
            Type::Char => Expr::Call("CharT".into(), vec![]),
            Type::Pointer(child) => Expr::Call("PointerT".into(), vec![Self::type_to_expr(child)]),
        }
    }

    pub(crate) fn pretty_print_expr(expr: &Expr) -> String {
        Self::pretty_print_expr_with_acc(expr, 0)
    }
    pub(crate) fn pretty_print_expr_with_acc(expr: &Expr, indent: usize) -> String {
        let indent_str = " ".repeat(indent * 2);
        match expr {
            Expr::Lit(lit) => format!("{}{}", indent_str, lit),
            Expr::Var(var) => format!("{}{}", indent_str, var),
            Expr::Call(op, args) => match args.as_slice() {
                [] => {
                    format!("{}({})", indent_str, op)
                }
                _ if args
                    .iter()
                    .all(|v| matches!(v, Expr::Var(..) | Expr::Lit(..))) =>
                {
                    let args_str = args
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("{}({} {})", indent_str, op, args_str)
                }
                [Expr::Var(..) | Expr::Lit(..), ..] => {
                    let args_str = args
                        .iter()
                        .skip(1)
                        .map(|arg| Self::pretty_print_expr_with_acc(arg, indent + 1))
                        .collect::<Vec<String>>()
                        .join("\n");
                    format!("{}({} {}\n{})", indent_str, op, args[0], args_str)
                }
                _ => {
                    let args_str = args
                        .iter()
                        .map(|arg| Self::pretty_print_expr_with_acc(arg, indent + 1))
                        .collect::<Vec<String>>()
                        .join("\n");
                    format!("{}({}\n{})", indent_str, op, args_str)
                }
            },
        }
    }

    /// The bril to_ssa script generates __undefined variables
    /// whenever a variable is used before it is defined in a phi node.
    /// We reject these programs because it means the variable was not defined
    /// in all control flow paths to the phi node.
    pub fn check_for_uninitialized_vars(prog: &Program) -> Result<(), EggCCError> {
        for func in &prog.functions {
            for instr in &func.instrs {
                if let Code::Instruction(Instruction::Value {
                    dest: _,
                    args,
                    funcs: _funcs,
                    op: ValueOps::Phi,
                    labels: _labels,
                    pos: _pos,
                    op_type: _op_type,
                }) = instr
                {
                    assert!(args.len() == 2);
                    if args[0] == "__undefined" {
                        return Err(EggCCError::UninitializedVariable(
                            args[1].clone(),
                            func.name.clone(),
                        ));
                    } else if args[1] == "__undefined" {
                        return Err(EggCCError::UninitializedVariable(
                            args[0].clone(),
                            func.name.clone(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn egglog_op_to_bril(op: Symbol) -> ValueOps {
    // remove b for bril
    // operators like "not" and "and" collide with egglog's
    // "not" and "and" operators

    let mut b_removed = op.to_string();
    assert!(b_removed.starts_with('b'));
    b_removed.remove(0);

    let with_quotes = "\"".to_owned() + &b_removed + "\"";
    serde_json::from_str(&with_quotes).unwrap()
}

pub(crate) fn op_to_egglog(op: ValueOps) -> Symbol {
    // add a b for bril
    // operators like "not" and "and" collide with egglog's
    // "not" and "and" operators

    let str = "b".to_string() + &op.to_string();
    str.into()
}
