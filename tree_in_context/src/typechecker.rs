use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::emptyt,
    schema::{BaseType, BinaryOp, Constant, Expr, RcExpr, TreeProgram, Type, UnaryOp},
};

impl TreeProgram {
    /// Adds correct types to arguments in the program
    /// and performs type checking.
    pub(crate) fn with_arg_types(&self) -> TreeProgram {
        let mut checker = TypeChecker::new(self);
        checker.add_arg_types()
    }

    pub fn with_arg_types_and_cache(&self) -> (TreeProgram, TypeCache) {
        let mut checker = TypeChecker::new(self);
        let prog = checker.add_arg_types();
        (prog, checker.type_cache)
    }
}

impl Expr {
    /// Performs type checking, and also replaces any `Unknown` types
    /// in arguments with the correct types.
    /// TODO remove dead code after use in translation
    #[allow(dead_code)]
    pub(crate) fn with_arg_types(self: RcExpr, input_ty: Type, output_ty: Type) -> RcExpr {
        let prog = self.to_program(input_ty.clone(), output_ty.clone());
        let mut checker = TypeChecker::new(&prog);
        let (ty, new_expr) = checker.add_arg_types_to_expr(self.clone(), &input_ty);
        assert_eq!(
            ty, output_ty,
            "Expected return type to be {:?}. Got {:?}",
            output_ty, ty
        );
        new_expr
    }

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

pub type TypeCache = HashMap<*const Expr, Type>;
/// Type checks program fragments.
/// Uses the program to look up function types.
pub(crate) struct TypeChecker<'a> {
    program: &'a TreeProgram,
    type_cache: TypeCache,
}

