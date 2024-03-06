//! Convert tree programs to RVSDGs

use std::{iter, rc::Rc};

use bril_rs::{ConstOps, EffectOps, Literal, ValueOps};
use tree_in_context::{
    schema::{BaseType, BinaryOp, Expr, Order, RcExpr, TreeProgram, Type, UnaryOp},
    typechecker::TypeCache,
};

use super::{BasicExpr, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};

type Operands = Vec<Operand>;

struct TreeToRvsdg<'a> {
    program: &'a TreeProgram,
    type_cache: &'a TypeCache,
    nodes: &'a mut Vec<RvsdgBody>,
    current_state_edge: Operand,
    current_args: Vec<Operand>,
}

pub(crate) fn tree_to_rvsdg(tree: &TreeProgram) -> RvsdgProgram {
    let mut res = RvsdgProgram { functions: vec![] };
    for func in &tree.functions {
        res.functions.push(tree_func_to_rvsdg(func.clone(), tree));
    }
    res.functions
        .push(tree_func_to_rvsdg(tree.entry.clone(), tree));
    res
}

fn bril_type_from_type(ty: Type) -> bril_rs::Type {
    match ty {
        Type::Base(base_ty) => match base_ty {
            BaseType::IntT => bril_rs::Type::Int,
            BaseType::BoolT => bril_rs::Type::Bool,
        },
        Type::PointerT(ty) => {
            let base_ty = bril_type_from_type(Type::Base(ty));
            bril_rs::Type::Pointer(Box::new(base_ty))
        }
        Type::TupleT(_) => panic!("Tuple types not supported in RVSDG"),
        Type::Unknown => panic!("Unknown type in tree_type_to_rvsdg_types"),
    }
}

fn convert_func_output_type(ty: Type) -> Option<bril_rs::Type> {
    eprintln!("Converting {:?}", ty);
    match ty {
        Type::TupleT(inner) => {
            assert_eq!(
                inner.len(),
                0,
                "Expected no tuple types in function_type_from_type"
            );
            None
        }
        _ => Some(bril_type_from_type(ty)),
    }
}

fn tree_func_to_rvsdg(func: RcExpr, program: &TreeProgram) -> RvsdgFunction {
    let output_type = func.func_output_ty().expect("Expected function types");

    let Type::TupleT(input_types) = func.func_input_ty().expect("Expected function types") else {
        panic!("Expected tuple type for inputs in tree_func_to_rvsdg")
    };

    let mut nodes = vec![];
    let (typechecked_program, type_cache) = program.with_arg_types_and_cache();
    // initial state edge is last argument
    let mut converter = TreeToRvsdg {
        program: &typechecked_program,
        type_cache: &type_cache,
        nodes: &mut nodes,
        current_state_edge: Operand::Arg(input_types.len()),
        current_args: (0..input_types.len()).map(Operand::Arg).collect(),
    };

    let converted = converter.convert_expr(func.clone());

    let resulting_state_edge = converter.current_state_edge;
    drop(converter);
    RvsdgFunction {
        name: func
            .func_name()
            .expect("Expected function in tree_func_to_rvsdg"),
        // normal types and a state edge at the end
        args: input_types
            .into_iter()
            .map(|ty| RvsdgType::Bril(bril_type_from_type(ty)))
            .chain(iter::once(RvsdgType::PrintState))
            .collect(),
        nodes,
        results: match convert_func_output_type(output_type) {
            Some(func_type) => {
                assert!(converted.len() == 1, "Expected exactly one result");
                vec![
                    (RvsdgType::Bril(func_type), converted[0]),
                    (RvsdgType::PrintState, resulting_state_edge),
                ]
            }
            None => {
                assert!(
                    converted.is_empty(),
                    "Expected no results. Got {:?}",
                    converted
                );
                vec![(RvsdgType::PrintState, resulting_state_edge)]
            }
        },
    }
}

fn value_op_from_binary_op(bop: BinaryOp) -> Option<ValueOps> {
    match bop {
        BinaryOp::Add => Some(ValueOps::Add),
        BinaryOp::Sub => Some(ValueOps::Sub),
        BinaryOp::Mul => Some(ValueOps::Mul),
        BinaryOp::Div => Some(ValueOps::Div),
        BinaryOp::And => Some(ValueOps::And),
        BinaryOp::Or => Some(ValueOps::Or),
        BinaryOp::Eq => Some(ValueOps::Eq),
        BinaryOp::LessThan => Some(ValueOps::Lt),
        BinaryOp::GreaterThan => Some(ValueOps::Gt),
        BinaryOp::Write => None,
        BinaryOp::PtrAdd => Some(ValueOps::PtrAdd),
    }
}

fn effect_op_from_binary_op(bop: BinaryOp) -> Option<EffectOps> {
    match bop {
        BinaryOp::Write => Some(EffectOps::Store),
        _ => None,
    }
}

