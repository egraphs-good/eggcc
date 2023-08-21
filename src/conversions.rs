use std::{collections::HashMap, iter::once};

use crate::{
    cfg::{
        structured::{StructuredBlock, StructuredFunction},
        BasicBlock, BlockName,
    },
    Optimizer,
};
use bril_rs::{EffectOps, Instruction, Literal, Type, ValueOps};
use egglog::ast::{Expr, Symbol};
use ordered_float::OrderedFloat;

impl Optimizer {
    pub(crate) fn expr_to_structured_func(&mut self, expr: Expr) -> StructuredFunction {
        if let Expr::Call(func, args) = expr {
            assert_eq!(func.to_string(), "Func");
            match &args.as_slice() {
                [func_name, body] => {
                    if let Expr::Lit(egglog::ast::Literal::String(fname)) = func_name {
                        StructuredFunction {
                            name: fname.to_string(),
                            args: vec![],
                            block: self.expr_to_structured_block(body),
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

    pub(crate) fn expr_to_structured_block(&mut self, expr: &Expr) -> StructuredBlock {
        if let Expr::Call(func, args) = expr {
            match (func.as_str(), &args.as_slice()) {
                ("Block", [block]) => {
                    StructuredBlock::Block(Box::new(self.expr_to_structured_block(block)))
                }
                ("Basic", [basic_block]) => {
                    StructuredBlock::Basic(Box::new(self.expr_to_basic_block(basic_block)))
                }
                ("Ite", [name, then_branch, else_branch]) => StructuredBlock::Ite(
                    Self::string_expr_to_string(name),
                    Box::new(self.expr_to_structured_block(then_branch)),
                    Box::new(self.expr_to_structured_block(else_branch)),
                ),
                ("Loop", [block]) => {
                    StructuredBlock::Loop(Box::new(self.expr_to_structured_block(block)))
                }
                ("Sequence", [block, rest]) => StructuredBlock::Sequence(vec![
                    self.expr_to_structured_block(block),
                    self.expr_to_structured_block(rest),
                ]),
                ("Break", [n]) => {
                    if let Expr::Lit(egglog::ast::Literal::Int(n)) = n {
                        StructuredBlock::Break((*n).try_into().unwrap())
                    } else {
                        panic!("expected int literal for break");
                    }
                }
                ("Return", [val]) => {
                    if let Expr::Call(op, args) = val {
                        match op.as_str() {
                            "Void" => StructuredBlock::Return(None),
                            "ReturnValue" => {
                                assert_eq!(args.len(), 1);
                                match &args[0] {
                                    Expr::Lit(egglog::ast::Literal::String(val)) => {
                                        StructuredBlock::Return(Some(val.to_string()))
                                    }
                                    _ => panic!("expected string literal for return value"),
                                }
                            }
                            _ => panic!("expected void or return value"),
                        }
                    } else {
                        panic!("expected call for return");
                    }
                }
                _ => panic!("unknown structured block"),
            }
        } else {
            panic!("expected call in expr_to_structured_block");
        }
    }

    pub(crate) fn expr_to_basic_block(&mut self, expr: &Expr) -> BasicBlock {
        if let Expr::Call(op, args) = expr {
            assert_eq!(op.as_str(), "BlockNamed");

            match &args.as_slice() {
                [Expr::Lit(egglog::ast::Literal::String(name)), code] => {
                    let code_vec = self.codelist_to_vec(code);
                    let mut instrs = vec![];

                    for expr in code_vec {
                        self.expr_to_instructions(&expr, &mut instrs);
                    }

                    BasicBlock {
                        name: BlockName::Named(name.to_string()),
                        instrs,
                        pos: None,
                    }
                }
                _ => panic!("expected 2 args in expr_to_basic_block"),
            }
        } else {
            panic!("expected call in expr_to_basic_block");
        }
    }

    fn codelist_to_vec_helper(expr: &Expr, res: &mut Vec<Expr>) {
        match expr {
            Expr::Call(op, args) => match (op.as_str(), args.as_slice()) {
                ("CodeCons", [head, tail]) => {
                    res.push(head.clone());
                    Self::codelist_to_vec_helper(tail, res);
                }
                ("CodeNil", []) => {}
                _ => panic!("expected CodeCons or CodeNil"),
            },
            _ => panic!("expected call in codelist_to_vec"),
        }
    }

    fn codelist_to_vec(&self, expr: &Expr) -> Vec<Expr> {
        let mut res = vec![];
        Self::codelist_to_vec_helper(expr, &mut res);
        res
    }

    fn vec_to_codelist(&mut self, vec: Vec<Expr>) -> Expr {
        let mut current = Expr::Call("CodeNil".into(), vec![]);
        for expr in vec.into_iter().rev() {
            current = Expr::Call("CodeCons".into(), vec![expr, current]);
        }
        current
    }

    fn expr_to_instructions(&mut self, expr: &Expr, res: &mut Vec<Instruction>) {
        if let Expr::Call(op, args) = expr {
            match op.to_string().as_str() {
                "Print" => {
                    assert!(args.len() == 1);
                    let arg = self.expr_to_code(&args[0], res, None);

                    res.push(Instruction::Effect {
                        op: EffectOps::Print,
                        args: vec![arg],
                        funcs: vec![],
                        labels: vec![],
                        pos: None,
                    });
                }
                "End" => {
                    assert!(args.is_empty());
                }
                "Assign" => {
                    assert!(
                        args.len() == 2,
                        "expected 2 args in Assign. got: {:?}",
                        args
                    );
                    let dest = match &args[0] {
                        Expr::Lit(egglog::ast::Literal::String(dest)) => dest.to_string(),
                        _ => panic!("expected string literal for dest"),
                    };
                    self.expr_to_code(&args[1], res, Some(dest));
                }
                "store" | "free" => {
                    let args = args
                        .iter()
                        .map(|arg| self.expr_to_code(arg, res, None))
                        .collect::<Vec<String>>();

                    res.push(Instruction::Effect {
                        op: serde_json::from_str(&format!("\"{}\"", op)).unwrap(),
                        args,
                        funcs: vec![],
                        labels: vec![],
                        pos: None,
                    });
                }

                _ => panic!("unknown effect in body_to_code {}", op),
            }
        } else {
            panic!("expected call in body_to_code");
        }
    }

    // TODO memoize exprs for common subexpression elimination
    pub(crate) fn expr_to_code(
        &mut self,
        expr: &Expr,
        res: &mut Vec<Instruction>,
        assign_to: Option<String>,
    ) -> String {
        let dest = match &assign_to {
            Some(dest) => dest.clone(),
            None => self.fresh_var(),
        };
        match expr {
            Expr::Lit(literal) => {
                res.push(Instruction::Constant {
                    dest: dest.clone(),
                    op: bril_rs::ConstOps::Const,
                    value: self.literal_to_bril(literal),
                    pos: None,
                    const_type: self.literal_to_type(literal),
                });
                dest
            }
            Expr::Var(var) => {
                if let Some(_output) = assign_to {
                    panic!("Cannot assign var to var")
                } else {
                    var.to_string()
                }
            }
            Expr::Call(op, args) => match op.to_string().as_str() {
                "Var" => {
                    assert!(args.len() == 1);
                    match &args[0] {
                        Expr::Lit(egglog::ast::Literal::String(var)) => var.to_string(),
                        _ => panic!("expected string literal for var"),
                    }
                }
                "ReturnValue" => self.expr_to_code(&args[0], res, assign_to),
                "Int" | "True" | "False" => {
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
                    res.push(Instruction::Constant {
                        dest: dest.clone(),
                        op: bril_rs::ConstOps::Const,
                        value: literal,
                        pos: None,
                        const_type: self.expr_to_type(&args[0]),
                    });
                    dest
                }
                _ => {
                    assert!(op.as_str() != "Void");
                    let etype = self.expr_to_type(&args[0]);
                    let args_vars = args
                        .iter()
                        .skip(1)
                        .map(|arg| self.expr_to_code(arg, res, None))
                        .collect::<Vec<String>>();
                    res.push(Instruction::Value {
                        dest: dest.clone(),
                        args: args_vars,
                        funcs: vec![],
                        op: self.egglog_op_to_bril(*op),
                        labels: vec![],
                        pos: None,
                        op_type: etype,
                    });
                    dest
                }
            },
        }
    }

    pub(crate) fn func_to_expr(&mut self, func: &StructuredFunction) -> Expr {
        Expr::Call(
            "Func".into(),
            vec![
                Expr::Lit(egglog::ast::Literal::String(func.name.clone().into())),
                self.structured_block_to_expr(&func.block),
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

    fn string_expr_to_string(expr: &Expr) -> String {
        if let Expr::Lit(egglog::ast::Literal::String(string)) = expr {
            string.to_string()
        } else {
            panic!("expected string literal");
        }
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
                self.vec_to_codelist(codelist),
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
            Instruction::Value {
                dest,
                args,
                funcs,
                op,
                labels: _labels,
                pos: _pos,
                op_type,
            } => {
                // Funcs should be empty when it's a constant
                // in valid Bril code
                assert!(funcs.is_empty());
                let arg_exprs = once(Self::type_to_expr(op_type))
                    .chain(args.iter().map(|arg| {
                        env.get(arg)
                            .unwrap_or(&self.string_to_var_encoding(arg.to_string()))
                            .clone()
                    }))
                    .collect::<Vec<Expr>>();
                let expr = Expr::Call(self.op_to_egglog(*op), arg_exprs);
                (dest.clone(), expr)
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
        };

        env.insert(dest.clone(), expr.clone());

        Expr::Call("Assign".into(), vec![self.string_to_expr(dest), expr])
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

    pub(crate) fn type_to_expr(ty: &Type) -> egglog::ast::Expr {
        match ty {
            Type::Int => Expr::Call("IntT".into(), vec![]),
            Type::Bool => Expr::Call("BoolT".into(), vec![]),
            Type::Float => Expr::Call("FloatT".into(), vec![]),
            Type::Char => Expr::Call("CharT".into(), vec![]),
            Type::Pointer(child) => Expr::Call("PointerT".into(), vec![Self::type_to_expr(child)]),
        }
    }

    pub(crate) fn expr_to_type(&self, expr: &Expr) -> Type {
        let Expr::Call(op, args) = expr else {
            panic!("expected call in expr_to_type");
        };
        match (op.as_str(), args.as_slice()) {
            ("IntT", []) => Type::Int,
            ("BoolT", []) => Type::Bool,
            ("FloatT", []) => Type::Float,
            ("CharT", []) => Type::Char,
            ("PointerT", [child]) => Type::Pointer(Box::new(self.expr_to_type(child))),
            _ => panic!("unknown type"),
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
}
