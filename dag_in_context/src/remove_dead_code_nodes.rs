use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    ast::parallel_vec_ty,
    schema::{Expr, RcExpr, TreeProgram, Type},
};

struct DeadCodeRemover {
    memo: HashMap<(*const Expr, Vec<usize>), RcExpr>,
    indices_used: HashMap<*const Expr, Vec<usize>>,
}

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
            let mut remover = DeadCodeRemover {
                memo: HashMap::new(),
                indices_used: HashMap::new(),
            };
            let new_body = remover.remove_dead_code_expr(body.clone(), &vec![]);
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

impl DeadCodeRemover {
    fn indices_used(&mut self, expr: RcExpr) -> HashSet<usize> {
        if let Some(indices) = self.indices_used.get(&Rc::as_ptr(&expr)) {
            return indices.iter().cloned().collect();
        }
        let mut res = HashSet::new();
        match expr.as_ref() {
            Expr::Get(expr, index) => match expr.as_ref() {
                Expr::Arg(_ty, _ctx) => {
                    res.insert(*index);
                }
                _ => {
                    res.extend(self.indices_used(expr.clone()));
                }
            },
            Expr::Arg(ty, _ctx) => {
                // all of them are used, add one per length of tuple
                match ty {
                    Type::TupleT(vec) => {
                        for i in 0..vec.len() {
                            res.insert(i);
                        }
                    }
                    _ => {
                        res.insert(0);
                    }
                }
            }
            _ => {
                for expr in expr.children_same_scope() {
                    res.extend(self.indices_used(expr));
                }
            }
        }
        self.indices_used
            .insert(Rc::as_ptr(&expr), res.iter().cloned().collect());
        res
    }

    /// given a vector of inputs, add the non-dead ones to a new vector
    /// and return the indicies of the dead ones
    fn partition_inputs_and_remove_dead_code(
        &mut self,
        inputs: Vec<RcExpr>,
        regions: Vec<RcExpr>,
        current_dead: &Vec<usize>,
    ) -> (Vec<RcExpr>, Vec<usize>) {
        let indices_used = regions
            .iter()
            .map(|region| self.indices_used(region.clone()))
            .fold(HashSet::new(), |mut acc, used| {
                acc.extend(used);
                acc
            });

        let mut new_inputs = vec![];
        let mut new_dead_indicies = vec![];
        for (i, input) in inputs.iter().enumerate() {
            if indices_used.contains(&i) {
                new_inputs.push(self.remove_dead_code_expr(input.clone(), current_dead));
            } else {
                new_dead_indicies.push(i);
            }
        }

        (new_inputs, new_dead_indicies)
    }

    fn remove_dead_code_expr(&mut self, expr: RcExpr, dead_indicies: &Vec<usize>) -> RcExpr {
        if let Some(new_expr) = self.memo.get(&(Rc::as_ptr(&expr), dead_indicies.clone())) {
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
                        // if the index is dead, panic
                        if dead_indicies.contains(index) {
                            panic!("Found dead code in argument");
                        }

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
                        let new_expr = self.remove_dead_code_expr(expr.clone(), dead_indicies);
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
                    self.remove_dead_code_expr(inputs.clone(), dead_indicies),
                    self.remove_dead_code_expr(body.clone(), &vec![]),
                ))
            }
            Expr::If(pred, inputs, then, else_case) => {
                if let Some(split_inputs) = try_split_inputs(inputs.clone()) {
                    let (new_inputs, new_dead_indicies) = self
                        .partition_inputs_and_remove_dead_code(
                            split_inputs,
                            vec![then.clone(), else_case.clone()],
                            dead_indicies,
                        );
                    let new_pred = self.remove_dead_code_expr(pred.clone(), dead_indicies);
                    RcExpr::new(Expr::If(
                        new_pred.clone(),
                        parallel_vec_ty(new_inputs, new_pred.get_arg_type()),
                        self.remove_dead_code_expr(then.clone(), &new_dead_indicies),
                        self.remove_dead_code_expr(else_case.clone(), &new_dead_indicies),
                    ))
                } else {
                    RcExpr::new(Expr::If(
                        self.remove_dead_code_expr(pred.clone(), dead_indicies),
                        self.remove_dead_code_expr(inputs.clone(), dead_indicies),
                        self.remove_dead_code_expr(then.clone(), &vec![]),
                        self.remove_dead_code_expr(else_case.clone(), &vec![]),
                    ))
                }
            }
            Expr::Switch(pred, inputs, branches) => {
                if let Some(split_inputs) = try_split_inputs(inputs.clone()) {
                    let (new_inputs, new_dead_indicies) = self
                        .partition_inputs_and_remove_dead_code(
                            split_inputs,
                            branches.clone(),
                            dead_indicies,
                        );
                    let mut new_branches = vec![];
                    for branch in branches.iter() {
                        new_branches
                            .push(self.remove_dead_code_expr(branch.clone(), &new_dead_indicies));
                    }
                    let new_pred = self.remove_dead_code_expr(pred.clone(), dead_indicies);
                    RcExpr::new(Expr::Switch(
                        new_pred.clone(),
                        parallel_vec_ty(new_inputs, new_pred.get_arg_type()),
                        new_branches,
                    ))
                } else {
                    RcExpr::new(Expr::Switch(
                        self.remove_dead_code_expr(pred.clone(), dead_indicies),
                        self.remove_dead_code_expr(inputs.clone(), dead_indicies),
                        branches
                            .iter()
                            .map(|branch| self.remove_dead_code_expr(branch.clone(), &vec![]))
                            .collect(),
                    ))
                }
            }
            Expr::Function(_, _, _, _expr) => panic!("Found function inside of function"),
            _ => expr
                .map_expr_children(|expr| self.remove_dead_code_expr(expr.clone(), dead_indicies)),
        };

        self.memo
            .insert((Rc::as_ptr(&expr), dead_indicies.clone()), res.clone());
        res
    }
}
