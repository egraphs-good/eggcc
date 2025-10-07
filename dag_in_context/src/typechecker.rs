use std::rc::Rc;

use indexmap::IndexMap;

use crate::{
    ast::{base, empty, emptyt, function, program, statet},
    schema::{BaseType, BinaryOp, Constant, Expr, RcExpr, TernaryOp, TreeProgram, Type},
    tuplet,
};

pub(crate) struct TypeStack(Vec<Type>);

impl TypeStack {
    pub(crate) fn get(&self) -> &Type {
        self.0.last().unwrap()
    }

    pub(crate) fn pushed(&self, ty: Type) -> TypeStack {
        let mut new_stack = self.0.clone();
        new_stack.push(ty);
        TypeStack(new_stack)
    }
}

impl TreeProgram {
    pub(crate) fn typecheck(&self) -> TypeCache {
        let mut checker = TypeChecker::new(self, true);
        checker.add_arg_types();
        checker.type_cache
    }

    /// Adds correct types to arguments in the program
    /// and performs type checking.
    /// Maintains the invariant that common subexpressions are shared using
    /// the same Rc<Expr> pointer.
    pub(crate) fn with_arg_types(&self) -> TreeProgram {
        let mut checker = TypeChecker::new(self, false);
        checker.add_arg_types()
    }

    pub fn with_arg_types_and_cache(&self) -> (TreeProgram, TypeCache) {
        let mut checker = TypeChecker::new(self, false);
        let prog = checker.add_arg_types();
        (prog, checker.type_cache)
    }
}

impl Expr {
    #[allow(dead_code)]
    pub(crate) fn with_arg_type(self: RcExpr, input_ty: Type) -> RcExpr {
        // we need a dummy program, since there are no calls in self
        let prog = program!(function("dummy", tuplet!(), tuplet!(), empty()),);
        let mut checker = TypeChecker::new(&prog, false);
        let (_ty, new_expr) =
            checker.add_arg_types_to_expr(self.clone(), &Some(TypeStack(vec![input_ty])));
        new_expr
    }

    /// Performs type checking, and also replaces any `Unknown` types
    /// in arguments with the correct types.
    /// Expects an expression without any call statements.
    #[allow(dead_code)]
    pub(crate) fn with_arg_types(self: RcExpr, input_ty: Type, output_ty: Type) -> RcExpr {
        // we need a dummy program, since there are no calls in self
        let prog = program!(function("dummy", tuplet!(), tuplet!(), empty()),);
        let mut checker = TypeChecker::new(&prog, false);
        let (ty, new_expr) =
            checker.add_arg_types_to_expr(self.clone(), &Some(TypeStack(vec![input_ty])));
        assert_eq!(
            ty, output_ty,
            "Expected return type to be {:?}. Got {:?}",
            output_ty, ty
        );
        new_expr
    }
    /// Adds argument types to the expression.
    #[allow(dead_code)]
    pub(crate) fn add_arg_type(self: RcExpr, input_ty: Type) -> RcExpr {
        // we need a dummy program, since there are no calls in self
        let prog = program!(function("dummy", tuplet!(), tuplet!(), empty()),);
        let mut checker = TypeChecker::new(&prog, false);
        let (_ty, new_expr) =
            checker.add_arg_types_to_expr(self.clone(), &Some(TypeStack(vec![input_ty])));
        new_expr
    }

    #[allow(dead_code)]
    pub(crate) fn func_with_arg_types(self: RcExpr) -> RcExpr {
        match self.as_ref() {
            Expr::Function(name, in_ty, out_ty, body) => RcExpr::new(Expr::Function(
                name.clone(),
                in_ty.clone(),
                out_ty.clone(),
                body.clone().with_arg_types(in_ty.clone(), out_ty.clone()),
            )),
            _ => panic!("Expected function, got {:?}", self),
        }
    }
}

/// Typechecking produces new, typed expressions.
/// This map is used to memoize the results of typechecking.
/// It maps the old untyped expression to the new typed expression
/// The type can be None when `expect_fully_typed` is true.
pub type TypedExprCache = IndexMap<(*const Expr, Option<Type>), RcExpr>;

/// We also need to keep track of the type of the newly typed expression.
/// This maps the newly instrumented expression to its type.
pub type TypeCache = IndexMap<*const Expr, Type>;
/// Type checks program fragments.
/// Uses the program to look up function types.
pub(crate) struct TypeChecker<'a> {
    program: &'a TreeProgram,
    type_cache: TypeCache,
    type_expr_cache: TypedExprCache,
    /// When this is true, the type checker does not perform any inference.
    /// As a result, the type_expr_cache contains expressions from the original program.
    #[allow(dead_code)]
    expect_fully_typed: bool,
}