fn value_op_from_unary_op(uop: UnaryOp) -> Option<ValueOps> {
    match uop {
        UnaryOp::Not => Some(ValueOps::Not),
        UnaryOp::Print => None,
        UnaryOp::Load => Some(ValueOps::Load),
    }
}

fn effect_op_from_unary_op(uop: UnaryOp) -> Option<EffectOps> {
    match uop {
        UnaryOp::Not => None,
        UnaryOp::Print => Some(EffectOps::Print),
        UnaryOp::Load => None,
    }
}

impl<'a> TreeToRvsdg<'a> {
    fn args_with_state_edge(&self) -> Vec<Operand> {
        self.current_args
            .iter()
            .cloned()
            .chain(iter::once(self.current_state_edge))
            .collect()
    }

    /// Translates an expression in a new subregion
    /// num_args is the number of non-state-edge arguments
    fn translate_subregion(&mut self, expr: RcExpr, num_args: usize) -> Vec<Operand> {
        let inner_args = (0..num_args).map(Operand::Arg).collect();
        let mut translator = TreeToRvsdg {
            program: self.program,
            nodes: self.nodes,
            type_cache: self.type_cache,
            current_state_edge: Operand::Arg(num_args),
            current_args: inner_args,
        };
        let mut results = translator.convert_expr(expr);
        results.push(translator.current_state_edge);
        results
    }

    fn push_basic(&mut self, mut basic: BasicExpr<Operand>) -> Vec<Operand> {
        match &basic {
            BasicExpr::Effect(..) => {
                let new_id = self.nodes.len();
                basic.push_operand(self.current_state_edge);
                self.nodes.push(RvsdgBody::BasicOp(basic));
                self.current_state_edge = Operand::Project(0, new_id);
                vec![]
            }
            BasicExpr::Op(ValueOps::Alloc | ValueOps::Load, _, _) | BasicExpr::Call(..) => {
                let new_id = self.nodes.len();
                basic.push_operand(self.current_state_edge);
                self.nodes.push(RvsdgBody::BasicOp(basic));
                self.current_state_edge = Operand::Project(1, new_id);
                vec![Operand::Project(0, new_id)]
            }
            BasicExpr::Op(..) | BasicExpr::Const(..) => {
                let new_id = self.nodes.len();
                self.nodes.push(RvsdgBody::BasicOp(basic));
                vec![Operand::Project(0, new_id)]
            }
        }
    }

