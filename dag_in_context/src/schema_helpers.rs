use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    rc::Rc,
    vec,
};

use strum_macros::EnumIter;

use crate::{
    add_context::ContextCache,
    ast::{base, boolt, floatt, inif, inloop, inswitch, intt},
    schema::{
        Assumption, BaseType, BinaryOp, Constant, Expr, RcExpr, TernaryOp, TreeProgram, Type,
        UnaryOp,
    },
};

/// Display for Constant implements a
/// rust-readable representation using
/// the sugar in `ast.rs`.
impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let rcexpr = RcExpr::new(self.clone());
        let (term, termdag) = rcexpr.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Display for Assumption {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl TernaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use TernaryOp::*;
        match self {
            Write => "Write",
            Select => "Select",
        }
    }
}

impl BinaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use BinaryOp::*;
        match self {
            Add => "Add",
            Sub => "Sub",
            Mul => "Mul",
            Div => "Div",
            Eq => "Eq",
            GreaterThan => "GreaterThan",
            LessThan => "LessThan",
            GreaterEq => "GreaterEq",
            LessEq => "LessEq",
            Smax => "Smax",
            Smin => "Smin",
            Shl => "Shl",
            Shr => "Shr",
            FAdd => "FAdd",
            FSub => "FSub",
            FMul => "FMul",
            FDiv => "FDiv",
            FEq => "FEq",
            FGreaterThan => "FGreaterThan",
            FLessThan => "FLessThan",
            FGreaterEq => "FGreaterEq",
            FLessEq => "FLessEq",
            Fmax => "Fmax",
            Fmin => "Fmin",
            And => "And",
            Or => "Or",
            Load => "Load",
            Free => "Free",
            Print => "Print",
            PtrAdd => "PtrAdd",
        }
    }
}

impl UnaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use UnaryOp::*;
        match self {
            Abs => "Abs",
            Not => "Not",
        }
    }
}

impl Display for TreeProgram {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Expr {
    pub fn constructor(&self) -> Constructor {
        match self {
            Expr::Function(..) => Constructor::Function,
            Expr::Const(..) => Constructor::Const,
            Expr::Bop(..) => Constructor::Bop,
            Expr::Uop(..) => Constructor::Uop,
            Expr::Get(..) => Constructor::Get,
            Expr::Concat(..) => Constructor::Concat,
            Expr::Single(..) => Constructor::Single,
            Expr::Switch(..) => Constructor::Switch,
            Expr::If(..) => Constructor::If,
            Expr::DoWhile(..) => Constructor::DoWhile,
            Expr::Arg(..) => Constructor::Arg,
            Expr::Call(..) => Constructor::Call,
            Expr::Empty(..) => Constructor::Empty,
            Expr::Alloc(..) => Constructor::Alloc,
            Expr::Top(..) => Constructor::Top,
            Expr::Symbolic(_, _ty) => panic!("found symbolic"),
            Expr::DeadCode(_arg_ty, _ty) => panic!("found dead code"),
        }
    }
    pub fn func_name(&self) -> Option<String> {
        match self {
            Expr::Function(name, _, _, _) => Some(name.clone()),
            _ => None,
        }
    }

    pub fn func_input_ty(&self) -> Option<Type> {
        match self {
            Expr::Function(_, ty, _, _) => Some(ty.clone()),
            _ => None,
        }
    }

    pub fn func_output_ty(&self) -> Option<Type> {
        match self {
            Expr::Function(_, _, ty, _) => Some(ty.clone()),
            _ => None,
        }
    }

    pub fn func_body(&self) -> Option<&RcExpr> {
        match self {
            Expr::Function(_, _, _, body) => Some(body),
            _ => None,
        }
    }

