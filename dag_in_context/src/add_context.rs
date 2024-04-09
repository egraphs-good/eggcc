//! Adds context to the tree program.
//! The `add_context` method recursively adds context to all of the nodes in the tree program
//! by remembering the most recent context (ex. Let or If).

use crate::{
    ast::{in_context, infunc},
    schema::{Assumption, Expr, RcExpr, TreeProgram},
};

impl TreeProgram {
    pub fn add_context(&self) -> TreeProgram {
        TreeProgram {
            functions: self
                .functions
                .iter()
                .map(|f| f.clone().func_add_context())
                .collect(),
            entry: self.entry.clone().func_add_context(),
        }
    }
}

impl Expr {
    pub(crate) fn func_add_context(self: RcExpr) -> RcExpr {
        let Expr::Function(name, arg_ty, ret_ty, body) = &self.as_ref() else {
            panic!("Expected Function, got {:?}", self);
        };
        let current_ctx = infunc(name);
        RcExpr::new(Expr::Function(
            name.clone(),
            arg_ty.clone(),
            ret_ty.clone(),
            body.add_context(current_ctx),
        ))
    }

    fn add_context(self: &RcExpr, current_ctx: Assumption) -> RcExpr {
        match self.as_ref() {
            // leaf nodes are constant, empty, and arg
            Expr::Const(..) | Expr::Empty(..) | Expr::Arg(..) => {
                in_context(current_ctx, self.clone())
            }
            // create new contexts for let, loop, and if
            Expr::DoWhile(inputs, pred_and_body) => {
                let new_inputs = inputs.add_context(current_ctx.clone());
                let new_ctx = Assumption::InLoop(new_inputs.clone(), pred_and_body.clone());
                RcExpr::new(Expr::DoWhile(
                    new_inputs,
                    pred_and_body.add_context(new_ctx),
                ))
            }
            Expr::If(pred, then_case, else_calse) => {
                let new_pred = pred.add_context(current_ctx.clone());
                let then_ctx = Assumption::InIf(true, new_pred.clone());
                let else_ctx = Assumption::InIf(false, new_pred.clone());
                RcExpr::new(Expr::If(
                    new_pred,
                    then_case.add_context(then_ctx),
                    else_calse.add_context(else_ctx),
                ))
            }
            Expr::Switch(case_num, branches) => {
                let new_case_num = case_num.add_context(current_ctx.clone());
                let new_branches = branches
                    .iter()
                    .map(|b| b.add_context(current_ctx.clone()))
                    .collect();
                RcExpr::new(Expr::Switch(new_case_num, new_branches))
            }
            // for all other nodes, just add the context to the children
            Expr::Bop(op, x, y) => RcExpr::new(Expr::Bop(
                op.clone(),
                x.add_context(current_ctx.clone()),
                y.add_context(current_ctx),
            )),
            Expr::Top(op, x, y, z) => RcExpr::new(Expr::Top(
                op.clone(),
                x.add_context(current_ctx.clone()),
                y.add_context(current_ctx.clone()),
                z.add_context(current_ctx),
            )),
            Expr::Uop(op, x) => RcExpr::new(Expr::Uop(op.clone(), x.add_context(current_ctx))),
            Expr::Get(e, i) => RcExpr::new(Expr::Get(e.add_context(current_ctx), *i)),
            Expr::Alloc(id, e, state, ty) => RcExpr::new(Expr::Alloc(
                *id,
                e.add_context(current_ctx.clone()),
                state.add_context(current_ctx),
                ty.clone(),
            )),
            Expr::Call(f, arg) => {
                RcExpr::new(Expr::Call(f.clone(), arg.add_context(current_ctx.clone())))
            }
            Expr::Single(e) => RcExpr::new(Expr::Single(e.add_context(current_ctx))),
            Expr::Concat(order, x, y) => RcExpr::new(Expr::Concat(
                order.clone(),
                x.add_context(current_ctx.clone()),
                y.add_context(current_ctx),
            )),
            Expr::InContext(..) => {
                panic!("add_context expects a term without context")
            }
            Expr::Function(..) => panic!("Function should have been handled in func_add_context"),
        }
    }
}
