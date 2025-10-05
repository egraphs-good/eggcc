use crate::schema::{Expr, RcExpr, TreeProgram, Type};
use crate::tiger_extractor_types::{TigerExtraction, TigerExtractionResult};
use crate::tiger_format::TigerEGraph;
use egraph_serialize::EGraph;
use indexmap::IndexMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TigerReconstructError {
    #[error("missing extraction for function root {0}")]
    MissingExtraction(String),
    #[error("no root index in extraction for function root {0}")]
    MissingRoot(String),
    #[error("unexpected: child index out of range in extraction graph")]
    ChildIndexRange,
}

fn build_expr_from_extraction(
    serialized: &EGraph,
    tiger: &TigerEGraph,
    extraction: &TigerExtraction,
) -> Result<RcExpr, TigerReconstructError> {
    use crate::schema::Expr::*;
    let mut built: Vec<Option<RcExpr>> = vec![None; extraction.nodes.len()];
    for (idx, en) in extraction.nodes.iter().enumerate() {
        let t_idx = *tiger
            .class_index
            .get(&en.eclass)
            .expect("eclass missing in tiger graph");
        let ten = &tiger.eclasses[t_idx].enodes[en.enode_index];
        let op = ten.head.as_str();
        let mut rc_children: Vec<RcExpr> = Vec::new();
        for &ch_idx in &en.children {
            let Some(ch_expr) = &built[ch_idx] else {
                return Err(TigerReconstructError::ChildIndexRange);
            };
            rc_children.push(ch_expr.clone());
        }
        let expr: RcExpr = match op {
            "Add" if rc_children.len() == 2 => Rc::new(Bop(
                crate::schema::BinaryOp::Add,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Sub" if rc_children.len() == 2 => Rc::new(Bop(
                crate::schema::BinaryOp::Sub,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Mul" if rc_children.len() == 2 => Rc::new(Bop(
                crate::schema::BinaryOp::Mul,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Div" if rc_children.len() == 2 => Rc::new(Bop(
                crate::schema::BinaryOp::Div,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "And" if rc_children.len() == 2 => Rc::new(Bop(
                crate::schema::BinaryOp::And,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Or" if rc_children.len() == 2 => Rc::new(Bop(
                crate::schema::BinaryOp::Or,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            _ => Rc::new(Expr::Symbolic(ten.head.clone(), Some(Type::Unknown))),
        };
        built[idx] = Some(expr);
    }
    if let Some(ridx) = extraction.root_index {
        Ok(built[ridx].as_ref().unwrap().clone())
    } else {
        Err(TigerReconstructError::MissingRoot("<unknown>".into()))
    }
}

pub fn reconstruct_program_from_tiger(
    original_prog: &TreeProgram,
    serialized: &EGraph,
    tiger: &TigerEGraph,
    batch: &[String],
    tiger_res: &TigerExtractionResult,
) -> Result<TreeProgram, TigerReconstructError> {
    use crate::schema::Expr;
    let mut new_funcs: IndexMap<String, RcExpr> = IndexMap::new();
    for f in &original_prog.functions {
        if let Expr::Function(name, _, _, _) = &**f {
            new_funcs.insert(name.clone(), f.clone());
        }
    }
    if let Expr::Function(name, _, _, _) = &*original_prog.entry {
        new_funcs.insert(name.clone(), original_prog.entry.clone());
    }
    for fname in batch {
        let root_cid =
            serialized.nid_to_cid(&crate::greedy_dag_extractor::get_root(serialized, fname));
        let Some(tex) = tiger_res.extractions.get(root_cid) else {
            return Err(TigerReconstructError::MissingExtraction(fname.clone()));
        };
        let body = build_expr_from_extraction(serialized, tiger, tex)?;
        if let Some(orig_fn) = new_funcs.get(fname).cloned() {
            if let Expr::Function(_, arg_ty, ret_ty, _) = &*orig_fn {
                let replaced = Rc::new(Expr::Function(
                    fname.clone(),
                    arg_ty.clone(),
                    ret_ty.clone(),
                    body,
                ));
                new_funcs.insert(fname.clone(), replaced);
            }
        }
    }
    let orig_entry_name = if let Expr::Function(n, _, _, _) = &*original_prog.entry {
        n.clone()
    } else {
        "main".to_string()
    };
    let mut others: Vec<RcExpr> = Vec::new();
    let mut entry: Option<RcExpr> = None;
    for (_name, fexpr) in new_funcs.into_iter() {
        if let Expr::Function(n, _, _, _) = &*fexpr {
            if *n == orig_entry_name {
                entry = Some(fexpr.clone());
            } else {
                others.push(fexpr.clone());
            }
        }
    }
    if entry.is_none() {
        entry = Some(original_prog.entry.clone());
    }
    Ok(TreeProgram {
        entry: entry.unwrap(),
        functions: others,
    })
}