impl<'a> TypeChecker<'a> {
    pub(crate) fn new(prog: &'a TreeProgram, expect_fully_typed: bool) -> Self {
        TypeChecker {
            program: prog,
            type_cache: IndexMap::new(),
            type_expr_cache: IndexMap::new(),
            expect_fully_typed,
        }
    }

    pub(crate) fn add_arg_types(&mut self) -> TreeProgram {
        TreeProgram {
            entry: self.add_arg_types_to_func(self.program.entry.clone()),
            functions: self
                .program
                .functions
                .iter()
                .map(|expr| self.add_arg_types_to_func(expr.clone()))
                .collect(),
        }
    }

    pub(crate) fn add_arg_types_to_func(&mut self, func: RcExpr) -> RcExpr {
        match func.as_ref() {
            Expr::Function(name, in_ty, out_ty, body) => {
                assert!(
                    !matches!(in_ty, Type::Unknown),
                    "Function {name} has unknown input type"
                );
                assert!(
                    !matches!(out_ty, Type::Unknown),
                    "Function {name} has unknown output type"
                );
                let (expr_ty, new_body) =
                    self.add_arg_types_to_expr(body.clone(), &Some(TypeStack(vec![in_ty.clone()])));
                assert_eq!(
                    expr_ty, *out_ty,
                    "Expected return type to be {:?}. Got {:?}",
                    out_ty, expr_ty
                );
                RcExpr::new(Expr::Function(
                    name.clone(),
                    in_ty.clone(),
                    out_ty.clone(),
                    new_body,
                ))
            }
            _ => panic!("Expected function, got {:?}", func),
        }
    }

