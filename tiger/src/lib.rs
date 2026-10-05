//! Tiger: the effect-safe extractor for eggcc.
//!
//! Tiger extracts programs from an e-graph whose terms carry an explicit state
//! edge (eggcc's `StateT`). Within a region, the effectful e-nodes form a chain
//! (the *statewalk*); the extractor chooses that chain with a dynamic program
//! ([`statewalk`]) so that the pure terms it needs are all extractable from the
//! chosen state, then fills in the pure terms greedily ([`greedy`]). Regions
//! (function bodies, loop bodies, branches) are extracted independently and
//! stitched together ([`region`]).
//!
//! The entry point is [`extract_from_egglog`], which reads the e-graph straight
//! out of egglog ([`egglog_in`]) and returns egglog terms ([`to_term`]).

mod checks;
pub mod cost;
mod egglog_in;
pub mod egraph;
mod greedy;
mod persistent;
mod region;
mod statewalk;
mod to_term;

use egglog::{TermDag, TermId};

pub use statewalk::StatewalkOptions;

/// Extract every `Function` in `egraph` with tiger.
///
/// Returns the root term of each extracted function (in e-graph order) together
/// with the `TermDag` the terms live in. Each root is a `(Function name in-ty out-ty body)`
/// term. Types and contexts on `Arg`, `Const` and `Empty` are placeholders
/// (`(TupleT (TNil))` and `(DumC)`); callers restore them afterwards.
pub fn extract_from_egglog(egraph: &egglog::EGraph) -> (Vec<TermId>, TermDag) {
    let (g, roots) = egglog_in::build_egraph(egraph);
    let extractions = region::extract_all(&g, &roots, StatewalkOptions::default());
    let mut termdag = TermDag::default();
    let terms = extractions
        .iter()
        .map(|e| to_term::extraction_to_term(&g, e, &mut termdag))
        .collect();
    (terms, termdag)
}
