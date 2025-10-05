use crate::schema::{RcExpr, TreeProgram};
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
    #[error("unsupported operation or arity: {0}")]
    UnsupportedHead(String),
}

fn build_expr_from_extraction(
    _serialized: &EGraph,
    tiger: &TigerEGraph,
    extraction: &TigerExtraction,
) -> Result<RcExpr, TigerReconstructError> {
    use crate::schema::Expr::*;
    use crate::schema::{
        Assumption, BinaryOp, Constant as SchemaConstant, TernaryOp, Type as SchemaType, UnaryOp,
    };
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
        // Helpers
        let bop2 = |opv: BinaryOp, c: &Vec<RcExpr>| -> Result<RcExpr, TigerReconstructError> {
            if c.len() != 2 {
                return Err(TigerReconstructError::UnsupportedHead(format!(
                    "{op} arity {} != 2",
                    c.len()
                )));
            }
            Ok(Rc::new(Bop(opv, c[0].clone(), c[1].clone())))
        };
        let uop1 = |opv: UnaryOp, c: &Vec<RcExpr>| -> Result<RcExpr, TigerReconstructError> {
            if c.len() != 1 {
                return Err(TigerReconstructError::UnsupportedHead(format!(
                    "{op} arity {} != 1",
                    c.len()
                )));
            }
            Ok(Rc::new(Uop(opv, c[0].clone())))
        };
        let expr: RcExpr = match op {
            // Flattened function wrapper: take last child as body
            "Function" => {
                if let Some(body) = rc_children.last() {
                    body.clone()
                } else {
                    return Err(TigerReconstructError::UnsupportedHead(
                        "Function(no children)".into(),
                    ));
                }
            }
            // Leaf / scope ops
            "Arg" => Rc::new(Arg(SchemaType::Unknown, Assumption::dummy())),
            "Empty" => Rc::new(Empty(SchemaType::Unknown, Assumption::dummy())),
            // Very conservative Const reconstruction: default value based on child head if available.
            // TODO: Inspect serialized original node to recover literal and context accurately.
            "Const" => {
                // Try to peek at first child (constant constructor head) if present
                let val = if let Some(first_child) = rc_children.first() {
                    // Heuristically pattern match on Debug formatting
                    let s = format!("{}", first_child.as_ref());
                    if s.contains("Bool true") {
                        SchemaConstant::Bool(true)
                    } else if s.contains("Bool false") {
                        SchemaConstant::Bool(false)
                    } else {
                        SchemaConstant::Int(0)
                    }
                } else {
                    SchemaConstant::Int(0)
                };
                Rc::new(Const(val, SchemaType::Unknown, Assumption::dummy()))
            }
            // Ternary ops (Write(ptr,val,state), Select(cond,then,else))
            "Write" => {
                if rc_children.len() != 3 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "Write arity {} != 3",
                        rc_children.len()
                    )));
                }
                Rc::new(Top(
                    TernaryOp::Write,
                    rc_children[0].clone(),
                    rc_children[1].clone(),
                    rc_children[2].clone(),
                ))
            }
            "Select" => {
                if rc_children.len() != 3 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "Select arity {} != 3",
                        rc_children.len()
                    )));
                }
                Rc::new(Top(
                    TernaryOp::Select,
                    rc_children[0].clone(),
                    rc_children[1].clone(),
                    rc_children[2].clone(),
                ))
            }
            // Binary ops
            "Add" => bop2(BinaryOp::Add, &rc_children)?,
            "Sub" => bop2(BinaryOp::Sub, &rc_children)?,
            "Mul" => bop2(BinaryOp::Mul, &rc_children)?,
            "Div" => bop2(BinaryOp::Div, &rc_children)?,
            "Eq" => bop2(BinaryOp::Eq, &rc_children)?,
            "LessThan" => bop2(BinaryOp::LessThan, &rc_children)?,
            "GreaterThan" => bop2(BinaryOp::GreaterThan, &rc_children)?,
            "LessEq" => bop2(BinaryOp::LessEq, &rc_children)?,
            "GreaterEq" => bop2(BinaryOp::GreaterEq, &rc_children)?,
            "Smax" => bop2(BinaryOp::Smax, &rc_children)?,
            "Smin" => bop2(BinaryOp::Smin, &rc_children)?,
            "Shl" => bop2(BinaryOp::Shl, &rc_children)?,
            "Shr" => bop2(BinaryOp::Shr, &rc_children)?,
            "FAdd" => bop2(BinaryOp::FAdd, &rc_children)?,
            "FSub" => bop2(BinaryOp::FSub, &rc_children)?,
            "FMul" => bop2(BinaryOp::FMul, &rc_children)?,
            "FDiv" => bop2(BinaryOp::FDiv, &rc_children)?,
            "FEq" => bop2(BinaryOp::FEq, &rc_children)?,
            "FLessThan" => bop2(BinaryOp::FLessThan, &rc_children)?,
            "FGreaterThan" => bop2(BinaryOp::FGreaterThan, &rc_children)?,
            "FLessEq" => bop2(BinaryOp::FLessEq, &rc_children)?,
            "FGreaterEq" => bop2(BinaryOp::FGreaterEq, &rc_children)?,
            "Fmax" => bop2(BinaryOp::Fmax, &rc_children)?,
            "Fmin" => bop2(BinaryOp::Fmin, &rc_children)?,
            "And" => bop2(BinaryOp::And, &rc_children)?,
            "Or" => bop2(BinaryOp::Or, &rc_children)?,
            "PtrAdd" => bop2(BinaryOp::PtrAdd, &rc_children)?,
            "Load" => bop2(BinaryOp::Load, &rc_children)?,
            "Print" => bop2(BinaryOp::Print, &rc_children)?,
            "Free" => bop2(BinaryOp::Free, &rc_children)?,
            "Bitand" => bop2(BinaryOp::Bitand, &rc_children)?,
            // Unary ops
            "Abs" => uop1(UnaryOp::Abs, &rc_children)?,
            "Not" => uop1(UnaryOp::Not, &rc_children)?,
            "Neg" => uop1(UnaryOp::Neg, &rc_children)?,
            // Simple tuple constructors
            "Single" => {
                if rc_children.len() != 1 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "Single arity {} != 1",
                        rc_children.len()
                    )));
                }
                Rc::new(Single(rc_children[0].clone()))
            }
            "Concat" => {
                if rc_children.len() != 2 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "Concat arity {} != 2",
                        rc_children.len()
                    )));
                }
                Rc::new(Concat(rc_children[0].clone(), rc_children[1].clone()))
            }
            // Control flow (flattened) – placeholders until full reconstruction of constants & indices
            // We currently require exact arities to avoid silent mis-reconstruction.
            "If" => {
                if rc_children.len() != 4 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "If arity {} != 4",
                        rc_children.len()
                    )));
                }
                Rc::new(If(
                    rc_children[0].clone(),
                    rc_children[1].clone(),
                    rc_children[2].clone(),
                    rc_children[3].clone(),
                ))
            }
            "Switch" => {
                // Expect: pred, input, branches... (at least pred,input, one branch)
                if rc_children.len() < 3 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "Switch arity {} < 3",
                        rc_children.len()
                    )));
                }
                let pred = rc_children[0].clone();
                let input = rc_children[1].clone();
                let branches = rc_children[2..].to_vec();
                Rc::new(Switch(pred, input, branches))
            }
            "DoWhile" => {
                if rc_children.len() != 2 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "DoWhile arity {} != 2",
                        rc_children.len()
                    )));
                }
                Rc::new(DoWhile(rc_children[0].clone(), rc_children[1].clone()))
            }
            // Tuple & indexing helpers (Get handled ONLY if 2 children and second is a Const Int below once constants supported)
            "Get" => {
                if rc_children.len() != 2 {
                    return Err(TigerReconstructError::UnsupportedHead(format!(
                        "Get arity {} != 2",
                        rc_children.len()
                    )));
                }
                // Expect rhs to ultimately encode an integer constant; we currently only support small indices 0..=1024 via string parse fallback.
                let mut idx_opt: Option<usize> = None;
                // Attempt to parse from Debug string of second child (e.g., Const (Int N) ...)
                let dbg = format!("{}", rc_children[1].as_ref());
                for tok in dbg.split(|c: char| !c.is_ascii_digit()) {
                    if !tok.is_empty() {
                        if let Ok(v) = tok.parse::<usize>() {
                            idx_opt = Some(v);
                            break;
                        }
                    }
                }
                let idx = idx_opt.ok_or_else(|| {
                    TigerReconstructError::UnsupportedHead("Get(second child not int const)".into())
                })?;
                Rc::new(Get(rc_children[0].clone(), idx))
            }
            other => return Err(TigerReconstructError::UnsupportedHead(other.to_string())),
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
        let mut body = build_expr_from_extraction(serialized, tiger, tex)?;
        // If extraction gave us a full function, unwrap its body.
        if let Expr::Function(_, _in_ty, _out_ty, inner) = body.as_ref() {
            eprintln!("[tiger reconstruct] Unwrapping extracted Function node for {fname} and using its body");
            body = inner.clone();
        }
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
    let prog = TreeProgram {
        entry: entry.unwrap(),
        functions: others,
    };
    // Fill in argument types.
    Ok(prog.with_arg_types())
}