    pub(crate) fn add_arg_types_to_expr(
        &mut self,
        expr: RcExpr,
        // the current argument types
        // can be None when `expect_fully_typed` is true
        arg_tys: &Option<TypeStack>,
    ) -> (Type, RcExpr) {
        if let Some(tys) = arg_tys {
            assert!(tys.get() != &Type::Unknown, "Expected known argument type");
        }

        let old_expr_ptr = Rc::as_ptr(&expr);

        if let Some(updated_expr) = self.type_expr_cache.get(&(
            old_expr_ptr,
            arg_tys.as_ref().map(|inner| inner.get().clone()),
        )) {
            let new_expr_ptr = Rc::as_ptr(updated_expr);
            let ty = self.type_cache.get(&new_expr_ptr).unwrap();
            return (ty.clone(), updated_expr.clone());
        }

        let (res_ty, mut res_expr) = match expr.as_ref() {
            Expr::Const(constant, ty, ctx) => {
                let cty = match constant {
                    Constant::Int(_) => Type::Base(BaseType::IntT),
                    Constant::Bool(_) => Type::Base(BaseType::BoolT),
                    Constant::Float(_) => Type::Base(BaseType::FloatT),
                };
                match ty {
                    Type::Unknown => {
                        if self.expect_fully_typed {
                            panic!("Expected type to be known in constant")
                        }
                        (
                            cty.clone(),
                            RcExpr::new(Expr::Const(
                                constant.clone(),
                                arg_tys.as_ref().unwrap().get().clone(),
                                ctx.clone(),
                            )),
                        )
                    }
                    _ => {
                        if let Some(tys) = arg_tys {
                            assert_eq!(
                                tys.get(),
                                ty,
                                "Expected arg type in constant to be {:?}. Got {:?}",
                                tys.get(),
                                ty
                            );
                        }
                        (cty, expr.clone())
                    }
                }
            }
            Expr::Top(TernaryOp::Write, left, right, state) => {
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_tys);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_tys);
                let (_sty, new_state) = self.add_arg_types_to_expr(state.clone(), arg_tys);
                let Type::Base(BaseType::PointerT(innert)) = lty else {
                    panic!("Expected pointer type. Got {:?}", lty)
                };
                let Type::Base(baset) = &rty else {
                    panic!("Expected base type. Got {:?}", rty);
                };
                assert_eq!(
                    *innert,
                    baset.clone(),
                    "Expected right type to be {:?}. Got {:?}",
                    innert,
                    rty
                );
                (
                    base(statet()),
                    RcExpr::new(Expr::Top(TernaryOp::Write, new_left, new_right, new_state)),
                )
            }
            Expr::Top(TernaryOp::Select, c, t, e) => {
                let (cty, new_cond) = self.add_arg_types_to_expr(c.clone(), arg_tys);
                let (tty, new_then) = self.add_arg_types_to_expr(t.clone(), arg_tys);
                let (ety, new_else) = self.add_arg_types_to_expr(e.clone(), arg_tys);
                let Type::Base(BaseType::BoolT) = cty else {
                    panic!("Expected base type. Got {:?}", cty)
                };
                assert_eq!(
                    tty, ety,
                    "Expected then and else types to be the same. Got {:?} and {:?}",
                    tty, ety
                );
                (
                    tty,
                    RcExpr::new(Expr::Top(TernaryOp::Select, new_cond, new_then, new_else)),
                )
            }
            Expr::Bop(BinaryOp::PtrAdd, left, right) => {
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_tys);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_tys);
                let Type::Base(BaseType::PointerT(innert)) = lty else {
                    panic!("Expected pointer type. Got {:?}", lty)
                };
                let Type::Base(BaseType::IntT) = rty else {
                    panic!("Expected int type. Got {:?}", rty)
                };
                (
                    Type::Base(BaseType::PointerT(innert)),
                    RcExpr::new(Expr::Bop(BinaryOp::PtrAdd, new_left, new_right)),
                )
            }
            // covers all cases where the input and output types are concrete
            Expr::Bop(op, left, right) if op.types().is_some() => {
                let (left_expected, right_expected, out_expected) = op.types().unwrap();
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_tys);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_tys);
                assert_eq!(
                    lty, left_expected,
                    "Expected left type to be {:?}. Got {:?}",
                    left_expected, lty
                );
                assert_eq!(
                    rty, right_expected,
                    "Expected right type to be {:?} in {:?}. Got {:?}",
                    right_expected, expr, rty
                );
                (
                    out_expected,
                    RcExpr::new(Expr::Bop(op.clone(), new_left, new_right)),
                )
            }
            // covers all cases where the input and output types are concrete
            Expr::Uop(op, inner) if op.types().is_some() => {
                let (expected_inner, expected_out) = op.types().unwrap();
                let (ity, new_inner) = self.add_arg_types_to_expr(inner.clone(), arg_tys);
                assert_eq!(
                    ity, expected_inner,
                    "Expected inner type to be {:?}. Got {:?}",
                    expected_inner, ity
                );
                (expected_out, RcExpr::new(Expr::Uop(op.clone(), new_inner)))
            }
            Expr::Bop(BinaryOp::Print, inner, state) => {
                let (_ity, new_inner) = self.add_arg_types_to_expr(inner.clone(), arg_tys);
                let (_sty, new_state) = self.add_arg_types_to_expr(state.clone(), arg_tys);
                (
                    base(statet()),
                    RcExpr::new(Expr::Bop(BinaryOp::Print, new_inner, new_state)),
                )
            }
            Expr::Bop(BinaryOp::Load, inner, state) => {
                let (ity, new_inner) = self.add_arg_types_to_expr(inner.clone(), arg_tys);
                let (_sty, new_state) = self.add_arg_types_to_expr(state.clone(), arg_tys);
                let Type::Base(BaseType::PointerT(out_ty)) = ity else {
                    panic!("Expected pointer type. Got {:?}", ity)
                };
                (
                    tuplet!(*out_ty, statet()),
                    RcExpr::new(Expr::Bop(BinaryOp::Load, new_inner, new_state)),
                )
            }
            Expr::Bop(BinaryOp::Free, inner, state) => {
                let (ity, new_inner) = self.add_arg_types_to_expr(inner.clone(), arg_tys);
                let (_sty, new_state) = self.add_arg_types_to_expr(state.clone(), arg_tys);
                let Type::Base(BaseType::PointerT(_out_ty)) = ity else {
                    panic!("Expected pointer type. Got {:?}", ity)
                };
                (
                    base(statet()),
                    RcExpr::new(Expr::Bop(BinaryOp::Free, new_inner, new_state)),
                )
            }
            Expr::Get(child, index) => {
                let (cty, new_child) = self.add_arg_types_to_expr(child.clone(), arg_tys);
                let Type::TupleT(types) = cty.clone() else {
                    panic!("Expected tuple type in {:?}. Got {:?}", child, cty)
                };
                if *index >= types.len() {
                    panic!(
                        "Index out of bounds. Tuple has type {}, index is {}. Expr:\n{}",
                        cty, index, expr
                    );
                }
                let expected_ty = types[*index].clone();
                (
                    Type::Base(expected_ty),
                    RcExpr::new(Expr::Get(new_child, *index)),
                )
            }
            Expr::Alloc(id, amount, state, baset) => {
                let (aty, new_amount) = self.add_arg_types_to_expr(amount.clone(), arg_tys);
                let (_sty, new_state) = self.add_arg_types_to_expr(state.clone(), arg_tys);
                let Type::Base(BaseType::IntT) = aty else {
                    panic!("Expected int type. Got {:?}", aty)
                };
                (
                    tuplet!(baset.clone(), statet()),
                    RcExpr::new(Expr::Alloc(*id, new_amount, new_state, baset.clone())),
                )
            }
            Expr::Call(string, arg) => {
                let (aty, new_arg) = self.add_arg_types_to_expr(arg.clone(), arg_tys);
                let func = self
                    .program
                    .get_function(string)
                    .unwrap_or_else(|| panic!("Function {string} should exist in program."));
                assert_eq!(
                    aty,
                    func.func_input_ty().unwrap(),
                    "Expected argument type to be {:?}. Got {:?}",
                    func.func_input_ty().unwrap(),
                    aty
                );
                (
                    func.func_output_ty().unwrap(),
                    RcExpr::new(Expr::Call(string.clone(), new_arg)),
                )
            }
            Expr::Empty(ty, ctx) => match ty {
                Type::Unknown => {
                    if self.expect_fully_typed {
                        panic!("Expected type to be known in empty")
                    }
                    (
                        emptyt(),
                        RcExpr::new(Expr::Empty(
                            arg_tys.as_ref().unwrap().get().clone(),
                            ctx.clone(),
                        )),
                    )
                }
                _ => {
                    if let Some(arg_tys) = arg_tys {
                        assert_eq!(
                            arg_tys.get(),
                            ty,
                            "Expected arg type in empty to be {:?}. Got {:?}",
                            arg_tys.get(),
                            ty
                        );
                    }
                    (emptyt(), expr.clone())
                }
            },
            Expr::Single(arg) => {
                let (Type::Base(basety), new_arg) =
                    self.add_arg_types_to_expr(arg.clone(), arg_tys)
                else {
                    panic!("Expected base type in child of Single. Got {:?}", arg)
                };
                (
                    Type::TupleT(vec![basety]),
                    RcExpr::new(Expr::Single(new_arg)),
                )
            }
            Expr::Concat(left, right) => {
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_tys);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_tys);
                let Type::TupleT(ltypes) = lty else {
                    panic!(
                        "Expected tuple type. Got {:?}. Left Expr:{} Right Expr: {}",
                        lty, left, right
                    )
                };
                let Type::TupleT(rtypes) = rty else {
                    panic!("Expected tuple type. Got {:?}", rty)
                };
                let result_types = ltypes.into_iter().chain(rtypes).collect();
                (
                    Type::TupleT(result_types),
                    RcExpr::new(Expr::Concat(new_left, new_right)),
                )
            }
            Expr::Switch(integer, input, branches) => {
                let (ity, new_integer) = self.add_arg_types_to_expr(integer.clone(), arg_tys);
                let (inputty, new_input) = self.add_arg_types_to_expr(input.clone(), arg_tys);
                let Type::Base(BaseType::IntT) = ity else {
                    panic!("Expected int type. Got {:?}", ity)
                };
                let mut new_branches = vec![];
                let mut res_type = None;
                for branch in branches {
                    let (bty, new_branch) = self.add_arg_types_to_expr(
                        branch.clone(),
                        &arg_tys.as_ref().map(|inner| inner.pushed(inputty.clone())),
                    );
                    new_branches.push(new_branch);
                    res_type = match res_type {
                        Some(t) => {
                            assert_eq!(t, bty, "Expected all branches to have the same type");
                            Some(t)
                        }
                        None => Some(bty),
                    };
                }
                (
                    res_type.unwrap(),
                    RcExpr::new(Expr::Switch(new_integer, new_input, new_branches)),
                )
            }
            Expr::If(pred, input, then, else_branch) => {
                let (pty, new_pred) = self.add_arg_types_to_expr(pred.clone(), arg_tys);
                let (ity, new_input) = self.add_arg_types_to_expr(input.clone(), arg_tys);
                let Type::Base(BaseType::BoolT) = pty else {
                    panic!("Expected bool type. Got {:?}", pty)
                };
                let (tty, new_then) = self.add_arg_types_to_expr(
                    then.clone(),
                    &arg_tys.as_ref().map(|inner| inner.pushed(ity.clone())),
                );
                let (ety, new_else) = self.add_arg_types_to_expr(
                    else_branch.clone(),
                    &arg_tys.as_ref().map(|inner| inner.pushed(ity)),
                );
                assert_eq!(
                    tty, ety,
                    "Expected then and else types to be the same. Got {:?} and {:?}",
                    tty, ety
                );
                (
                    tty,
                    RcExpr::new(Expr::If(new_pred, new_input, new_then, new_else)),
                )
            }
            Expr::DoWhile(inputs, pred_and_outputs) => {
                let (ity, new_inputs) = self.add_arg_types_to_expr(inputs.clone(), arg_tys);
                let Type::TupleT(in_tys) = ity.clone() else {
                    panic!("Expected tuple type. Got {:?}", ity)
                };
                let (pty, new_pred_and_outputs) = self.add_arg_types_to_expr(
                    pred_and_outputs.clone(),
                    &arg_tys.as_ref().map(|inner| inner.pushed(ity)),
                );
                let Type::TupleT(out_tys) = pty else {
                    panic!("Expected tuple type. Got {:?}", pty)
                };
                assert_eq!(
                    out_tys[0],
                    BaseType::BoolT,
                    "Expected first output type to be bool"
                );
                assert_eq!(
                    in_tys,
                    out_tys[1..],
                    "Expected output types to match input types"
                );
                (
                    Type::TupleT(out_tys[1..].to_vec()),
                    RcExpr::new(Expr::DoWhile(new_inputs, new_pred_and_outputs)),
                )
            }
            // Replace the argument type with the new type
            Expr::Arg(Type::Unknown, ctx) => {
                if self.expect_fully_typed {
                    panic!("Expected type to be known in arg")
                }
                (
                    arg_tys.as_ref().unwrap().get().clone(),
                    Rc::new(Expr::Arg(
                        arg_tys.as_ref().unwrap().get().clone(),
                        ctx.clone(),
                    )),
                )
            }
            Expr::Arg(found_ty, _ctx) => {
                if let Some(arg_tys) = arg_tys {
                    assert_eq!(
                        &found_ty,
                        &arg_tys.get(),
                        "Expected argument type to be {:?}. Got {:?}",
                        arg_tys.get(),
                        found_ty
                    );
                }
                (found_ty.clone(), expr.clone())
            }
            Expr::Function(_, _, _, _) => panic!("Expected expression, got function"),
            Expr::Symbolic(_, ty) => (
                ty.clone()
                    .expect("symbolic expression missing type annotation"),
                expr.clone(),
            ),
            // should have covered all cases, but rust can't prove it
            // due to the side conditions
            _ => panic!("Unexpected expression {:?}", expr.clone()),
        };
        if self.expect_fully_typed {
            res_expr = expr.clone();
        }

        let new_expr_ptr: *const Expr = Rc::as_ptr(&res_expr);
        self.type_expr_cache.insert(
            (
                old_expr_ptr,
                arg_tys.as_ref().map(|inner| inner.get().clone()),
            ),
            res_expr.clone(),
        );
        self.type_cache.insert(new_expr_ptr, res_ty.clone());

        (res_ty, res_expr)
    }

    pub(crate) fn get_arg_type(expr: &RcExpr) -> Type {
        match expr.as_ref() {
            Expr::Arg(ty, _) => ty.clone(),
            Expr::Const(_, ty, _) => ty.clone(),
            Expr::Top(_, rc, _, _) => Self::get_arg_type(rc),
            Expr::Bop(_, left, _) => Self::get_arg_type(left),
            Expr::Uop(_, inner) => Self::get_arg_type(inner),
            Expr::Get(child, _) => Self::get_arg_type(child),
            Expr::Alloc(_, amount, _, _) => Self::get_arg_type(amount),
            Expr::Call(_, arg) => Self::get_arg_type(arg),
            Expr::Empty(ty, _) => ty.clone(),
            Expr::Single(arg) => Self::get_arg_type(arg),
            Expr::Concat(left, _) => Self::get_arg_type(left),
            Expr::Switch(_, input, _) => Self::get_arg_type(input),
            Expr::If(pred, _, _, _) => Self::get_arg_type(pred),
            Expr::DoWhile(inputs, _) => Self::get_arg_type(inputs),
            Expr::Function(_, inty, _, _) => inty.clone(),
            Expr::Symbolic(_, _ty) => panic!("Found symbolic expr in get_arg_type"),
        }
    }
}
