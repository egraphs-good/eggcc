//! Tiger: the effect-safe greedy extractor for eggcc, ported from
//! `dag_in_context/src/tiger/*.cpp`.
//!
//! The extractor reads the egglog e-graph directly (see [`egglog_in`]) and
//! returns one egglog [`Term`](egglog::Term) per extracted function.

#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::needless_range_loop)]

mod debug;
mod egglog_in;
pub mod egraphin;
mod greedy;
mod persistent_btree;
mod regionalize;
mod statewalkdp;
mod tiger;
mod to_term;

use egglog::{TermDag, TermId};

/// Extract every `Function` in `egraph` with the tiger greedy extractor.
///
/// Returns the root term of each extracted function (in e-graph order) together
/// with the `TermDag` the terms live in. Each root is a `(Function name in-ty out-ty body)`
/// term. Types and contexts on `Arg`, `Const` and `Empty` are placeholders
/// (`(TupleT (TNil))` and `(DumC)`); callers restore them afterwards.
pub fn extract_from_egglog(egraph: &egglog::EGraph) -> (Vec<TermId>, TermDag) {
    let (g, roots) = egglog_in::build_egraph(egraph);
    let extractions = regionalize::extract_all_fun_roots_tiger(&g, &roots);
    let mut termdag = TermDag::default();
    let terms = extractions
        .iter()
        .map(|e| to_term::extraction_to_term(&g, e, &mut termdag))
        .collect();
    (terms, termdag)
}