    pub fn func_to_program(&self) -> TreeProgram {
        match self {
            Expr::Function(name, input_ty, output_ty, body) => TreeProgram {
                entry: RcExpr::new(Expr::Function(
                    name.clone(),
                    input_ty.clone(),
                    output_ty.clone(),
                    body.clone(),
                )),
                functions: vec![],
            },
            _ => panic!("Expected function"),
        }
        .with_arg_types()
    }

    /// Converts this expression to a program, and ensures arguments
    /// have the correct type by calling `with_arg_types`.
    pub fn to_program(self: &RcExpr, input_ty: Type, output_ty: Type) -> TreeProgram {
        match self.as_ref() {
            Expr::Function(..) => TreeProgram {
                entry: self.clone(),
                functions: vec![],
            },
            _ => TreeProgram {
                entry: RcExpr::new(Expr::Function(
                    "main".to_string(),
                    input_ty,
                    output_ty,
                    self.clone(),
                )),
                functions: vec![],
            },
        }
        .with_arg_types()
    }

    // Get all the Expr children of this expression
    pub fn children_exprs(self: &RcExpr) -> Vec<RcExpr> {
        match self.as_ref() {
            Expr::Top(_, x, y, z) => vec![x.clone(), y.clone(), z.clone()],
            Expr::Bop(_, x, y) => vec![x.clone(), y.clone()],
            Expr::Uop(_, x) => vec![x.clone()],
            Expr::Alloc(_, x, y, _) => vec![x.clone(), y.clone()],
            Expr::Call(_, x) => vec![x.clone()],
            Expr::Single(x) => vec![x.clone()],
            Expr::Concat(x, y) => vec![x.clone(), y.clone()],
            Expr::If(x, y, z, w) => vec![x.clone(), y.clone(), z.clone(), w.clone()],
            Expr::Switch(x, y, cases) => {
                let mut res = vec![x.clone(), y.clone()];
                res.extend(cases.clone());
                res
            }
            Expr::DoWhile(x, y) => vec![x.clone(), y.clone()],
            Expr::Function(_, _, _, x) => vec![x.clone()],
            Expr::Get(x, _) => vec![x.clone()],
            Expr::Const(_, _, _) => vec![],
            Expr::Empty(_, _) => vec![],
            Expr::Arg(_, _) => vec![],
            Expr::Symbolic(_, _ty) => vec![],
            Expr::DeadCode(_arg_ty, _ty) => vec![],
        }
    }

    /// Get the children of this expression that are still in the same scope
    /// For context nodes, doesn't include the context (which is an assumption)
    pub fn children_same_scope(self: &RcExpr) -> Vec<RcExpr> {
        match self.as_ref() {
            Expr::Function(_, _, _, body) => vec![body.clone()],
            Expr::Const(..) => vec![],
            Expr::Top(_, x, y, z) => vec![x.clone(), y.clone(), z.clone()],
            Expr::Bop(_, x, y) => vec![x.clone(), y.clone()],
            Expr::Uop(_, x) => vec![x.clone()],
            Expr::Get(x, _) => vec![x.clone()],
            Expr::Alloc(_, x, y, _) => vec![x.clone(), y.clone()],
            Expr::Call(_, x) => vec![x.clone()],
            Expr::Empty(_, _) => vec![],
            Expr::Single(x) => vec![x.clone()],
            Expr::Concat(x, y) => vec![x.clone(), y.clone()],
            Expr::Switch(x, inputs, _branches) => {
                let children = vec![x.clone(), inputs.clone()];
                children
            }
            Expr::If(x, inputs, _y, _z) => {
                let children = vec![x.clone(), inputs.clone()];
                children
            }
            Expr::DoWhile(inputs, _body) => vec![inputs.clone()],
            Expr::Arg(_, _) => vec![],
            Expr::Symbolic(_, _ty) => vec![],
            Expr::DeadCode(_arg_ty, _ty) => vec![],
        }
    }

