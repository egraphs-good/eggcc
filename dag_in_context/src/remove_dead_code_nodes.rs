use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::schema::{Expr, RcExpr, TreeProgram, Type};

impl TreeProgram {
    pub fn remove_dead_code_nodes(&mut self) {
        for func in self.functions.iter_mut() {
            *func = remove_dead_code_fn(func.clone());
        }
        self.entry = remove_dead_code_fn(self.entry.clone());
    }
}

fn remove_dead_code_fn(func: RcExpr) -> RcExpr {
    match func.as_ref() {
        Expr::Function(name, ret_type, arg_type, body) => {
            let mut memo = HashMap::new();
            let new_body = remove_dead_code_expr(body.clone(), &mut memo, &vec![]);
            RcExpr::new(Expr::Function(
                name.clone(),
                ret_type.clone(),
                arg_type.clone(),
                new_body,
            ))
        }
        _ => panic!("Expected function, got {:?}", func),
    }
}

fn remove_dead_code_ty(ty: Type, dead_indicies: &Vec<usize>) -> Type {
    match ty {
        Type::Base(base_type) => {
            assert!(dead_indicies.is_empty());
            Type::Base(base_type)
        }
        Type::TupleT(vec) => {
            let mut new_vec = vec![];
            for (i, ty) in vec.iter().enumerate() {
                if !dead_indicies.contains(&i) {
                    new_vec.push(ty.clone());
                }
            }
            Type::TupleT(new_vec)
        }
        Type::Unknown => Type::Unknown,
        Type::Symbolic(s) => Type::Symbolic(s.clone()),
    }
}

fn remove_dead_code_expr(
    expr: RcExpr,
    memo: HashMap<(*const Expr, Vec<usize>), RcExpr>,
    dead_indicies: &Vec<usize>,
) -> RcExpr {
    if let Some(new_expr) = memo.get(&(Rc::as_ptr(&expr), dead_indicies.clone())) {
        return new_expr.clone();
    }

    match expr.as_ref() {
        Expr::Const(constant, ty, assumption) => RcExpr::new(Expr::Const(
            constant.clone(),
            remove_dead_code_ty(ty.clone(), dead_indicies),
            assumption.clone(),
        )),
        Expr::Get(expr, _) => todo!(),
        Expr::Top(ternary_op, expr, expr1, expr2) => todo!(),
        Expr::Bop(binary_op, expr, expr1) => todo!(),
        Expr::Uop(unary_op, expr) => todo!(),
        Expr::Alloc(_, expr, expr1, base_type) => todo!(),
        Expr::Call(_, expr) => todo!(),
        Expr::Empty(_, assumption) => todo!(),
        Expr::Single(expr) => todo!(),
        Expr::Concat(expr, expr1) => todo!(),
        Expr::If(expr, expr1, expr2, expr3) => todo!(),
        Expr::Switch(expr, expr1, vec) => todo!(),
        Expr::DoWhile(expr, expr1) => todo!(),
        Expr::Arg(_, assumption) => todo!(),
        Expr::Function(_, _, _, expr) => todo!(),
        Expr::DeadCode() => todo!(),
        Expr::Symbolic(_, _) => todo!(),
    }
}
