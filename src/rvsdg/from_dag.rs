//! Convert tree programs to RVSDGs.
//! This is a strait-forward translation, since DAG programs are like RVSDGs
//! but with tuple constructs such as Concat.

use std::rc::Rc;

use bril_rs::{ConstOps, EffectOps, Literal, ValueOps};
use dag_in_context::{
    schema::{BaseType, BinaryOp, Expr, RcExpr, TernaryOp, TreeProgram, Type, UnaryOp},
    typechecker::TypeCache,
};
use indexmap::IndexMap;

use super::{BasicExpr, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};

type Operands = Vec<Operand>;

/// State needed to translate a tree program to
/// an RVSDG.
struct TreeToRvsdg<'a> {
    program: &'a TreeProgram,
    /// A cache of types for every expression in program
    type_cache: &'a TypeCache,
    /// A cache of already converted expressions.
    /// Shared expressions must be converted to the same RVSDG nodes.
    /// For branches, this can be pre-propulated with the arguments passed to the branch.
    translation_cache: IndexMap<*const Expr, Operands>,
    nodes: &'a mut Vec<RvsdgBody>,
    /// The current arguments to the tree program
    /// as RVSDG operands.
    current_args: Vec<Operand>,
}

pub(crate) fn dag_to_rvsdg(tree: &TreeProgram) -> RvsdgProgram {
    let mut res = RvsdgProgram { functions: vec![] };
    for func in &tree.functions {
        res.functions.push(tree_func_to_rvsdg(func.clone(), tree));
    }
    res.functions
        .push(tree_func_to_rvsdg(tree.entry.clone(), tree));
    res
}

fn basetype_to_bril_type(ty: BaseType) -> bril_rs::Type {
    match ty {
        BaseType::IntT => bril_rs::Type::Int,
        BaseType::FloatT => bril_rs::Type::Float,
        BaseType::BoolT => bril_rs::Type::Bool,
        BaseType::PointerT(inner) => {
            bril_rs::Type::Pointer(Box::new(basetype_to_bril_type(*inner)))
        }
        BaseType::StateT => panic!("State type not supported in bril"),
    }
}

fn basetype_to_rvsdg_type(ty: BaseType) -> RvsdgType {
    match ty {
        BaseType::StateT => RvsdgType::PrintState,
        _ => RvsdgType::Bril(basetype_to_bril_type(ty)),
    }
}

/// RVSDG functions support at most one return type, and at most one state edge.
/// This function returns the return type and if there is a state edge
fn convert_func_type(ty: Type) -> (Option<bril_rs::Type>, bool) {
    match ty {
        Type::TupleT(inner) => match inner.as_slice() {
            [BaseType::StateT] => (None, true),
            [some_ty, BaseType::StateT] => (Some(basetype_to_bril_type(some_ty.clone())), true),
            [other_ty] => (Some(basetype_to_bril_type(other_ty.clone())), false),
            _ => panic!("Expected one bril type and at most one state edge"),
        },
        _ => panic!("Expected tuple type for call type"),
    }
}

fn type_to_bril_type(ty: Type) -> Option<bril_rs::Type> {
    match ty {
        Type::TupleT(inner) => {
            assert!(
                inner.is_empty(),
                "Expected no tuple types in type_to_bril_type. Got: {:?}",
                inner
            );
            None
        }
        Type::Base(basetype) => Some(basetype_to_bril_type(basetype)),
        Type::Unknown => panic!("Expected known type in type_to_bril_type"),
        Type::Symbolic(_) => panic!("Symbolic not supported"),
    }
}