    pub fn get_arg_type(&self) -> Type {
        match self {
            Expr::Const(_, ty, _) => ty.clone(),
            Expr::Top(_, x, _, _) => x.get_arg_type(),
            Expr::Bop(_, x, _) => x.get_arg_type(),
            Expr::Uop(_, x) => x.get_arg_type(),
            Expr::Get(x, _) => x.get_arg_type(),
            Expr::Alloc(_, x, _, _) => x.get_arg_type(),
            Expr::Call(_, x) => x.get_arg_type(),
            Expr::Empty(ty, _) => ty.clone(),
            Expr::Single(x) => x.get_arg_type(),
            Expr::Concat(x, _) => x.get_arg_type(),
            Expr::If(x, _, _, _) => x.get_arg_type(),
            Expr::Switch(x, _, _) => x.get_arg_type(),
            Expr::DoWhile(x, _) => x.get_arg_type(),
            Expr::Arg(ty, _) => ty.clone(),
            Expr::Function(_, ty, _, _) => ty.clone(),
            Expr::Symbolic(_, _ty) => panic!("found symbolic"),
            Expr::DeadCode(arg_ty, _ty) => arg_ty.clone(),
        }
    }

    // this function might violate RcExpr's invariant
    // for example function map_child is id function that create new RcExpr, and &self have two same children
    pub fn map_expr_children<F>(self: &RcExpr, mut map_child: F) -> RcExpr
    where
        F: FnMut(&RcExpr) -> RcExpr,
    {
        match self.as_ref() {
            Expr::Function(name, ty1, ty2, body) => Rc::new(Expr::Function(
                name.into(),
                ty1.clone(),
                ty2.clone(),
                map_child(body),
            )),
            Expr::Top(op, x, y, z) => Rc::new(Expr::Top(
                op.clone(),
                map_child(x),
                map_child(y),
                map_child(z),
            )),
            Expr::Bop(op, x, y) => Rc::new(Expr::Bop(op.clone(), map_child(x), map_child(y))),
            Expr::Uop(op, x) => Rc::new(Expr::Uop(op.clone(), map_child(x))),
            Expr::Get(x, pos) => Rc::new(Expr::Get(map_child(x), *pos)),
            Expr::Alloc(id, x, y, ty) => {
                Rc::new(Expr::Alloc(*id, map_child(x), map_child(y), ty.clone()))
            }
            Expr::Call(name, x) => Rc::new(Expr::Call(name.into(), map_child(x))),
            Expr::Single(x) => Rc::new(Expr::Single(map_child(x))),
            Expr::Concat(x, y) => Rc::new(Expr::Concat(map_child(x), map_child(y))),
            Expr::Switch(cond, inputs, branches) => {
                let br = branches.iter().map(&mut map_child).collect::<Vec<_>>();
                Rc::new(Expr::Switch(map_child(cond), map_child(inputs), br))
            }
            Expr::If(pred, input, then, els) => Rc::new(Expr::If(
                map_child(pred),
                map_child(input),
                map_child(then),
                map_child(els),
            )),
            Expr::DoWhile(input, body) => Rc::new(Expr::DoWhile(map_child(input), map_child(body))),
            Expr::Symbolic(_, _ty) => panic!("No symbolic should occur here"),
            _ => self.clone(),
        }
    }

    pub fn map_expr_type<F>(self: &RcExpr, mut map_type: F) -> RcExpr
    where
        F: FnMut(&Type) -> Type,
    {
        match self.as_ref() {
            Expr::Function(name, ty1, ty2, body) => Rc::new(Expr::Function(
                name.into(),
                map_type(ty1),
                map_type(ty2),
                body.clone(),
            )),
            Expr::Const(c, ty, assum) => {
                Rc::new(Expr::Const(c.clone(), map_type(ty), assum.clone()))
            }
            Expr::Empty(ty, assum) => Rc::new(Expr::Empty(map_type(ty), assum.clone())),
            Expr::Arg(ty, assum) => Rc::new(Expr::Arg(map_type(ty), assum.clone())),
            _ => self.clone(),
        }
    }

