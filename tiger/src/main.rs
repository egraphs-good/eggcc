// Tiger extractor — Rust port of dag_in_context/src/tiger/*.cpp (greedy extractor only).
//
// Reads a serialized egglog e-graph (JSON) on stdin and writes an egglog program
// that reconstructs the extracted functions on stdout.

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

// Use mimalloc as the global allocator: the JSON tokenizer allocates a String per
// token, which is noticeably slower on glibc malloc than on mimalloc.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod debug;
mod egraphin;
mod greedy;
mod json2egraphin;
mod persistent_btree;
mod regionalize;
mod statewalkdp;
mod tiger;
mod toegglog;

fn main() {
    let (g, roots) = json2egraphin::parse_egglog_json();
    let extractions = regionalize::extract_all_fun_roots_tiger(&g, &roots);
    toegglog::output_egglog(&g, &extractions);
}
