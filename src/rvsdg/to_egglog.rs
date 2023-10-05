use bril_rs::{ConstOps, Literal, Type};
use egglog::ast::{Expr, Symbol};
use ordered_float::OrderedFloat;

use super::{BasicExpr, Operand, RvsdgBody, RvsdgFunction, RvsdgType};

impl RvsdgFunction {
    pub(crate) fn result_val(&self) -> Option<&Operand> {
        match &self.result {
            Some((_ty, val)) => Some(val),
            None => None,
        }
    }

    fn expr_from_ty(ty: &Type) -> Expr {
        use Expr::*;
        match ty {
            Type::Int => Call("IntT".into(), vec![]),
            Type::Bool => Call("BoolT".into(), vec![]),
            Type::Float => Call("FloatT".into(), vec![]),
            Type::Char => Call("CharT".into(), vec![]),
            Type::Pointer(ty) => Call("PointerT".into(), vec![Self::expr_from_ty(ty.as_ref())]),
        }
    }

    fn expr_from_rvsdg_ty(ty: &RvsdgType) -> Expr {
        use Expr::*;
        match ty {
            RvsdgType::Bril(ty) => Call("Bril".into(), vec![RvsdgFunction::expr_from_ty(ty)]),
            RvsdgType::PrintState => Call("PrintState".into(), vec![]),
        }
    }

    fn expr_to_egglog_expr(&self, expr: &BasicExpr<Operand>) -> Expr {
        use egglog::ast::{Expr::*, Literal::*};
        let f = |operands: &Vec<Operand>, ty: Option<Type>| {
            let mut res = Vec::with_capacity(operands.len() + ty.is_some() as usize);
            if let Some(ty) = ty {
                res.push(Self::expr_from_ty(&ty));
            }
            res.extend(operands.iter().map(|op| self.operand_to_egglog_expr(op)));
            res
        };

        match expr {
            BasicExpr::Op(op, operands, ty) => {
                Call(op.to_string().into(), f(operands, Some(ty.clone())))
            }
            // TODO I'm pretty sure this conversion isn't right
            BasicExpr::Call(ident, operands, _, ty) => {
                Call(ident.to_string().into(), f(operands, ty.clone()))
            }
            BasicExpr::Print(operands) => Call("PRINT".into(), f(operands, None)),
            BasicExpr::Const(ConstOps::Const, lit, ty) => {
                let lit = match (ty, lit) {
                    (Type::Int, Literal::Int(n)) => Call("Num".into(), vec![Lit(Int(*n))]),
                    (Type::Bool, Literal::Bool(b)) => {
                        Call("Bool".into(), vec![Lit(Int(*b as i64))])
                    }
                    (Type::Float, Literal::Float(f)) => Call(
                        "Float".into(),
                        vec![Lit(F64(OrderedFloat::<f64>::from(*f)))],
                    ),
                    (Type::Char, Literal::Char(c)) => {
                        Call("Char".into(), vec![Lit(String(c.to_string().into()))])
                    }
                    (Type::Pointer(ty), Literal::Int(p)) => Call(
                        "Ptr".into(),
                        vec![Self::expr_from_ty(ty.as_ref()), Lit(Int(*p))],
                    ),
                    _ => panic!("type mismatch"),
                };
                Call(
                    "Const".into(),
                    vec![Self::expr_from_ty(ty), Call("const".into(), vec![]), lit],
                )
            }
        }
    }

    fn vec_operand(operands: &[Expr]) -> Expr {
        use Expr::*;
        Call("VO".into(), vec![Call("vec-of".into(), operands.to_vec())])
    }

    fn vec_vec_operand(vecs: &[Expr]) -> Expr {
        use Expr::*;
        Call("VVO".into(), vec![Call("vec-of".into(), vecs.to_vec())])
    }

    /// Encode the rvsdg body as an egglog expression.
    /// Corresponds to a `Body` in the `schema.egg` file.
    fn body_to_egglog_expr(&self, body: &RvsdgBody) -> Expr {
        use Expr::*;
        match body {
            RvsdgBody::BasicOp(expr) => Call("PureOp".into(), vec![self.expr_to_egglog_expr(expr)]),
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => {
                let pred = self.operand_to_egglog_expr(pred);
                let inputs = inputs
                    .iter()
                    .map(|input| self.operand_to_egglog_expr(input))
                    .collect::<Vec<_>>();
                let inputs = Self::vec_operand(&inputs);
                let outputs = outputs
                    .iter()
                    .map(|region| {
                        let region = region
                            .iter()
                            .map(|output| self.operand_to_egglog_expr(output))
                            .collect::<Vec<_>>();
                        Self::vec_operand(&region)
                    })
                    .collect::<Vec<_>>();
                let outputs = Self::vec_vec_operand(&outputs);
                Call("Gamma".into(), vec![pred, inputs, outputs])
            }
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            } => {
                let pred = self.operand_to_egglog_expr(pred);
                let inputs = inputs
                    .iter()
                    .map(|input| self.operand_to_egglog_expr(input))
                    .collect::<Vec<_>>();
                let inputs = Self::vec_operand(&inputs);
                let outputs = outputs
                    .iter()
                    .map(|output| self.operand_to_egglog_expr(output))
                    .collect::<Vec<_>>();
                let outputs = Self::vec_operand(&outputs);
                Call("Theta".into(), vec![pred, inputs, outputs])
            }
        }
    }

    fn operand_to_egglog_expr(&self, op: &Operand) -> Expr {
        use egglog::ast::{Expr::*, Literal::*};
        match op {
            Operand::Arg(p) => Call("Arg".into(), vec![Lit(Int(i64::try_from(*p).unwrap()))]),
            Operand::Id(id) => Call(
                "Node".into(),
                vec![self.body_to_egglog_expr(&self.nodes[*id])],
            ),
            Operand::Project(i, id) => {
                let body = self.body_to_egglog_expr(&self.nodes[*id]);
                Call(
                    "Project".into(),
                    vec![Lit(Int(i64::try_from(*i).unwrap())), body],
                )
            }
        }
    }

    pub fn to_egglog_expr(&self) -> Expr {
        use egglog::ast::{Expr::*, Literal::*};
        let name: Symbol = self.name.clone().into();

        let sig = Call(
            "vec-of".into(),
            self.args
                .iter()
                .map(RvsdgFunction::expr_from_rvsdg_ty)
                .collect(),
        );
        let output = {
            let state = self.operand_to_egglog_expr(&self.state);
            if let Some((ty, result)) = &self.result {
                let value = self.operand_to_egglog_expr(result);
                let ty = Self::expr_from_ty(ty);
                Call("StateAndValue".into(), vec![state, ty, value])
            } else {
                Call("StateOnly".into(), vec![state])
            }
        };
        Call("Func".into(), vec![Lit(String(name)), sig, output])
    }
}