    pub fn map_expr_assum<F>(self: &RcExpr, mut map_assum: F) -> RcExpr
    where
        F: FnMut(&Assumption) -> Assumption,
    {
        match self.as_ref() {
            Expr::Const(c, ty, assum) => {
                Rc::new(Expr::Const(c.clone(), ty.clone(), map_assum(assum)))
            }
            Expr::Empty(ty, assum) => Rc::new(Expr::Empty(ty.clone(), map_assum(assum))),
            Expr::Arg(ty, assum) => Rc::new(Expr::Arg(ty.clone(), map_assum(assum))),
            _ => self.clone(),
        }
    }

    pub fn get_ctx(&self) -> &Assumption {
        match self {
            Expr::Const(_, _, ctx) => ctx,
            Expr::Top(_, x, _, _) => x.get_ctx(),
            Expr::Bop(_, x, _) => x.get_ctx(),
            Expr::Uop(_, x) => x.get_ctx(),
            Expr::Get(x, _) => x.get_ctx(),
            Expr::Alloc(_, x, _, _) => x.get_ctx(),
            Expr::Call(_, x) => x.get_ctx(),
            Expr::Empty(_, ctx) => ctx,
            Expr::Single(x) => x.get_ctx(),
            Expr::Concat(x, _) => x.get_ctx(),
            Expr::If(x, _, _, _) => x.get_ctx(),
            Expr::Switch(x, _, _) => x.get_ctx(),
            Expr::DoWhile(x, _) => x.get_ctx(),
            Expr::Arg(_, ctx) => ctx,
            Expr::Function(_, _, _, x) => x.get_ctx(),
            Expr::Symbolic(_, _ty) => panic!("found symbolic"),
            Expr::DeadCode(_arg_ty, _ty) => panic!("found dead code"),
        }
    }

    // Substitute "arg" for Arg() in within. Also replaces context with "arg"'s context.
    pub fn subst(arg: &RcExpr, within: &RcExpr, context_cache: &mut ContextCache) -> RcExpr {
        let mut subst_cache: HashMap<*const Expr, RcExpr> = HashMap::new();

        let arg_ty = arg.get_arg_type();
        let arg_ctx = arg.get_ctx();
        Self::subst_with_cache(
            arg,
            &arg_ty,
            arg_ctx,
            within,
            &mut subst_cache,
            context_cache,
        )
    }