    fn convert_expr(&mut self, expr: RcExpr) -> Operands {
        eprintln!("Converting {:?}", expr);
        let res = match expr.as_ref() {
            Expr::Function(_name, _inty, _outty, expr) => self.convert_expr(expr.clone()),
            Expr::Const(constant, _ty) => match constant {
                tree_in_context::schema::Constant::Int(integer) => self.push_basic(
                    BasicExpr::Const(ConstOps::Const, Literal::Int(*integer), bril_rs::Type::Int),
                ),
                tree_in_context::schema::Constant::Bool(boolean) => {
                    self.push_basic(BasicExpr::Const(
                        ConstOps::Const,
                        Literal::Bool(*boolean),
                        bril_rs::Type::Bool,
                    ))
                }
            },
            Expr::Bop(op, l, r) => {
                let l = self.convert_expr(l.clone());
                let r = self.convert_expr(r.clone());
                assert_eq!(l.len(), 1, "Expected exactly one result for left operand");
                assert_eq!(r.len(), 1, "Expected exactly one result for right operand");
                let l = l[0];
                let r = r[0];
                if let Some(vop) = value_op_from_binary_op(op.clone()) {
                    self.push_basic(BasicExpr::Op(
                        vop,
                        vec![l, r],
                        bril_type_from_type(
                            self.type_cache
                                .get(&Rc::as_ptr(&expr))
                                .expect("Expected type for expression")
                                .clone(),
                        ),
                    ))
                } else if let Some(eop) = effect_op_from_binary_op(op.clone()) {
                    self.push_basic(BasicExpr::Effect(eop, vec![l, r]))
                } else {
                    panic!("Unknown binary op {:?}", op)
                }
            }
            Expr::Uop(op, child) => {
                let child = self.convert_expr(child.clone());
                assert_eq!(child.len(), 1, "Expected exactly one result for child");
                let child = child[0];
                if let Some(vop) = value_op_from_unary_op(op.clone()) {
                    self.push_basic(BasicExpr::Op(vop, vec![child], bril_rs::Type::Int))
                } else if let Some(eop) = effect_op_from_unary_op(op.clone()) {
                    self.push_basic(BasicExpr::Effect(eop, vec![child]))
                } else {
                    panic!("Unknown unary op {:?}", op)
                }
            }
            Expr::Get(child, index) => {
                let child = self.convert_expr(child.clone());
                assert!(
                    child.len() > *index,
                    "Index out of bounds. Got child {:?} with index {:?}",
                    child,
                    index
                );
                vec![child[*index]]
            }
            Expr::Alloc(size, ty) => {
                let size = self.convert_expr(size.clone());
                assert_eq!(size.len(), 1, "Expected exactly one result for size");
                let size = size[0];
                self.push_basic(BasicExpr::Op(
                    ValueOps::Alloc,
                    vec![size],
                    bril_type_from_type(ty.clone()),
                ))
            }
            Expr::Arg(_ty) => self.current_args.clone(),
            Expr::Call(name, args) => {
                let func = self.program.get_function(name).expect("Function not found");
                let func_ty = convert_func_output_type(
                    func.func_output_ty().expect("Expected function types"),
                );
                let args = self.convert_expr(args.clone());
                let num_results = func_ty.is_some() as usize + 1;
                self.push_basic(BasicExpr::Call(name.clone(), args, num_results, func_ty))
            }
            Expr::Empty(_ty) => {
                vec![]
            }
            Expr::Let(input, body) => {
                let input = self.convert_expr(input.clone());
                self.current_args = input.clone();
                self.convert_expr(body.clone())
            }
            Expr::InContext(_assum, body) => self.convert_expr(body.clone()),
            Expr::Concat(order, left, right) => match order {
                Order::Parallel | Order::Sequential => {
                    let left = self.convert_expr(left.clone());
                    let right = self.convert_expr(right.clone());
                    left.into_iter().chain(right).collect()
                }
                Order::Reversed => {
                    let left = self.convert_expr(left.clone());
                    let right = self.convert_expr(right.clone());
                    right.into_iter().chain(left).collect()
                }
            },
            Expr::If(pred, then_branch, else_branch) => {
                let pred = self.convert_expr(pred.clone());
                assert_eq!(pred.len(), 1, "Expected exactly one result for predicate");
                let then_branch =
                    self.translate_subregion(then_branch.clone(), self.current_args.len());

                let else_branch =
                    self.translate_subregion(else_branch.clone(), self.current_args.len());

                let new_id = self.nodes.len();
                assert_eq!(
                    then_branch.len(),
                    else_branch.len(),
                    "Expected same number of values for then and else branches"
                );
                let args_and_state_edge = self
                    .current_args
                    .iter()
                    .cloned()
                    .chain(iter::once(self.current_state_edge))
                    .collect();

                self.current_state_edge = Operand::Project(then_branch.len() - 1, new_id);
                let res = (0..(then_branch.len() - 1))
                    .map(|i| Operand::Project(i, new_id))
                    .collect();
                self.nodes.push(RvsdgBody::If {
                    pred: pred[0],
                    inputs: args_and_state_edge,
                    then_branch,
                    else_branch,
                });

                res
            }
            Expr::Switch(pred, cases) => {
                let pred = self.convert_expr(pred.clone());
                assert_eq!(pred.len(), 1, "Expected exactly one result for predicate");
                let mut outputs = vec![];
                for case in cases {
                    let case = self.translate_subregion(case.clone(), self.current_args.len());
                    outputs.push(case);
                }
                assert!(
                    !outputs.is_empty(),
                    "Expected at least one case for switch statement"
                );
                let new_id = self.nodes.len();
                let res = (0..outputs[0].len())
                    .map(|i| Operand::Project(i, new_id))
                    .collect();
                self.current_state_edge = Operand::Project(outputs[0].len() - 1, new_id);
                self.nodes.push(RvsdgBody::Gamma {
                    pred: pred[0],
                    inputs: self.args_with_state_edge(),
                    outputs,
                });
                res
            }
            Expr::DoWhile(inputs, body) => {
                let mut inputs_with_state_edge = self.convert_expr(inputs.clone());
                inputs_with_state_edge.push(self.current_state_edge);
                let pred_and_body_and_state_edge =
                    self.translate_subregion(body.clone(), inputs_with_state_edge.len() - 1);
                assert_eq!(
                    inputs_with_state_edge.len(),
                    pred_and_body_and_state_edge.len() - 1,
                    "Expected matching number of inputs and outputs for do-while body"
                );
                let pred_inner = pred_and_body_and_state_edge[0];
                let body_and_state_edge = pred_and_body_and_state_edge[1..].to_vec();

                let new_id = self.nodes.len();
                self.current_state_edge = Operand::Project(body_and_state_edge.len() - 1, new_id);
                let res = (0..(body_and_state_edge.len() - 1))
                    .map(|i| Operand::Project(i, new_id))
                    .collect();
                self.nodes.push(RvsdgBody::Theta {
                    pred: pred_inner,
                    inputs: inputs_with_state_edge,
                    outputs: body_and_state_edge,
                });
                res
            }
            Expr::Single(body) => {
                let res = self.convert_expr(body.clone());
                assert_eq!(res.len(), 1, "Expected exactly one result for Single node");
                res
            }
        };
        eprintln!("Converted {:?} to {:?}", expr, res);
        res
    }
}
