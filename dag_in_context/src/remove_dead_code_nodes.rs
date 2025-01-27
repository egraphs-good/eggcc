use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::parallel_vec_ty,
    schema::{Expr, RcExpr, TreeProgram, Type},
};

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

fn remove_dead_code_ty(ty: Type, dead_indicies: &[usize]) -> Type {
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

/// Try to split an expression that is a concat into multiple parts.
fn try_split_inputs(expr: RcExpr) -> Option<Vec<RcExpr>> {
    match expr.as_ref() {
        Expr::Concat(left, right) => {
            let mut left_parts = try_split_inputs(left.clone())?;
            let mut right_parts = try_split_inputs(right.clone())?;
            left_parts.append(&mut right_parts);
            Some(left_parts)
        }
        Expr::Single(expr) => Some(vec![expr.clone()]),
        Expr::Empty(_, _) => Some(vec![]),
        _ => None,
    }
}

/// given a vector of inputs, add the non-dead ones to a new vector
/// and return the indicies of the dead ones
fn partition_inputs_and_remove_dead_code(
    inputs: Vec<RcExpr>,
    memo: &mut HashMap<(*const Expr, Vec<usize>), RcExpr>,
    current_dead: &Vec<usize>,
) -> (Vec<RcExpr>, Vec<usize>) {
    let mut new_inputs = vec![];
    let mut new_dead_indicies = vec![];
    for (index, input) in inputs.iter().enumerate() {
        match input.as_ref() {
            Expr::DeadCode(_subexpr) => {
                new_dead_indicies.push(index);
            }
            _ => {
                new_inputs.push(remove_dead_code_expr(input.clone(), memo, current_dead));
            }
        }
    }
    (new_inputs, new_dead_indicies)
}

fn remove_dead_code_expr(
    expr: RcExpr,
    memo: &mut HashMap<(*const Expr, Vec<usize>), RcExpr>,
    dead_indicies: &Vec<usize>,
) -> RcExpr {
    if let Some(new_expr) = memo.get(&(Rc::as_ptr(&expr), dead_indicies.clone())) {
        return new_expr.clone();
    }

    let res = match expr.as_ref() {
        Expr::Const(constant, ty, assumption) => RcExpr::new(Expr::Const(
            constant.clone(),
            remove_dead_code_ty(ty.clone(), dead_indicies),
            assumption.clone(),
        )),
        Expr::Get(expr, index) => {
            // check if the expr is an argument
            match expr.as_ref() {
                Expr::Arg(ty, ctx) => {
                    let new_ty = remove_dead_code_ty(ty.clone(), dead_indicies);
                    let mut new_index = *index;
                    for dead_index in dead_indicies {
                        if dead_index < index {
                            new_index -= 1;
                        }
                    }
                    RcExpr::new(Expr::Get(
                        RcExpr::new(Expr::Arg(new_ty, ctx.clone())),
                        new_index,
                    ))
                }
                _ => {
                    let new_expr = remove_dead_code_expr(expr.clone(), memo, dead_indicies);
                    RcExpr::new(Expr::Get(new_expr, *index))
                }
            }
        }
        Expr::Arg(_, _) => {
            if dead_indicies.is_empty() {
                expr.clone()
            } else {
                panic!("Found argument used directly, but code was supposed to be dead at indicies {:?}", dead_indicies)
            }
        }
        Expr::DoWhile(inputs, body) => {
            // TODO: dead code isn't generated for loops yet, but a fancier
            // extractor could
            RcExpr::new(Expr::DoWhile(
                remove_dead_code_expr(inputs.clone(), memo, dead_indicies),
                remove_dead_code_expr(body.clone(), memo, &vec![]),
            ))
        }
        Expr::If(pred, inputs, then, else_case) => {
            if let Some(split_inputs) = try_split_inputs(inputs.clone()) {
                let (new_inputs, new_dead_indicies) =
                    partition_inputs_and_remove_dead_code(split_inputs, memo, dead_indicies);
                let new_pred = remove_dead_code_expr(pred.clone(), memo, dead_indicies);
                RcExpr::new(Expr::If(
                    new_pred.clone(),
                    parallel_vec_ty(new_inputs, new_pred.get_arg_type()),
                    remove_dead_code_expr(then.clone(), memo, &new_dead_indicies),
                    remove_dead_code_expr(else_case.clone(), memo, &new_dead_indicies),
                ))
            } else {
                RcExpr::new(Expr::If(
                    remove_dead_code_expr(pred.clone(), memo, dead_indicies),
                    remove_dead_code_expr(inputs.clone(), memo, dead_indicies),
                    remove_dead_code_expr(then.clone(), memo, &vec![]),
                    remove_dead_code_expr(else_case.clone(), memo, &vec![]),
                ))
            }
        }
        Expr::Switch(pred, inputs, branches) => {
            if let Some(split_inputs) = try_split_inputs(inputs.clone()) {
                let (new_inputs, new_dead_indicies) =
                    partition_inputs_and_remove_dead_code(split_inputs, memo, dead_indicies);
                let mut new_branches = vec![];
                for branch in branches.iter() {
                    new_branches.push(remove_dead_code_expr(
                        branch.clone(),
                        memo,
                        &new_dead_indicies,
                    ));
                }
                let new_pred = remove_dead_code_expr(pred.clone(), memo, dead_indicies);
                RcExpr::new(Expr::Switch(
                    new_pred.clone(),
                    parallel_vec_ty(new_inputs, new_pred.get_arg_type()),
                    new_branches,
                ))
            } else {
                RcExpr::new(Expr::Switch(
                    remove_dead_code_expr(pred.clone(), memo, dead_indicies),
                    remove_dead_code_expr(inputs.clone(), memo, dead_indicies),
                    branches
                        .iter()
                        .map(|branch| remove_dead_code_expr(branch.clone(), memo, &vec![]))
                        .collect(),
                ))
            }
        }
        Expr::DeadCode(_subexpr) => {
            panic!("Reached dead code without being in inputs of control flow node");
        }
        Expr::Function(_, _, _, _expr) => panic!("Found function inside of function"),
        _ => {
            expr.map_expr_children(|expr| remove_dead_code_expr(expr.clone(), memo, dead_indicies))
        }
    };

    memo.insert((Rc::as_ptr(&expr), dead_indicies.clone()), res.clone());
    res
}
