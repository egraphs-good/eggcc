//! Convert tree programs to RVSDGs

use std::iter;

use bril_rs::{ConstOps, EffectOps, Literal, ValueOps};
use tree_in_context::schema::{BaseType, BinaryOp, Expr, RcExpr, TreeProgram, Type, UnaryOp};

use super::{BasicExpr, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};

type Operands = Vec<Operand>;

struct TreeToRvsdg<'a> {
    program: &'a TreeProgram,
    nodes: Vec<RvsdgBody>,
    current_state_edge: Operand,
    current_args: Vec<Operand>,
}

pub(crate) fn tree_to_rvsdg(tree: &TreeProgram) -> RvsdgProgram {
    let mut res = RvsdgProgram { functions: vec![] };
    for func in tree.functions {
        res.functions.push(tree_func_to_rvsdg(func, tree));
    }
    res.functions.push(tree_func_to_rvsdg(tree.entry, tree));
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

fn func_type_from_type(ty: Type) -> Option<bril_rs::Type> {
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

    // initial state edge is last argument
    let mut converter = TreeToRvsdg {
        program,
        nodes: vec![],
        current_state_edge: Operand::Arg(input_types.len()),
        current_args: (0..input_types.len()).map(|i| Operand::Arg(i)).collect(),
    };

    let converted = converter.convert_func(func.clone());

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
        nodes: converter.nodes,
        results: match func_type_from_type(output_type) {
            Some(func_type) => {
                assert!(converted.len() == 1, "Expected exactly one result");
                vec![
                    (RvsdgType::Bril(func_type), converted[0].clone()),
                    (RvsdgType::PrintState, converter.current_state_edge),
                ]
            }
            None => {
                assert!(converted.len() == 0, "Expected no results");
                vec![(RvsdgType::PrintState, converter.current_state_edge)]
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
    pub fn convert_func(&mut self, func: RcExpr) -> Vec<Operand> {
        todo!()
    }

    fn push_basic(&mut self, mut basic: BasicExpr<Operand>) -> Vec<Operand> {
        match &basic {
            BasicExpr::Effect(..)
            | BasicExpr::Op(ValueOps::Alloc | ValueOps::Load, _, _)
            | BasicExpr::Call(..) => {
                let new_id = self.nodes.len();
                basic.push_operand(self.current_state_edge.clone());
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
        match expr.as_ref() {
            Expr::Const(constant, ty) => match constant {
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
                let l = l[0].clone();
                let r = r[0].clone();
                if let Some(vop) = value_op_from_binary_op(*op) {
                    self.push_basic(BasicExpr::Op(vop, vec![l, r], bril_rs::Type::Int))
                } else if let Some(eop) = effect_op_from_binary_op(*op) {
                    self.push_basic(BasicExpr::Effect(eop, vec![l, r]))
                } else {
                    panic!("Unknown binary op {:?}", op)
                }
            }
            Expr::Uop(op, child) => {
                let child = self.convert_expr(child.clone());
                let child = child[0].clone();
                if let Some(vop) = value_op_from_unary_op(*op) {
                    self.push_basic(BasicExpr::Op(vop, vec![child], bril_rs::Type::Int))
                } else if let Some(eop) = effect_op_from_unary_op(*op) {
                    self.push_basic(BasicExpr::Effect(eop, vec![child]))
                } else {
                    panic!("Unknown unary op {:?}", op)
                }
            }
            Expr::Get(child, index) => {
                let child = self.convert_expr(child.clone());
                assert!(child.len() > *index, "Index out of bounds");
                vec![child[*index].clone()]
            }
            Expr::Alloc(size, ty) => {
                let size = self.convert_expr(size.clone());
                let size = size[0].clone();
                self.push_basic(BasicExpr::Op(
                    ValueOps::Alloc,
                    vec![size],
                    bril_type_from_type(ty.clone()),
                ))
            }
            Expr::Arg(ty) => self.current_args,
            Expr::Call(name, args) => {
                let func = self.program.get_function(name).expect("Function not found");
                let func_ty =
                    func_type_from_type(func.func_input_ty().expect("Expected function types"));
                let args = self.convert_expr(args.clone());
                let num_results = func_ty.is_some() as usize + 1;
                self.push_basic(BasicExpr::Call(name.clone(), args, num_results, func_ty))
            }
            Expr::Empty(ty) => {
                vec![]
            }
            Expr::Let(input, body) => {
                let input = self.convert_expr(input.clone());
                self.current_args = input.clone();
                self.convert_expr(body.clone())
            }
        }
    }
}