fn tree_func_to_rvsdg(func: RcExpr, program: &TreeProgram) -> RvsdgFunction {
    let func_name = func
        .func_name()
        .expect("Expected function in tree_func_to_rvsdg");
    let output_type = func.func_output_ty().expect("Expected function types");

    let Type::TupleT(input_types) = func.func_input_ty().expect("Expected function types") else {
        panic!(
            "Expected tuple type for inputs in tree_func_to_rvsdg. Got: {:?}",
            func.func_input_ty()
        )
    };

    let mut nodes = vec![];
    let (typechecked_program, type_cache) = program.with_arg_types_and_cache();
    let typechecked_func = typechecked_program
        .get_function(&func_name)
        .expect("Expected function in tree_func_to_rvsdg");

    let mut converter = TreeToRvsdg {
        program: &typechecked_program,
        type_cache: &type_cache,
        translation_cache: IndexMap::new(),
        nodes: &mut nodes,
        // initial arguments are the first n arguments
        current_args: (0..input_types.len()).map(Operand::Arg).collect(),
    };

    let converted = converter.convert_expr(typechecked_func.clone());

    RvsdgFunction {
        name: func
            .func_name()
            .expect("Expected function in tree_func_to_rvsdg"),
        // normal types and a state edge at the end
        args: input_types
            .into_iter()
            .map(basetype_to_rvsdg_type)
            .collect(),
        nodes,
        results: match output_type {
            Type::TupleT(types) => types
                .into_iter()
                .map(basetype_to_rvsdg_type)
                .zip(converted)
                .collect(),
            Type::Base(ty) => vec![(basetype_to_rvsdg_type(ty), converted[0])],
            Type::Unknown => panic!("Expected known type for function output"),
            Type::Symbolic(_) => {
                panic!("Symbolic type not supported in tree program to rvsdg conversion")
            }
        },
    }
}

fn value_op_from_binary_op(bop: BinaryOp) -> Option<ValueOps> {
    match bop {
        // integer operators
        BinaryOp::Add => Some(ValueOps::Add),
        BinaryOp::Sub => Some(ValueOps::Sub),
        BinaryOp::Mul => Some(ValueOps::Mul),
        BinaryOp::Div => Some(ValueOps::Div),
        BinaryOp::Eq => Some(ValueOps::Eq),
        BinaryOp::LessThan => Some(ValueOps::Lt),
        BinaryOp::GreaterThan => Some(ValueOps::Gt),
        BinaryOp::LessEq => Some(ValueOps::Le),
        BinaryOp::GreaterEq => Some(ValueOps::Ge),
        BinaryOp::Smax => Some(ValueOps::Smax),
        BinaryOp::Smin => Some(ValueOps::Smin),
        BinaryOp::Shl => Some(ValueOps::Shl),
        BinaryOp::Shr => Some(ValueOps::Shr),
        // float operators
        BinaryOp::FAdd => Some(ValueOps::Fadd),
        BinaryOp::FSub => Some(ValueOps::Fsub),
        BinaryOp::FMul => Some(ValueOps::Fmul),
        BinaryOp::FDiv => Some(ValueOps::Fdiv),
        BinaryOp::FEq => Some(ValueOps::Feq),
        BinaryOp::FLessThan => Some(ValueOps::Flt),
        BinaryOp::FGreaterThan => Some(ValueOps::Fgt),
        BinaryOp::FLessEq => Some(ValueOps::Fle),
        BinaryOp::FGreaterEq => Some(ValueOps::Fge),
        BinaryOp::Fmax => Some(ValueOps::Fmax),
        BinaryOp::Fmin => Some(ValueOps::Fmin),
        // logical op
        BinaryOp::And => Some(ValueOps::And),
        BinaryOp::Or => Some(ValueOps::Or),
        // pointer operators
        BinaryOp::PtrAdd => Some(ValueOps::PtrAdd),
        BinaryOp::Load => Some(ValueOps::Load),
        BinaryOp::Print => None,
        BinaryOp::Free => None,
    }
}

fn effect_op_from_binary_op(bop: BinaryOp) -> Option<EffectOps> {
    match bop {
        BinaryOp::Free => Some(EffectOps::Free),
        BinaryOp::Print => Some(EffectOps::Print),
        _ => None,
    }
}

fn value_op_from_unary_op(uop: UnaryOp) -> Option<ValueOps> {
    match uop {
        UnaryOp::Not => Some(ValueOps::Not),
    }
}

fn effect_op_from_unary_op(uop: UnaryOp) -> Option<EffectOps> {
    match uop {
        UnaryOp::Not => None,
    }
}

impl<'a> TreeToRvsdg<'a> {
    /// Translates an expression in a new subregion
    /// current_args is the operands that (Arg) refers to.
    /// initial_translation_cache is a cache of already evaluated expressions.
    /// For branch subregions, the initial translation cache maps branch input expressions
    /// to the Operand::Arg corresponding to them.
    fn translate_subregion(&mut self, expr: RcExpr, num_args: usize) -> Vec<Operand> {
        let args = (0..num_args).map(Operand::Arg).collect();
        let mut translator = TreeToRvsdg {
            program: self.program,
            nodes: self.nodes,
            type_cache: self.type_cache,
            translation_cache: IndexMap::new(),
            current_args: args,
        };
        translator.convert_expr(expr)
    }

