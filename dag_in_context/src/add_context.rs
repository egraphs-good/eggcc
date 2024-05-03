//! Adds context to the tree program.
//! The `add_context` method recursively adds context to all of the nodes in the tree program
//! by remembering the most recent context (ex. DoWhile or If).
//! Mantains the sharing invariant (see restore_sharing_invariant) by using a cache.

use std::collections::HashMap;

use crate::{
    schema::{Assumption, Expr, RcExpr, TreeProgram},
    schema_helpers::AssumptionRef,
};

struct ContextCache {
    with_ctx: HashMap<(*const Expr, AssumptionRef), RcExpr>,
    symbol_gen: HashMap<(*const Expr, AssumptionRef), String>,
    /// When true, don't add context- instead, make fresh query variables
    /// and put these in place of context
    symbolic_ctx: bool,
    /// Replace all context with (InFunc "dummy")
    dummy_ctx: bool,
}

impl ContextCache {
    pub(crate) fn get_symbolic_ctx(&mut self, expr: &RcExpr, ctx: &Assumption) -> Assumption {
        let ctx_ref = ctx.to_ref();
        let key = (expr.as_ref() as *const Expr, ctx_ref.clone());
        if let Some(sym) = self.symbol_gen.get(&key) {
            return Assumption::WildCard(sym.clone());
        }
        let sym = format!("ctx__{}", self.symbol_gen.len());
        self.symbol_gen.insert(key, sym.clone());
        Assumption::WildCard(sym)
    }
}

impl TreeProgram {
    pub fn add_context(&self) -> TreeProgram {
        TreeProgram {
            functions: self
                .functions
                .iter()
                .map(|f| f.clone().func_add_ctx())
                .collect(),
            entry: self.entry.clone().func_add_ctx(),
        }
    }

    /// add stand-in variables for all the contexts in the program
    /// useful for testing if you don't care about context in the test
    #[allow(dead_code)]
    pub(crate) fn add_symbolic_ctx(&self) -> TreeProgram {
        TreeProgram {
            functions: self
                .functions
                .iter()
                .map(|f| f.clone().add_symbolic_ctx())
                .collect(),
            entry: self.entry.clone().add_symbolic_ctx(),
        }
    }

    pub(crate) fn add_dummy_ctx(&self) -> TreeProgram {
        TreeProgram {
            functions: self
                .functions
                .iter()
                .map(|f| f.clone().add_dummy_ctx())
                .collect(),
            entry: self.entry.clone().add_dummy_ctx(),
        }
    }
}

impl Expr {
    pub(crate) fn func_add_ctx(self: RcExpr) -> RcExpr {
        let Expr::Function(name, arg_ty, ret_ty, body) = &self.as_ref() else {
            panic!("Expected Function, got {:?}", self);
        };
        let current_ctx = Assumption::InFunc(name.clone());
        RcExpr::new(Expr::Function(
            name.clone(),
            arg_ty.clone(),
            ret_ty.clone(),
            body.add_ctx(current_ctx),
        ))
    }

    pub(crate) fn add_dummy_ctx(self: RcExpr) -> RcExpr {
        let mut cache = ContextCache {
            with_ctx: HashMap::new(),
            symbol_gen: HashMap::new(),
            symbolic_ctx: false,
            dummy_ctx: true,
        };
        self.add_ctx_with_cache(Assumption::dummy(), &mut cache)
    }

    pub(crate) fn add_symbolic_ctx(self: RcExpr) -> RcExpr {
        let mut cache = ContextCache {
            with_ctx: HashMap::new(),
            symbol_gen: HashMap::new(),
            symbolic_ctx: true,
            dummy_ctx: false,
        };
        self.add_ctx_with_cache(Assumption::dummy(), &mut cache)
    }

    pub(crate) fn add_ctx(self: &RcExpr, current_ctx: Assumption) -> RcExpr {
        let mut cache = ContextCache {
            with_ctx: HashMap::new(),
            symbol_gen: HashMap::new(),
            symbolic_ctx: false,
            dummy_ctx: false,
        };
        self.add_ctx_with_cache(current_ctx, &mut cache)
    }

