//! Convert a tiger `Extraction` into an egglog term.
//!
//! Replaces the C++ `toegglog.cpp`, which printed an egglog program that eggcc
//! then re-ran. Here we build the terms in memory instead.

use egglog::{TermDag, TermId};

use crate::egraphin::{EGraph, Extraction};

/// Build the term for extraction `e` of e-graph `g`. The extraction is in
/// topological order with children first; its last entry is the root.
///
/// `Arg`, `Const` and `Empty` carry placeholder type and context children
/// (`(TupleT (TNil))` and `(DumC)`), mirroring the C++ output; eggcc restores the
/// real types afterwards with `override_arg_types`.
pub fn extraction_to_term(g: &EGraph, e: &Extraction, termdag: &mut TermDag) -> TermId {
    let tnil = termdag.app("TNil".to_string(), vec![]);
    let dum_t = termdag.app("TupleT".to_string(), vec![tnil]);
    let dum_c = termdag.app("DumC".to_string(), vec![]);

    let mut terms: Vec<TermId> = Vec::with_capacity(e.len());
    for en in e {
        let n = &g.eclasses[en.c as usize].enodes[en.n as usize];
        let term = if let Some(lit) = &n.lit {
            termdag.lit(lit.clone())
        } else {
            let children: Vec<TermId> = en.ch.iter().map(|&c| terms[c as usize]).collect();
            match n.op.as_str() {
                "Arg" => {
                    assert!(children.is_empty());
                    termdag.app("Arg".to_string(), vec![dum_t, dum_c])
                }
                "Const" => {
                    assert_eq!(children.len(), 1);
                    termdag.app("Const".to_string(), vec![children[0], dum_t, dum_c])
                }
                "Empty" => {
                    assert!(children.is_empty());
                    termdag.app("Empty".to_string(), vec![dum_t, dum_c])
                }
                op => termdag.app(op.to_string(), children),
            }
        };
        terms.push(term);
    }
    *terms.last().expect("extraction is empty")
}
