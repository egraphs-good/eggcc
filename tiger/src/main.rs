// Tiger extractor — Rust port of dag_in_context/src/tiger/*.cpp
// Direct line-by-line translation. Module names mirror the .cpp files.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

// Use mimalloc as the global allocator. The port allocates a fresh String for
// every JSON token, which on Linux's glibc malloc is significantly slower than
// the C++ side that reuses a `static char buf[505]`. mimalloc has small-object
// fast paths comparable to macOS's allocator and brings Linux performance back
// in line with the C++ binary.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod egraphin;
mod debug;
mod persistent_btree;
mod greedy;
mod json2egraphin;
mod toegglog;
mod statewalkdp;
mod tiger;
mod regionalize;
mod ilp;
mod time_ilp;
mod config;

use crate::config::{set_config, Config};
use crate::egraphin::{EClassId, EGraph, Extraction};

/*
Extractor flags:

--report-region-timings
    a file path should follow
    When on, write the tiger timing information into the designated file
*/

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let mut g_config: Config = Config::default();
    let mut requested_ilp_no_minimize: bool = false;
    let mut use_gurobi_solver: bool = true;
    let mut i: usize = 1;
    while i < argv.len() {
        if argv[i] == "--report-region-timings" {
            assert!(i + 1 < argv.len());
            g_config.extract_region_timings_path = argv[i + 1].clone();
            i += 1;
        } else if argv[i] == "--ilp-mode" {
            g_config.ilp_mode = true;
        } else if argv[i] == "--ilp-no-minimize" {
            requested_ilp_no_minimize = true;
        } else if argv[i] == "--time-ilp" {
            g_config.time_ilp = true;
        } else if argv[i] == "--percent-regions" {
            if i + 1 >= argv.len() {
                eprintln!("--percent-regions requires a value");
                std::process::exit(1);
            }
            match argv[i + 1].parse::<f64>() {
                Ok(v) => {
                    g_config.percent_regions = v;
                }
                Err(_) => {
                    eprintln!("Invalid value for --percent-regions: {}", argv[i + 1]);
                    std::process::exit(1);
                }
            }
            if !g_config.percent_regions.is_finite()
                || g_config.percent_regions < 0.0
                || g_config.percent_regions > 100.0
            {
                eprintln!(
                    "--percent-regions must be a finite number between 0.0 and 100.0, got: {}",
                    argv[i + 1]
                );
                std::process::exit(1);
            }
            i += 1;
        } else if argv[i] == "--ilp-solver" {
            assert!(i + 1 < argv.len());
            let solver: &str = &argv[i + 1];
            if solver == "gurobi" {
                use_gurobi_solver = true;
            } else if solver == "cbc" {
                use_gurobi_solver = false;
            } else {
                eprintln!(
                    "Unknown ILP solver '{}'. Expected 'gurobi' or 'cbc'.",
                    solver
                );
                std::process::exit(1);
            }
            i += 1;
        }
        i += 1;
    }

    if requested_ilp_no_minimize {
        if !g_config.ilp_mode {
            eprintln!("--ilp-no-minimize requires --ilp-mode");
            std::process::exit(1);
        }
        g_config.ilp_minimize_objective = false;
    }
    set_config(g_config);
    let g_config = crate::config::g_config();
    let res: (EGraph, Vec<EClassId>) = crate::json2egraphin::parse_egglog_json();
    let g: EGraph = res.0;
    let roots: Vec<EClassId> = res.1;
    if g_config.time_ilp && g_config.extract_region_timings_path.is_empty() {
        eprintln!("--time-ilp requires --report-region-timings");
        std::process::exit(1);
    }

    let extractions: Vec<Extraction>;
    if g_config.ilp_mode {
        extractions = crate::ilp::extractAllILP(g.clone(), roots.clone(), use_gurobi_solver);
    } else {
        extractions = crate::regionalize::extract_all_fun_roots_tiger(&g, &roots);
    }

    if g_config.time_ilp {
        let timings: Vec<crate::time_ilp::ExtractRegionTiming> =
            crate::time_ilp::compute_extract_region_timings(&g, &roots);
        if !crate::time_ilp::write_extract_region_timings_json(
            &timings,
            &g_config.extract_region_timings_path,
        ) {
            eprintln!(
                "failed to write extract-region timings to {}",
                g_config.extract_region_timings_path
            );
            std::process::exit(1);
        }
    }

    crate::toegglog::output_egglog(&g, &extractions);
}