    fn add_ctx_with_cache(
        self: &RcExpr,
        current_ctx: Assumption,
        cache: &mut ContextCache,
    ) -> RcExpr {
        let ctx_ref = current_ctx.to_ref();
        if let Some(expr) = cache
            .with_ctx
            .get(&((*self).as_ref() as *const Expr, ctx_ref.clone()))
        {
            return expr.clone();
        }
        let context_to_add = if cache.dummy_ctx {
            Assumption::dummy()
        } else if cache.symbolic_ctx {
            cache.get_symbolic_ctx(self, &current_ctx)
        } else {
            current_ctx.clone()
        };
        let res = match self.as_ref() {
            // replace the context of leaf nodes
            Expr::Const(c, ty, _oldctx) => {
                RcExpr::new(Expr::Const(c.clone(), ty.clone(), context_to_add))
            }
            Expr::Empty(ty, _oldctx) => RcExpr::new(Expr::Empty(ty.clone(), context_to_add)),
            Expr::Arg(ty, _oldctx) => RcExpr::new(Expr::Arg(ty.clone(), context_to_add)),
            // create new contexts for let, loop, and if
            Expr::DoWhile(inputs, pred_and_body) => {
                let new_inputs = inputs.add_ctx_with_cache(current_ctx.clone(), cache);
                let new_ctx = Assumption::InLoop(new_inputs.clone(), pred_and_body.clone());
                RcExpr::new(Expr::DoWhile(
                    new_inputs,
                    pred_and_body.add_ctx_with_cache(new_ctx, cache),
                ))
            }
            Expr::If(pred, input, then_case, else_calse) => {
                let new_pred = pred.add_ctx_with_cache(current_ctx.clone(), cache);
                let new_input = input.add_ctx_with_cache(current_ctx.clone(), cache);
                let then_ctx = Assumption::InIf(true, new_pred.clone(), new_input.clone());
                let else_ctx = Assumption::InIf(false, new_pred.clone(), new_input.clone());
                RcExpr::new(Expr::If(
                    new_pred,
                    new_input,
                    then_case.add_ctx_with_cache(then_ctx, cache),
                    else_calse.add_ctx_with_cache(else_ctx, cache),
                ))
            }
            Expr::Switch(case_num, input, branches) => {
                let new_case_num = case_num.add_ctx_with_cache(current_ctx.clone(), cache);
                let new_input = input.add_ctx_with_cache(current_ctx.clone(), cache);
                let new_branches = branches
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        let b_ctx = Assumption::InSwitch(
                            i.try_into().unwrap(),
                            new_case_num.clone(),
                            new_input.clone(),
                        );
                        b.add_ctx_with_cache(b_ctx, cache)
                    })
                    .collect();
                RcExpr::new(Expr::Switch(new_case_num, new_input, new_branches))
            }
            // for all other nodes, just add the context to the children
            Expr::Bop(op, x, y) => RcExpr::new(Expr::Bop(
                op.clone(),
                x.add_ctx_with_cache(current_ctx.clone(), cache),
                y.add_ctx_with_cache(current_ctx, cache),
            )),
            Expr::Top(op, x, y, z) => RcExpr::new(Expr::Top(
                op.clone(),
                x.add_ctx_with_cache(current_ctx.clone(), cache),
                y.add_ctx_with_cache(current_ctx.clone(), cache),
                z.add_ctx_with_cache(current_ctx, cache),
            )),
            Expr::Uop(op, x) => RcExpr::new(Expr::Uop(
                op.clone(),
                x.add_ctx_with_cache(current_ctx, cache),
            )),
            Expr::Get(e, i) => RcExpr::new(Expr::Get(e.add_ctx_with_cache(current_ctx, cache), *i)),
            Expr::Alloc(id, e, state, ty) => RcExpr::new(Expr::Alloc(
                *id,
                e.add_ctx_with_cache(current_ctx.clone(), cache),
                state.add_ctx_with_cache(current_ctx, cache),
                ty.clone(),
            )),
            Expr::Call(f, arg) => RcExpr::new(Expr::Call(
                f.clone(),
                arg.add_ctx_with_cache(current_ctx.clone(), cache),
            )),
            Expr::Single(e) => RcExpr::new(Expr::Single(e.add_ctx_with_cache(current_ctx, cache))),
            Expr::Concat(x, y) => RcExpr::new(Expr::Concat(
                x.add_ctx_with_cache(current_ctx.clone(), cache),
                y.add_ctx_with_cache(current_ctx, cache),
            )),
            Expr::Function(name, in_ty, out_ty, body) => RcExpr::new(Expr::Function(
                name.clone(),
                in_ty.clone(),
                out_ty.clone(),
                body.add_ctx_with_cache(current_ctx, cache),
            )),
        };
        cache
            .with_ctx
            .insert((self.as_ref() as *const Expr, ctx_ref), res.clone());
        res
    }
}