    /// push a new basic expr and memoize the results
    fn push_basic(&mut self, basic: BasicExpr<Operand>) -> Vec<Operand> {
        let new_id = self.nodes.len();
        let num_outputs = basic.num_outputs();
        self.nodes.push(RvsdgBody::BasicOp(basic));

        (0..num_outputs)
            .map(|i| Operand::Project(i, new_id))
            .collect()
    }

    /// Some expressions such as Load and Alloc also return a state edge,
    /// so we need to ignore this when computing the bril type.
    fn get_basic_expr_type(&self, expr: RcExpr) -> bril_rs::Type {
        let cached = self
            .type_cache
            .get(&Rc::as_ptr(&expr))
            .expect("Expected to find type for expr")
            .clone();
        match cached {
            Type::Base(base) => basetype_to_bril_type(base),
            Type::TupleT(tuplet) => match tuplet.as_slice() {
                [base, BaseType::StateT] => basetype_to_bril_type(base.clone()),
                _ => panic!("Expected at most one type in basic expr type"),
            },
            Type::Unknown => panic!("Expected known type for expr"),
            Type::Symbolic(_) => panic!("Symbolic type not supported"),
        }
    }

    fn convert_expr(&mut self, expr: RcExpr) -> Operands {
        if let Some(operands) = self.translation_cache.get(&Rc::as_ptr(&expr)) {
            return operands.clone();
        }

        let res = match expr.as_ref() {
            Expr::Function(_name, _inty, _outty, expr) => self.convert_expr(expr.clone()),
            Expr::Const(constant, _ty, _ctx) => match constant {
                dag_in_context::schema::Constant::Int(integer) => self.push_basic(
                    BasicExpr::Const(ConstOps::Const, Literal::Int(*integer), bril_rs::Type::Int),
                ),
                dag_in_context::schema::Constant::Bool(boolean) => {
                    self.push_basic(BasicExpr::Const(
                        ConstOps::Const,
                        Literal::Bool(*boolean),
                        bril_rs::Type::Bool,
                    ))
                }
                dag_in_context::schema::Constant::Float(f) => self.push_basic(BasicExpr::Const(
                    ConstOps::Const,
                    Literal::Float(f.0),
                    bril_rs::Type::Float,
                )),
            },
            Expr::Top(TernaryOp::Write, c1, c2, c3) => {
                let c1 = self.convert_expr(c1.clone());
                let c2 = self.convert_expr(c2.clone());
                let c3 = self.convert_expr(c3.clone());
                self.push_basic(BasicExpr::Effect(
                    EffectOps::Store,
                    vec![c1[0], c2[0], c3[0]],
                ))
            }
            Expr::Top(TernaryOp::Select, c, t, e) => {
                let c = self.convert_expr(c.clone());
                let t = self.convert_expr(t.clone());
                let e = self.convert_expr(e.clone());
                let bril_type = self.get_basic_expr_type(expr.clone());
                assert_eq!(c.len(), 1, "Expected exactly one result for cond operand");
                assert_eq!(t.len(), 1, "Expected exactly one result for then operand");
                assert_eq!(e.len(), 1, "Expected exactly one result for else operand");
                self.push_basic(BasicExpr::Op(
                    ValueOps::Select,
                    vec![c[0], t[0], e[0]],
                    bril_type,
                ))
            }
            Expr::Bop(op, l, r) => {
                let l = self.convert_expr(l.clone());
                let r = self.convert_expr(r.clone());
                assert_eq!(l.len(), 1, "Expected exactly one result for left operand");
                assert_eq!(r.len(), 1, "Expected exactly one result for right operand");
                let l = l[0];
                let r = r[0];
                if let Some(vop) = value_op_from_binary_op(op.clone()) {
                    let bril_type = self.get_basic_expr_type(expr.clone());
                    self.push_basic(BasicExpr::Op(vop, vec![l, r], bril_type))
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
                    self.push_basic(BasicExpr::Op(
                        vop,
                        vec![child],
                        type_to_bril_type(
                            self.type_cache
                                .get(&Rc::as_ptr(&expr))
                                .expect("Expected to find type for expr")
                                .clone(),
                        )
                        .expect("Expected base type for unary op"),
                    ))
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
                    "Index out of bounds. Got child {:?} with index {:?}. Expression: {}",
                    child,
                    index,
                    expr
                );
                vec![child[*index]]
            }
            Expr::Alloc(_id, size, state, basety) => {
                let size = self.convert_expr(size.clone());
                assert_eq!(size.len(), 1, "Expected exactly one result for size");
                let state = self.convert_expr(state.clone());
                assert_eq!(state.len(), 1, "Expected exactly one result for state");
                self.push_basic(BasicExpr::Op(
                    ValueOps::Alloc,
                    vec![size[0], state[0]],
                    basetype_to_bril_type(basety.clone()),
                ))
            }
            Expr::Arg(_ty, _ctx) => self.current_args.clone(),
            Expr::Call(name, args) => {
                let func = self.program.get_function(name).expect("Function not found");
                let (func_ty, has_state_edge) = convert_func_type(func.func_output_ty().unwrap());
                let args = self.convert_expr(args.clone());
                let num_results = func_ty.is_some() as usize + has_state_edge as usize;
                self.push_basic(BasicExpr::Call(name.clone(), args, num_results, func_ty))
            }
            Expr::Empty(_ty, _ctx) => {
                vec![]
            }
            Expr::Concat(left, right) => {
                let left = self.convert_expr(left.clone());
                let right = self.convert_expr(right.clone());
                left.into_iter().chain(right).collect()
            }
            Expr::If(pred, input, then_branch, else_branch) => {
                // first convert the predicate
                let pred = self.convert_expr(pred.clone());
                assert_eq!(pred.len(), 1, "Expected exactly one result for predicate");

                // then convert the inputs
                let input = self.convert_expr(input.clone());

                let then_region = self.translate_subregion(then_branch.clone(), input.len());
                let else_region = self.translate_subregion(else_branch.clone(), input.len());

                let new_id = self.nodes.len();
                assert_eq!(
                    then_region.len(),
                    else_region.len(),
                    "Expected same number of values for then and else branches"
                );

                let res: Vec<Operand> = (0..then_region.len())
                    .map(|i| Operand::Project(i, new_id))
                    .collect();
                self.nodes.push(RvsdgBody::If {
                    pred: pred[0],
                    inputs: input,
                    then_branch: then_region,
                    else_branch: else_region,
                });

                res
            }
            Expr::Switch(pred, input, cases) => {
                // first convert the predicate
                let pred = self.convert_expr(pred.clone());
                assert_eq!(pred.len(), 1, "Expected exactly one result for predicate");
                let new_inputs = self.convert_expr(input.clone());

                let mut case_regions = vec![];
                for case_expr in cases {
                    let case_region = self.translate_subregion(case_expr.clone(), new_inputs.len());
                    case_regions.push(case_region);
                }

                let new_id = self.nodes.len();
                let res: Vec<Operand> = (0..case_regions[0].len())
                    .map(|i| Operand::Project(i, new_id))
                    .collect();
                self.nodes.push(RvsdgBody::Gamma {
                    pred: pred[0],
                    inputs: new_inputs,
                    outputs: case_regions,
                });

                res
            }
            Expr::DoWhile(inputs, body) => {
                let inputs_converted = self.convert_expr(inputs.clone());
                let pred_and_body = self.translate_subregion(body.clone(), inputs_converted.len());
                assert_eq!(
                    inputs_converted.len(),
                    pred_and_body.len() - 1,
                    "Expected matching number of inputs and outputs for do-while body"
                );

                let pred_inner = pred_and_body[0];
                let body_and_state_edge = pred_and_body[1..].to_vec();

                let new_id = self.nodes.len();

                // Project each result out of the
                // resulting Theta node (excluding state edge)
                let res = (0..body_and_state_edge.len())
                    .map(|i| Operand::Project(i, new_id))
                    .collect();
                self.nodes.push(RvsdgBody::Theta {
                    pred: pred_inner,
                    inputs: inputs_converted,
                    outputs: body_and_state_edge,
                });
                res
            }
            Expr::Single(body) => {
                let res = self.convert_expr(body.clone());
                assert_eq!(res.len(), 1, "Expected exactly one result for Single node");
                res
            }
            Expr::Symbolic(_, _ty) => panic!("symbolic not supported"),
        };
        self.translation_cache
            .insert(Rc::as_ptr(&expr), res.clone());
        res
    }
}