impl<'a> TypeChecker<'a> {
    pub(crate) fn new(prog: &'a TreeProgram) -> Self {
        TypeChecker {
            program: prog,
            type_cache: HashMap::new(),
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
                let (expr_ty, new_body) = self.add_arg_types_to_expr(body.clone(), in_ty);
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

    pub(crate) fn add_arg_types_to_expr(&mut self, expr: RcExpr, arg_ty: &Type) -> (Type, RcExpr) {
        assert!(arg_ty != &Type::Unknown, "Expected known argument type");
        let expr_ptr = Rc::as_ptr(&expr);
        let (res_ty, res_expr) = match expr.as_ref() {
            Expr::Const(constant, ty) => {
                let cty = match constant {
                    Constant::Int(_) => Type::Base(BaseType::IntT),
                    Constant::Bool(_) => Type::Base(BaseType::BoolT),
                };
                match ty {
                    Type::Unknown => (
                        cty.clone(),
                        RcExpr::new(Expr::Const(constant.clone(), arg_ty.clone())),
                    ),
                    _ => {
                        assert_eq!(
                            arg_ty, ty,
                            "Expected arg type in constant to be {:?}. Got {:?}",
                            arg_ty, ty
                        );
                        (cty, expr)
                    }
                }
            }
            Expr::Bop(BinaryOp::Write, left, right) => {
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_ty);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_ty);
                let Type::PointerT(innert) = lty else {
                    panic!("Expected pointer type. Got {:?}", lty)
                };
                let Type::Base(baset) = &rty else {
                    todo!("Support pointers to pointers");
                };
                assert_eq!(
                    innert,
                    baset.clone(),
                    "Expected right type to be {:?}. Got {:?}",
                    innert,
                    rty
                );
                (
                    emptyt(),
                    RcExpr::new(Expr::Bop(BinaryOp::Write, new_left, new_right)),
                )
            }
            Expr::Bop(BinaryOp::PtrAdd, left, right) => {
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_ty);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_ty);
                let Type::PointerT(innert) = lty else {
                    panic!("Expected pointer type. Got {:?}", lty)
                };
                let Type::Base(BaseType::IntT) = rty else {
                    panic!("Expected int type. Got {:?}", rty)
                };
                (
                    Type::PointerT(innert),
                    RcExpr::new(Expr::Bop(BinaryOp::PtrAdd, new_left, new_right)),
                )
            }
            // covers all cases where the input and output types are concrete
            Expr::Bop(op, left, right) if op.types().is_some() => {
                let (left_expected, right_expected, out_expected) = op.types().unwrap();
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_ty);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_ty);
                assert_eq!(
                    lty, left_expected,
                    "Expected left type to be {:?}. Got {:?}",
                    left_expected, lty
                );
                assert_eq!(
                    rty, right_expected,
                    "Expected right type to be {:?}. Got {:?}",
                    right_expected, rty
                );
                (
                    out_expected,
                    RcExpr::new(Expr::Bop(op.clone(), new_left, new_right)),
                )
            }
            // covers all cases where the input and output types are concrete
            Expr::Uop(op, inner) if op.types().is_some() => {
                let (expected_inner, expected_out) = op.types().unwrap();
                let (ity, new_inner) = self.add_arg_types_to_expr(inner.clone(), arg_ty);
                assert_eq!(
                    ity, expected_inner,
                    "Expected inner type to be {:?}. Got {:?}",
                    expected_inner, ity
                );
                (expected_out, RcExpr::new(Expr::Uop(op.clone(), new_inner)))
            }
            Expr::Uop(UnaryOp::Print, inner) => {
                let (_ity, new_inner) = self.add_arg_types_to_expr(inner.clone(), arg_ty);
                (emptyt(), RcExpr::new(Expr::Uop(UnaryOp::Print, new_inner)))
            }
            Expr::Uop(UnaryOp::Load, inner) => {
                let (ity, new_inner) = self.add_arg_types_to_expr(inner.clone(), arg_ty);
                let Type::PointerT(out_ty) = ity else {
                    panic!("Expected pointer type. Got {:?}", ity)
                };
                (
                    Type::Base(out_ty),
                    RcExpr::new(Expr::Uop(UnaryOp::Load, new_inner)),
                )
            }
            Expr::Get(child, index) => {
                let (cty, new_child) = self.add_arg_types_to_expr(child.clone(), arg_ty);
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
                (expected_ty, RcExpr::new(Expr::Get(new_child, *index)))
            }
            Expr::Alloc(amount, ty) => {
                let (aty, new_amount) = self.add_arg_types_to_expr(amount.clone(), arg_ty);
                let Type::Base(BaseType::IntT) = aty else {
                    panic!("Expected int type. Got {:?}", aty)
                };
                let Type::Base(baset) = ty else {
                    todo!("Support pointers to pointers");
                };
                (
                    Type::PointerT(baset.clone()),
                    RcExpr::new(Expr::Alloc(new_amount, ty.clone())),
                )
            }
            Expr::Call(string, arg) => {
                let (aty, new_arg) = self.add_arg_types_to_expr(arg.clone(), arg_ty);
                let func = self.program.get_function(string).unwrap();
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
            Expr::Empty(ty) => match ty {
                Type::Unknown => (emptyt(), RcExpr::new(Expr::Empty(arg_ty.clone()))),
                _ => {
                    assert_eq!(
                        arg_ty, ty,
                        "Expected arg type in empty to be {:?}. Got {:?}",
                        arg_ty, ty
                    );
                    (emptyt(), expr)
                }
            },
            Expr::Single(arg) => {
                let (ty, new_arg) = self.add_arg_types_to_expr(arg.clone(), arg_ty);
                (Type::TupleT(vec![ty]), RcExpr::new(Expr::Single(new_arg)))
            }
            Expr::Concat(order, left, right) => {
                let (lty, new_left) = self.add_arg_types_to_expr(left.clone(), arg_ty);
                let (rty, new_right) = self.add_arg_types_to_expr(right.clone(), arg_ty);
                let Type::TupleT(ltypes) = lty else {
                    panic!("Expected tuple type. Got {:?}", lty)
                };
                let Type::TupleT(rtypes) = rty else {
                    panic!("Expected tuple type. Got {:?}", rty)
                };
                let result_types = ltypes.into_iter().chain(rtypes).collect();
                (
                    Type::TupleT(result_types),
                    RcExpr::new(Expr::Concat(order.clone(), new_left, new_right)),
                )
            }
            Expr::Switch(integer, branches) => {
                let (ity, new_integer) = self.add_arg_types_to_expr(integer.clone(), arg_ty);
                let Type::Base(BaseType::IntT) = ity else {
                    panic!("Expected int type. Got {:?}", ity)
                };
                let mut new_branches = vec![];
                let mut res_type = None;
                for branch in branches {
                    let (bty, new_branch) = self.add_arg_types_to_expr(branch.clone(), arg_ty);
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
                    RcExpr::new(Expr::Switch(new_integer, new_branches)),
                )
            }
            Expr::If(pred, then, else_branch) => {
                let (pty, new_pred) = self.add_arg_types_to_expr(pred.clone(), arg_ty);
                let Type::Base(BaseType::BoolT) = pty else {
                    panic!("Expected bool type. Got {:?}", pty)
                };
                let (tty, new_then) = self.add_arg_types_to_expr(then.clone(), arg_ty);
                let (ety, new_else) = self.add_arg_types_to_expr(else_branch.clone(), arg_ty);
                assert_eq!(
                    tty, ety,
                    "Expected then and else types to be the same. Got {:?} and {:?}",
                    tty, ety
                );
                (tty, RcExpr::new(Expr::If(new_pred, new_then, new_else)))
            }
            Expr::Let(input, body) => {
                let (ity, new_input) = self.add_arg_types_to_expr(input.clone(), arg_ty);
                let (bty, new_body) = self.add_arg_types_to_expr(body.clone(), &ity);
                (bty, RcExpr::new(Expr::Let(new_input, new_body)))
            }
            Expr::DoWhile(inputs, pred_and_outputs) => {
                let (ity, new_inputs) = self.add_arg_types_to_expr(inputs.clone(), arg_ty);
                let Type::TupleT(in_tys) = ity.clone() else {
                    panic!("Expected tuple type. Got {:?}", ity)
                };
                let (pty, new_pred_and_outputs) =
                    self.add_arg_types_to_expr(pred_and_outputs.clone(), &ity);
                let Type::TupleT(out_tys) = pty else {
                    panic!("Expected tuple type. Got {:?}", pty)
                };
                assert_eq!(
                    out_tys[0],
                    Type::Base(BaseType::BoolT),
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
            Expr::Arg(Type::Unknown) => (arg_ty.clone(), Rc::new(Expr::Arg(arg_ty.clone()))),
            Expr::Arg(found_ty) => {
                assert_eq!(
                    &found_ty, &arg_ty,
                    "Expected argument type to be {:?}. Got {:?}",
                    arg_ty, found_ty
                );
                (arg_ty.clone(), expr)
            }
            Expr::InContext(assumption, body) => {
                let (bty, new_body) = self.add_arg_types_to_expr(body.clone(), arg_ty);
                (
                    bty,
                    RcExpr::new(Expr::InContext(assumption.clone(), new_body)),
                )
            }
            Expr::Function(_, _, _, _) => panic!("Expected expression, got function"),
            // should have covered all cases, but rust can't prove it
            // due to the side conditions
            _ => panic!("Unexpected expression {:?}", expr),
        };

        self.type_cache.insert(expr_ptr, res_ty.clone());

        (res_ty, res_expr)
    }
}
