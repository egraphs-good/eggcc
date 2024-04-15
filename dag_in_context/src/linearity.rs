//! This file contains helpers for making the extracted
//! program use memory linearly.

use std::collections::HashMap;

use egglog::{match_term_app, Term, TermDag};
use egraph_serialize::{ClassId, EGraph};

use crate::schema::{Expr, Type, *};

type EffectfulNode = Term;
type EffectfulNodes = Vec<EffectfulNode>;

/// Finds all the effectful nodes along the state
/// edge path (the path of the state edge from the argument to the return value).
/// Input: a term representing the program
/// Output: a vector of terms representing the effectful nodes along the state edge path
pub fn find_all_effectful_nodes_in_program(termdag: &mut TermDag, term: &Term, egraph: &EGraph) {
    match_term_app!(term.clone(); {
        ("Program", [main, functions]) => {

        },
        _ => panic!("Expected a Program term")
    });

    todo!()
}

fn find_effectul_in_function(fun_body: Expr, pos: usize) -> EffectfulNodes {
    match fun_body {
        Expr::Const(_, _) => todo!(),
        Expr::Top(_, _, _, _) => todo!(),
        Expr::Bop(_, _, _) => todo!(),
        Expr::Uop(_, _) => todo!(),
        Expr::Get(_, _) => todo!(),
        Expr::Alloc(_, _, _, _) => todo!(),
        Expr::Call(_, _) => todo!(),
        Expr::Empty(_) => todo!(),
        Expr::Single(_) => todo!(),
        Expr::Concat(e1, e2) => {
            let ty1: Type =  todo!("give me the type of e1");
            let Type::TupleT(typs1) = ty1 else {panic!("Expected a Tuple type")};
            if typs1.len() > pos {
                find_effectul_in_function(*e1, pos)
            } else {
                find_effectul_in_function(*e2, pos - typs1.len())
            }
        },
        Expr::If(pred, inps, _, _) => todo!(),
        Expr::Switch(_, _, _) => todo!(),
        Expr::DoWhile(_, _) => todo!(),
        Expr::Arg(_) => todo!(),
        Expr::InContext(_, _) => todo!(),
        Expr::Function(_, _, _, _) => todo!(),
    }
}

fn get_function(term: Term, termdag: &TermDag) -> EffectfulNodes {
    match_term_app!(term; {
    ("Function", [name, np_ty, out_ty, body]) => {
        let inp_ty: Type = todo!();
        let Type::TupleT(inp_tys) = inp_ty else {panic!("Expected a Tuple type")};

        // Step 1: get where the state edge is
        let mut state_type = vec![];
        for (i, ty) in inp_tys.iter().enumerate() {
            if matches!(ty, BaseType::StateT) {
                state_type.push(i);
            }
        }
        if state_type.len() == 0 {
            return vec![];
        }
        assert_eq!(state_type.len(), 1);
        let state_type_pos = state_type[0];

        let body: Expr = todo!("{:?}", termdag.get(*body));
        find_effectul_in_function(body, state_type_pos)
    },
    _ => panic!("Expected a Function term")
    })
}

fn get_all_functions(term: Term, termdag: &TermDag) -> Vec<EffectfulNodes> {
    match_term_app!(term; {
        ("Cons", [x, xs]) => {
            let x = termdag.get(*x);
            let xs = termdag.get(*xs);
            let function = get_function(x, termdag);
            let mut rest = get_all_functions(xs, termdag);
            rest.insert(0, function);
            rest
        },
        ("Nil", []) => {
            vec![]
        },
        _ => panic!("Expected a List term")
    })
}
