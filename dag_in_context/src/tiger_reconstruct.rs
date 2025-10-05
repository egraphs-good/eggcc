use crate::schema::{Assumption, BaseType, Expr, RcExpr, TreeProgram, Type};
use crate::tiger_extractor_types::{TigerExtraction, TigerExtractionENode, TigerExtractionResult};
use egraph_serialize::{ClassId, EGraph, NodeId};
use indexmap::IndexMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TigerReconstructError {
    #[error("missing extraction for function root {0}")]
    MissingExtraction(String),
    #[error("no root index in extraction for function root {0}")]
    MissingRoot(String),
    #[error("original node not found in serialized egraph for class {0}")]
    MissingOriginalNode(ClassId),
    #[error("unexpected: child index out of range in extraction graph")]
    ChildIndexRange,
}

fn make_arg_tuple_type(arity: usize) -> Type {
    if arity == 0 {
        Type::TupleT(vec![])
    } else {
        Type::TupleT(vec![BaseType::StateT; arity])
    }
}

fn build_expr_from_extraction(
    serialized: &EGraph,
    extraction: &TigerExtraction,
) -> Result<RcExpr, TigerReconstructError> {
    use crate::schema::Expr::*;
    // Map extraction node index -> RcExpr
    let mut built: Vec<Option<RcExpr>> = vec![None; extraction.nodes.len()];
    // Post-order assumed; build sequentially
    for (idx, en) in extraction.nodes.iter().enumerate() {
        // Determine op kind from original node
        let onid = &en.original_node;
        let node = &serialized[onid];
        // Children: map extraction child indices to RcExpr
        let mut rc_children: Vec<RcExpr> = Vec::new();
        for &ch_idx in &en.children {
            let Some(ch_expr) = &built[ch_idx] else {
                return Err(TigerReconstructError::ChildIndexRange);
            };
            rc_children.push(ch_expr.clone());
        }
        // Build expression heuristically mapping op strings; fallback to Symbolic
        let expr: RcExpr = match node.op.as_str() {
            // Minimal mapping; extend as needed.
            "Add" => Rc::new(Bop(
                crate::schema::BinaryOp::Add,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Sub" => Rc::new(Bop(
                crate::schema::BinaryOp::Sub,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Mul" => Rc::new(Bop(
                crate::schema::BinaryOp::Mul,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Div" => Rc::new(Bop(
                crate::schema::BinaryOp::Div,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "And" => Rc::new(Bop(
                crate::schema::BinaryOp::And,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            "Or" => Rc::new(Bop(
                crate::schema::BinaryOp::Or,
                rc_children[0].clone(),
                rc_children[1].clone(),
            )),
            // Fallback symbolic (Unknown type placeholder)
            _ => Rc::new(Expr::Symbolic(node.op.clone(), Some(Type::Unknown))),
        };
        built[idx] = Some(expr);
    }
    // Root is extraction.root_index
    if let Some(ridx) = extraction.root_index {
        Ok(built[ridx].as_ref().unwrap().clone())
    } else {
        Err(TigerReconstructError::MissingRoot("<unknown>".into()))
    }
}

pub fn reconstruct_program_from_tiger(
    original_prog: &TreeProgram,
    serialized: &EGraph,
    batch: &[String],
    tiger_res: &TigerExtractionResult,
) -> Result<TreeProgram, TigerReconstructError> {
    use crate::schema::Expr;
    let mut new_entry = original_prog.entry.clone();
    let mut new_funcs: IndexMap<String, RcExpr> = IndexMap::new();
    // Build map of existing function signatures
    for f in &original_prog.functions {
        if let Expr::Function(name, _, _, _) = &**f {
            new_funcs.insert(name.clone(), f.clone());
        }
    }
    if let Expr::Function(name, _, _, _) = &*original_prog.entry {
        new_funcs.insert(name.clone(), original_prog.entry.clone());
    }
    for fname in batch {
        // Locate function root eclass id
        // We rely on serialized root retrieval via greedy extractor helper
        let root_cid =
            serialized.nid_to_cid(&crate::greedy_dag_extractor::get_root(serialized, fname));
        let Some(tex) = tiger_res.extractions.get(root_cid) else {
            return Err(TigerReconstructError::MissingExtraction(fname.clone()));
        };
        let body = build_expr_from_extraction(serialized, tex)?;
        // Reuse existing signature if present
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
    // Rebuild program: keep original entry function name
    let mut others: Vec<RcExpr> = Vec::new();
    let mut entry: Option<RcExpr> = None;
    let orig_entry_name = if let Expr::Function(n, _, _, _) = &*original_prog.entry {
        n.clone()
    } else {
        "main".to_string()
    };
    for (_name, fexpr) in new_funcs.into_iter() {
        if let Expr::Function(n, _, _, _) = &*fexpr {
            if *n == orig_entry_name {
                entry = Some(fexpr.clone());
            } else {
                others.push(fexpr.clone());
            }
        }
    }
    // Fallback if entry unchanged
    if entry.is_none() {
        entry = Some(new_entry);
    }
    Ok(TreeProgram {
        entry: entry.unwrap(),
        functions: others,
    })
}
