//! Convert tree programs to RVSDGs

use bril_rs::{ConstOps, Literal};
use tree_in_context::schema::{BaseType, Expr, RcExpr, TreeProgram, Type};

use super::{BasicExpr, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};

type Operands = Vec<Operand>;

struct TreeToRvsdg {
    nodes: Vec<RvsdgBody>,
}

pub(crate) fn tree_to_rvsdg(tree: TreeProgram) -> RvsdgProgram {
    let mut res = RvsdgProgram { functions: vec![] };
    for func in tree.functions {
        res.functions.push(tree_func_to_rvsdg(func));
    }
    res.functions.push(tree_func_to_rvsdg(tree.entry));
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

fn rvsdg_types_from_tuple_type(ty: Type) -> Vec<RvsdgType> {
    let Type::TupleT(tys) = ty else {
        panic!("Expected tuple type in tree_type_to_rvsdg_types")
    };
    tys.into_iter()
        .map(|ty| RvsdgType::Bril(bril_type_from_type(ty)))
        .collect()
}

fn tree_func_to_rvsdg(func: RcExpr) -> RvsdgFunction {
    let types = func.func_output_ty().expect("Expected function types");

    let mut converter = TreeToRvsdg { nodes: vec![] };

    let converted = converter.convert_func(func.clone());

    RvsdgFunction {
        name: func
            .func_name()
            .expect("Expected function in tree_func_to_rvsdg"),
        args: rvsdg_types_from_tuple_type(types),
        nodes: converter.nodes,
        results: converted,
    }
}

impl TreeToRvsdg {
    pub fn convert_func(&mut self, func: RcExpr) -> Vec<(RvsdgType, Operand)> {
        todo!()
    }

    fn push_basic(&mut self, basic: BasicExpr<Operand>) -> Vec<Operand> {
        let new_id = self.nodes.len();
        self.nodes.push(RvsdgBody::BasicOp(basic));
        vec![Operand::Project(0, new_id)]
    }

    fn convert_expr(&mut self, expr: RcExpr) -> Operands {
        match expr.as_ref() {
            Expr::Const(constant) => match constant {
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
                self.push_basic(BasicExpr::Op(*op, vec![l, r], bril_rs::Type::Int))
            }
        }
    }
}