    fn subst_with_cache(
        arg: &RcExpr,
        arg_ty: &Type,
        arg_ctx: &Assumption,
        within: &RcExpr,
        subst_cache: &mut HashMap<*const Expr, RcExpr>,
        context_cache: &mut ContextCache,
    ) -> RcExpr {
        if let Some(substed) = subst_cache.get(&Rc::as_ptr(within)) {
            return substed.clone();
        }

        let substed = match within.as_ref() {
            // Substitute!
            Expr::Arg(_, _) => arg.clone(),

            // Propagate through current scope
            Expr::Top(op, x, y, z) => Rc::new(Expr::Top(
                op.clone(),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, x, subst_cache, context_cache),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, y, subst_cache, context_cache),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, z, subst_cache, context_cache),
            )),
            Expr::Bop(op, x, y) => Rc::new(Expr::Bop(
                op.clone(),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, x, subst_cache, context_cache),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, y, subst_cache, context_cache),
            )),
            Expr::Uop(op, x) => Rc::new(Expr::Uop(
                op.clone(),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, x, subst_cache, context_cache),
            )),
            Expr::Get(x, i) => Rc::new(Expr::Get(
                Self::subst_with_cache(arg, arg_ty, arg_ctx, x, subst_cache, context_cache),
                *i,
            )),
            Expr::Alloc(amt, x, y, ty) => Rc::new(Expr::Alloc(
                *amt,
                Self::subst_with_cache(arg, arg_ty, arg_ctx, x, subst_cache, context_cache),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, y, subst_cache, context_cache),
                ty.clone(),
            )),
            Expr::Call(name, x) => Rc::new(Expr::Call(
                name.clone(),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, x, subst_cache, context_cache),
            )),
            Expr::Single(x) => Rc::new(Expr::Single(Self::subst_with_cache(
                arg,
                arg_ty,
                arg_ctx,
                x,
                subst_cache,
                context_cache,
            ))),
            Expr::Concat(x, y) => Rc::new(Expr::Concat(
                Self::subst_with_cache(arg, arg_ty, arg_ctx, x, subst_cache, context_cache),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, y, subst_cache, context_cache),
            )),
            Expr::If(pred, input, then, els) => {
                let new_pred =
                    Self::subst_with_cache(arg, arg_ty, arg_ctx, pred, subst_cache, context_cache);
                let new_input =
                    Self::subst_with_cache(arg, arg_ty, arg_ctx, input, subst_cache, context_cache);
                Rc::new(Expr::If(
                    new_pred.clone(),
                    new_input.clone(),
                    then.add_ctx_with_cache(
                        inif(true, new_pred.clone(), new_input.clone()),
                        context_cache,
                    ),
                    els.add_ctx_with_cache(inif(false, new_pred, new_input), context_cache),
                ))
            }
            Expr::Switch(pred, input, branches) => {
                let new_pred =
                    Self::subst_with_cache(arg, arg_ty, arg_ctx, pred, subst_cache, context_cache);
                let new_input =
                    Self::subst_with_cache(arg, arg_ty, arg_ctx, input, subst_cache, context_cache);
                let new_branches = branches
                    .iter()
                    .enumerate()
                    .map(|(i, branch)| {
                        branch.add_ctx_with_cache(
                            inswitch(i.try_into().unwrap(), new_pred.clone(), new_input.clone()),
                            context_cache,
                        )
                    })
                    .collect();
                Rc::new(Expr::Switch(new_pred, new_input, new_branches))
            }
            Expr::DoWhile(input, pred_and_body) => {
                let placeholder = context_cache.new_placeholder();

                let new_inputs =
                    Self::subst_with_cache(arg, arg_ty, arg_ctx, input, subst_cache, context_cache);
                let new_pred_and_body =
                    pred_and_body.add_ctx_with_cache(placeholder.clone(), context_cache);

                let new_ctx = inloop(new_inputs.clone(), new_pred_and_body.clone());
                context_cache.push_loop_context_union(placeholder, new_ctx);

                RcExpr::new(Expr::DoWhile(new_inputs, new_pred_and_body))
            }
            Expr::Function(x, y, z, body) => Rc::new(Expr::Function(
                x.clone(),
                y.clone(),
                z.clone(),
                Self::subst_with_cache(arg, arg_ty, arg_ctx, body, subst_cache, context_cache),
            )),

            // For leaves, replace the type and context
            Expr::Const(c, _, _) => {
                Rc::new(Expr::Const(c.clone(), arg_ty.clone(), arg_ctx.clone()))
            }
            Expr::Empty(_, _) => Rc::new(Expr::Empty(arg_ty.clone(), arg_ctx.clone())),
            Expr::Symbolic(_, _ty) => panic!("found symbolic"),
            Expr::DeadCode(_arg_ty, _ty) => panic!("found dead code"),
        };

        // Add the substituted to cache
        subst_cache.insert(Rc::as_ptr(within), substed.clone());
        substed
    }
}

impl TreeProgram {
    pub fn get_function(&self, name: &str) -> Option<&RcExpr> {
        if self.entry.func_name() == Some(name.to_string()) {
            return Some(&self.entry);
        }
        self.functions
            .iter()
            .find(|expr| expr.func_name() == Some(name.to_string()))
    }

    pub fn pretty(&self) -> String {
        let (term, termdag) = self.to_egglog();
        let expr = termdag.term_to_expr(&term);
        expr.to_sexp().pretty()
    }

    pub fn fns(&self) -> Vec<String> {
        let mut res = vec![];
        if let Some(name) = self.entry.func_name() {
            res.push(name);
        }
        res.extend(self.functions.iter().filter_map(|expr| expr.func_name()));
        res
    }

    pub fn replace_fn(&mut self, name: &str, new_fn: RcExpr) {
        if self.entry.func_name() == Some(name.to_string()) {
            self.entry = new_fn;
        } else {
            for fn_expr in self.functions.iter_mut() {
                if fn_expr.func_name() == Some(name.to_string()) {
                    *fn_expr = new_fn.clone();
                    return;
                }
            }
            panic!("Function {} not found", name);
        }
    }
}

use std::iter;

#[derive(Clone, Copy, Debug, PartialEq, EnumIter)]
pub(crate) enum Sort {
    Expr,
    ListExpr,
    BinaryOp,
    UnaryOp,
    TernaryOp,
    I64,
    F64,
    Bool,
    Type,
    String,
    Constant,
    Assumption,
    BaseType,
    TypeList,
    ProgramType,
}

impl Sort {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Sort::Expr => "Expr",
            Sort::ListExpr => "ListExpr",
            Sort::I64 => "i64",
            Sort::F64 => "f64",
            Sort::Bool => "bool",
            Sort::String => "String",
            Sort::Type => "Type",
            Sort::BinaryOp => "BinaryOp",
            Sort::UnaryOp => "UnaryOp",
            Sort::TernaryOp => "TernaryOp",
            Sort::Constant => "Constant",
            Sort::Assumption => "Assumption",
            Sort::BaseType => "BaseType",
            Sort::TypeList => "TypeList",
            Sort::ProgramType => "ProgramType",
        }
    }
}

#[derive(Debug, EnumIter, PartialEq)]
pub(crate) enum ESort {
    Expr,
    ListExpr,
}

impl std::fmt::Display for ESort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ESort {
    pub(crate) fn to_sort(&self) -> Sort {
        match self {
            ESort::Expr => Sort::Expr,
            ESort::ListExpr => Sort::ListExpr,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.to_sort().name()
    }
}

#[derive(Clone, Debug, EnumIter, PartialEq)]
pub enum Constructor {
    Function,
    Const,
    Top,
    Bop,
    Uop,
    Get,
    Concat,
    Single,
    Switch,
    If,
    DoWhile,
    Arg,
    Call,
    Empty,
    Cons,
    Nil,
    Alloc,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Purpose {
    Static(Sort),        // some int, bool, order that parameterizes constructor
    SubExpr,             // subexpression, e.g. Add's summand
    CapturedSubListExpr, // a swtich's branches
    CapturedExpr,        // a body's outputs
}

impl Purpose {
    pub(crate) fn to_sort(self) -> Sort {
        match self {
            Purpose::SubExpr => Sort::Expr,
            Purpose::CapturedExpr => Sort::Expr,
            Purpose::CapturedSubListExpr => Sort::ListExpr,
            Purpose::Static(sort) => sort,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Field {
    pub purpose: Purpose,
    pub name: &'static str,
}

impl Field {
    pub(crate) fn sort(&self) -> Sort {
        self.purpose.to_sort()
    }

    pub(crate) fn var(&self) -> String {
        format!("_{name}", name = self.name)
    }
}

impl Constructor {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Constructor::Function => "Function",
            Constructor::Const => "Const",
            Constructor::Bop => "Bop",
            Constructor::Uop => "Uop",
            Constructor::Get => "Get",
            Constructor::Concat => "Concat",
            Constructor::Single => "Single",
            Constructor::Switch => "Switch",
            Constructor::If => "If",
            Constructor::DoWhile => "DoWhile",
            Constructor::Arg => "Arg",
            Constructor::Call => "Call",
            Constructor::Empty => "Empty",
            Constructor::Alloc => "Alloc",
            Constructor::Cons => "Cons",
            Constructor::Nil => "Nil",
            Constructor::Top => "Top",
        }
    }

    pub(crate) fn fields(&self) -> Vec<Field> {
        use Purpose::{CapturedExpr, CapturedSubListExpr, Static, SubExpr};
        let f = |purpose, name| Field { purpose, name };
        match self {
            Constructor::Function => {
                vec![
                    f(Static(Sort::String), "name"),
                    f(Static(Sort::Type), "tyin"),
                    f(Static(Sort::Type), "tyout"),
                    f(CapturedExpr, "out"),
                ]
            }
            Constructor::Const => {
                vec![
                    f(Static(Sort::Constant), "n"),
                    f(Static(Sort::Type), "ty"),
                    f(Static(Sort::Assumption), "ctx"),
                ]
            }
            Constructor::Top => vec![
                f(Static(Sort::TernaryOp), "op"),
                f(SubExpr, "x"),
                f(SubExpr, "y"),
                f(SubExpr, "z"),
            ],
            Constructor::Bop => vec![
                f(Static(Sort::BinaryOp), "op"),
                f(SubExpr, "x"),
                f(SubExpr, "y"),
            ],
            Constructor::Uop => {
                vec![f(Static(Sort::UnaryOp), "op"), f(SubExpr, "x")]
            }
            Constructor::Get => vec![f(SubExpr, "tup"), f(Static(Sort::I64), "i")],
            Constructor::Concat => {
                vec![f(SubExpr, "x"), f(SubExpr, "y")]
            }
            Constructor::Single => {
                vec![f(SubExpr, "x")]
            }
            Constructor::Switch => {
                vec![
                    f(SubExpr, "pred"),
                    f(SubExpr, "inputs"),
                    f(CapturedSubListExpr, "branches"),
                ]
            }
            Constructor::If => {
                vec![
                    f(SubExpr, "pred"),
                    f(SubExpr, "input"),
                    f(CapturedExpr, "then"),
                    f(CapturedExpr, "else"),
                ]
            }
            Constructor::DoWhile => {
                vec![f(SubExpr, "in"), f(CapturedExpr, "pred-and-output")]
            }
            Constructor::Arg => vec![
                f(Static(Sort::Type), "ty"),
                f(Static(Sort::Assumption), "ctx"),
            ],
            Constructor::Call => {
                vec![f(Static(Sort::String), "func"), f(SubExpr, "arg")]
            }
            Constructor::Empty => vec![
                f(Static(Sort::Type), "ty"),
                f(Static(Sort::Assumption), "ctx"),
            ],
            Constructor::Cons => vec![f(SubExpr, "hd"), f(CapturedSubListExpr, "tl")],
            Constructor::Nil => vec![],
            Constructor::Alloc => vec![
                f(Static(Sort::I64), "id"),
                f(SubExpr, "e"),
                f(SubExpr, "state"),
                f(Static(Sort::Type), "ty"),
            ],
        }
    }

    pub(crate) fn filter_map_fields<F, T>(&self, f: F) -> Vec<T>
    where
        F: FnMut(&Field) -> Option<T>,
    {
        self.fields().iter().filter_map(f).collect::<Vec<_>>()
    }

    pub(crate) fn construct<F>(&self, f: F) -> String
    where
        F: FnMut(&Field) -> String,
    {
        let without_parens = iter::once(self.name().to_string())
            .chain(self.fields().iter().map(f))
            .collect::<Vec<_>>()
            .join(" ");
        format!("({without_parens})")
    }

    pub(crate) fn sort(&self) -> ESort {
        match self {
            Constructor::Function => ESort::Expr,
            Constructor::Const => ESort::Expr,
            Constructor::Top => ESort::Expr,
            Constructor::Bop => ESort::Expr,
            Constructor::Uop => ESort::Expr,
            Constructor::Get => ESort::Expr,
            Constructor::Concat => ESort::Expr,
            Constructor::Single => ESort::Expr,
            Constructor::Switch => ESort::Expr,
            Constructor::If => ESort::Expr,
            Constructor::DoWhile => ESort::Expr,
            Constructor::Arg => ESort::Expr,
            Constructor::Call => ESort::Expr,
            Constructor::Empty => ESort::Expr,
            Constructor::Alloc => ESort::Expr,
            Constructor::Cons => ESort::ListExpr,
            Constructor::Nil => ESort::ListExpr,
        }
    }
}

impl BinaryOp {
    /// When a binary op has concrete input sorts, return them.
    pub fn types(&self) -> Option<(Type, Type, Type)> {
        match self {
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Smax
            | BinaryOp::Smin
            | BinaryOp::Shl
            | BinaryOp::Shr => Some((base(intt()), base(intt()), base(intt()))),
            BinaryOp::FAdd
            | BinaryOp::FSub
            | BinaryOp::FMul
            | BinaryOp::FDiv
            | BinaryOp::Fmax
            | BinaryOp::Fmin => Some((base(floatt()), base(floatt()), base(floatt()))),
            BinaryOp::And | BinaryOp::Or => Some((base(boolt()), base(boolt()), base(boolt()))),
            BinaryOp::LessThan
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterEq
            | BinaryOp::LessEq
            | BinaryOp::Eq => Some((base(intt()), base(intt()), base(boolt()))),
            BinaryOp::FLessThan
            | BinaryOp::FGreaterThan
            | BinaryOp::FGreaterEq
            | BinaryOp::FLessEq
            | BinaryOp::FEq => Some((base(floatt()), base(floatt()), base(boolt()))),
            BinaryOp::Load => None,
            BinaryOp::Free => None,
            BinaryOp::Print => None,
            BinaryOp::PtrAdd => None,
        }
    }
}

impl UnaryOp {
    pub(crate) fn types(&self) -> Option<(Type, Type)> {
        match self {
            UnaryOp::Abs => Some((base(intt()), base(intt()))),
            UnaryOp::Not => Some((base(boolt()), base(boolt()))),
        }
    }
}

/// used to hash an assumption
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssumptionRef {
    InLoop(*const Expr, *const Expr),
    InFunc(String),
    InIf(bool, *const Expr, *const Expr),
    InSwitch(i64, *const Expr, *const Expr),
    WildCard(String),
}

impl Assumption {
    pub fn to_ref(&self) -> AssumptionRef {
        match self {
            Assumption::InLoop(inputs, pred_and_body) => {
                AssumptionRef::InLoop(Rc::as_ptr(inputs), Rc::as_ptr(pred_and_body))
            }
            Assumption::InFunc(name) => AssumptionRef::InFunc(name.clone()),
            Assumption::InIf(b, pred, input) => {
                AssumptionRef::InIf(*b, Rc::as_ptr(pred), Rc::as_ptr(input))
            }
            Assumption::InSwitch(branch, pred, input) => {
                AssumptionRef::InSwitch(*branch, Rc::as_ptr(pred), Rc::as_ptr(input))
            }
            Assumption::WildCard(str) => AssumptionRef::WildCard(str.clone()),
        }
    }
}

impl BaseType {
    pub fn contains_state(&self) -> bool {
        match self {
            BaseType::IntT => false,
            BaseType::FloatT => false,
            BaseType::BoolT => false,
            BaseType::PointerT(inner) => {
                assert!(!inner.contains_state(), "Pointers can't contain state");
                false
            }
            BaseType::StateT => true,
        }
    }
}

impl Type {
    pub fn contains_state(&self) -> bool {
        match self {
            Type::Base(basety) => basety.contains_state(),
            Type::TupleT(types) => types.iter().any(|ty| ty.contains_state()),
            Type::Unknown => panic!("Unknown type"),
            Type::Symbolic(_) => panic!("Symbolic type"),
        }
    }
}
