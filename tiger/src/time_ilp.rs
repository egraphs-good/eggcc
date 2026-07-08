// Port of time_ilp.h / time_ilp.cpp — timing harness for the ILP variant.
// Direct line-by-line translation.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

use rand_mt::Mt;
use serde::Serialize;
use serde_json::{json, Value};

use crate::config::g_config;
use crate::egraphin::{EClassId, EGraph, EGraphMapping, Extraction};
use crate::greedy::{compute_statewalk_cost, project_statewalk_cost, Cost};
use crate::ilp::extract_region_ilp_with_timing;
use crate::regionalize::{construct_regionalized_egraph, find_all_region_roots};
use crate::tiger::{
    extract_regionalized_egraph_tiger, get_stat_regionalized_egraph_tiger, StatewalkWidthReports,
};

// using Clock = chrono::steady_clock;

#[derive(Clone, Default, Serialize)]
pub struct ExtractRegionTiming {
    pub egraph_size: usize,
    pub tiger_duration_liveon_satelliteon_ns: i64,
    pub tiger_duration_liveon_satelliteoff_ns: i64,
    pub tiger_duration_liveoff_satelliteon_ns: i64,
    pub tiger_duration_liveoff_satelliteoff_ns: i64,
    // False when the Gurobi run was skipped (e.g. gurobi_cl is not installed), in which
    // case the ilp_* fields below carry no meaning and only cbc_ilp_* has real data.
    pub ilp_ran: bool,
    pub ilp_duration_ns: Option<i64>,
    pub ilp_timed_out: bool,
    pub ilp_infeasible: bool,
    pub cbc_ilp_duration_ns: Option<i64>,
    pub cbc_ilp_timed_out: bool,
    pub cbc_ilp_infeasible: bool,
    pub ilp_encoding_num_vars: usize,
    pub statewalk_width_liveon_satelliteon_max: usize,
    pub statewalk_width_liveon_satelliteon_avg: f64,
    pub statewalk_width_liveon_satelliteoff_max: usize,
    pub statewalk_width_liveon_satelliteoff_avg: f64,
    pub statewalk_width_liveoff_satelliteon_max: usize,
    pub statewalk_width_liveoff_satelliteon_avg: f64,
    pub statewalk_width_liveoff_satelliteoff_max: usize,
    pub statewalk_width_liveoff_satelliteoff_avg: f64,
}

fn measure_tiger_duration(
    gr: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
    use_liveness: bool,
    use_satellite_opt: bool,
) -> i64 {
    let tiger_start = Instant::now();
    extract_regionalized_egraph_tiger(gr, root, rstatewalk_cost, use_liveness, use_satellite_opt);
    let tiger_end = Instant::now();
    tiger_end.duration_since(tiger_start).as_nanos() as i64
}

fn compute_tiger_metrics(
    sample: &mut ExtractRegionTiming,
    gr: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
) {
    sample.tiger_duration_liveon_satelliteon_ns =
        measure_tiger_duration(gr, root, rstatewalk_cost, true, true);
    sample.tiger_duration_liveon_satelliteoff_ns =
        measure_tiger_duration(gr, root, rstatewalk_cost, true, false);
    sample.tiger_duration_liveoff_satelliteon_ns =
        measure_tiger_duration(gr, root, rstatewalk_cost, false, true);
    sample.tiger_duration_liveoff_satelliteoff_ns =
        measure_tiger_duration(gr, root, rstatewalk_cost, false, false);

    let res: StatewalkWidthReports =
        get_stat_regionalized_egraph_tiger(gr, root, rstatewalk_cost);
    sample.statewalk_width_liveon_satelliteon_max = res.liveon_satelliteon.max_width;
    sample.statewalk_width_liveon_satelliteon_avg = res.liveon_satelliteon.avg_width;
    sample.statewalk_width_liveon_satelliteoff_max = res.liveon_satelliteoff.max_width;
    sample.statewalk_width_liveon_satelliteoff_avg = res.liveon_satelliteoff.avg_width;
    sample.statewalk_width_liveoff_satelliteon_max = res.liveoff_satelliteon.max_width;
    sample.statewalk_width_liveoff_satelliteon_avg = res.liveoff_satelliteon.avg_width;
    sample.statewalk_width_liveoff_satelliteoff_max = res.liveoff_satelliteoff.max_width;
    sample.statewalk_width_liveoff_satelliteoff_avg = res.liveoff_satelliteoff.avg_width;
}

// namespace { struct SolverMetrics { ... }; SolverMetrics run_solver_for_metrics(...); }
struct SolverMetrics {
    duration_ns: Option<i64>,
    timed_out: bool,
    infeasible: bool,
    encoding_vars: usize,
}

fn run_solver_for_metrics(
    gr: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
    use_gurobi: bool,
) -> SolverMetrics {
    let mut extraction: Extraction = Extraction::new();
    let mut timed_out: bool = false;
    let mut infeasible: bool = false;
    let mut encoding_vars: usize = 0;
    let ns: i64 = extract_region_ilp_with_timing(
        gr,
        root,
        rstatewalk_cost,
        &mut extraction,
        &mut timed_out,
        &mut infeasible,
        &mut encoding_vars,
        use_gurobi,
    );
    let duration: Option<i64> = if !timed_out { Some(ns) } else { None };
    SolverMetrics {
        duration_ns: duration,
        timed_out,
        infeasible,
        encoding_vars,
    }
}

fn compute_ilp_metrics(
    sample: &mut ExtractRegionTiming,
    gr: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
) {
    sample.ilp_ran = g_config().time_ilp_run_gurobi;
    if sample.ilp_ran {
        let gurobi_metrics: SolverMetrics = run_solver_for_metrics(gr, root, rstatewalk_cost, true);
        sample.ilp_timed_out = gurobi_metrics.timed_out;
        sample.ilp_infeasible = gurobi_metrics.infeasible;
        sample.ilp_encoding_num_vars = gurobi_metrics.encoding_vars;
        sample.ilp_duration_ns = gurobi_metrics.duration_ns;
    } else {
        // Gurobi was skipped (e.g. gurobi_cl not installed): leave the ilp_* fields
        // empty. The encoding size is solver-independent, so take it from CBC below.
        sample.ilp_timed_out = false;
        sample.ilp_infeasible = false;
        sample.ilp_duration_ns = None;
    }

    let cbc_metrics: SolverMetrics = run_solver_for_metrics(gr, root, rstatewalk_cost, false);
    sample.cbc_ilp_duration_ns = cbc_metrics.duration_ns;
    sample.cbc_ilp_timed_out = cbc_metrics.timed_out;
    sample.cbc_ilp_infeasible = cbc_metrics.infeasible;
    if !sample.ilp_ran {
        sample.ilp_encoding_num_vars = cbc_metrics.encoding_vars;
    }
}

struct PreparedRegion {
    egraph: EGraph,
    root: EClassId,
    timings_index: usize, // index into the timings vector
    statewalk_cost: Vec<Vec<Cost>>,
}

pub fn compute_extract_region_timings(
    g: &EGraph,
    fun_roots: &Vec<EClassId>,
) -> Vec<ExtractRegionTiming> {
    let region_roots: Vec<EClassId> = find_all_region_roots(g, fun_roots);

    if region_roots.is_empty() {
        return Vec::new();
    }

    // Select a random subset of regions based on percent_regions BEFORE preparing them
    // Calculate the number of regions to run on (rounded up)
    let mut num_regions_to_run: usize =
        (region_roots.len() as f64 * g_config().percent_regions / 100.0).ceil() as usize;
    if num_regions_to_run == 0 {
        num_regions_to_run = 1; // Always run at least one region if there are any
    }

    // Create indices and shuffle them to get random selection
    let mut region_indices: Vec<usize> = vec![0usize; region_roots.len()];
    for i in 0..region_roots.len() {
        region_indices[i] = i;
    }

    // Only shuffle and subset if we're not running all regions
    if num_regions_to_run < region_roots.len() {
        // C++: std::random_device rd; std::mt19937 rng(rd()); std::shuffle(...);
        // Mirror with rand_mt::Mt (32-bit Mersenne Twister) seeded from system entropy.
        let seed: u32 = {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0);
            nanos
        };
        let mut rng: Mt = Mt::new(seed);
        // Fisher-Yates shuffle, mirroring std::shuffle.
        let n: usize = region_indices.len();
        let mut i: usize = n;
        while i > 1 {
            i -= 1;
            // Generate a value in [0, i].
            let r: u32 = rng.next_u32();
            let j: usize = (r as usize) % (i + 1);
            region_indices.swap(i, j);
        }
        region_indices.resize(num_regions_to_run, 0);
    }

    let statewalk_cost: Vec<Vec<Cost>> = compute_statewalk_cost(g);

    // timings only contains entries for the regions we are actually measuring
    let mut timings: Vec<ExtractRegionTiming> =
        vec![ExtractRegionTiming::default(); region_indices.len()];

    let mut prepared_regions: Vec<PreparedRegion> = Vec::new();
    prepared_regions.reserve(region_indices.len());

    for timings_idx in 0..region_indices.len() {
        let region_idx: usize = region_indices[timings_idx];
        let regionalized: (EGraph, (EClassId, EGraphMapping)) =
            construct_regionalized_egraph(g, region_roots[region_idx]);
        let mut prepared = PreparedRegion {
            egraph: EGraph::default(),
            root: 0,
            timings_index: 0,
            statewalk_cost: Vec::new(),
        };
        prepared.egraph = regionalized.0;
        prepared.root = regionalized.1 .0;
        prepared.timings_index = timings_idx;

        let gr2g: &EGraphMapping = &regionalized.1 .1;
        prepared.statewalk_cost = project_statewalk_cost(gr2g, &statewalk_cost);

        let mut sample: ExtractRegionTiming = ExtractRegionTiming::default();

        // compute egraph size: number of nodes in the regionalized egraph
        let mut egraph_size: usize = 0;
        for eclass in prepared.egraph.eclasses.iter() {
            egraph_size += eclass.enodes.len();
        }

        sample.egraph_size = egraph_size;
        compute_tiger_metrics(
            &mut sample,
            &prepared.egraph,
            prepared.root,
            &prepared.statewalk_cost,
        );
        timings[timings_idx] = sample;

        prepared_regions.push(prepared);
    }

    let hardware_threads: u32 = thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(0);
    let mut usable_threads: u32 = if hardware_threads == 0 { 1 } else { hardware_threads };

    // divide by 11, we spin up 10 benchmarks at once in profile.py
    if usable_threads >= 20 {
        usable_threads /= 11;
    }

    let mut worker_count: usize =
        std::cmp::min(usable_threads as usize, prepared_regions.len());
    if worker_count == 0 {
        worker_count = 1;
    }

    let next_index: AtomicUsize = AtomicUsize::new(0);

    eprint!(
        "Running ILP timing on {}/{} regions, one dot per region:",
        region_indices.len(),
        region_roots.len()
    );
    eprintln!();

    // Worker closure mirrored as a scoped-thread body.
    // C++:
    //   auto worker = [&]() { while (true) { ... } };
    let timings_mutex: Mutex<&mut Vec<ExtractRegionTiming>> = Mutex::new(&mut timings);
    let prepared_ref: &Vec<PreparedRegion> = &prepared_regions;
    let next_index_ref: &AtomicUsize = &next_index;

    thread::scope(|s| {
        let mut threads = Vec::with_capacity(worker_count);
        for _i in 0..worker_count {
            let timings_mutex_ref = &timings_mutex;
            let handle = s.spawn(move || loop {
                let work_idx: usize = next_index_ref.fetch_add(1, Ordering::Relaxed);
                if work_idx >= prepared_ref.len() {
                    break;
                }
                eprint!(".");
                use std::io::Write;
                let _ = std::io::stderr().flush();
                let prepared: &PreparedRegion = &prepared_ref[work_idx];
                // Compute into a local sample to avoid holding the mutex during the
                // expensive ILP run.
                let mut sample_local: ExtractRegionTiming = {
                    let timings_guard = timings_mutex_ref.lock().unwrap();
                    timings_guard[prepared.timings_index].clone()
                };
                compute_ilp_metrics(
                    &mut sample_local,
                    &prepared.egraph,
                    prepared.root,
                    &prepared.statewalk_cost,
                );
                let mut timings_guard = timings_mutex_ref.lock().unwrap();
                timings_guard[prepared.timings_index] = sample_local;
            });
            threads.push(handle);
        }
        for h in threads {
            let _ = h.join();
        }
    });

    timings
}

pub fn write_extract_region_timings_json(
    timings: &Vec<ExtractRegionTiming>,
    path: &str,
) -> bool {
    // The C++ writes a hand-formatted JSON file with the schema:
    //   {"rows": [ {<row 1>}, {<row 2>}, ... ]}
    // Each row contains every public field of ExtractRegionTiming, with
    // std::optional<long long> serialised as either the integer or `null`.
    // We construct the same structure with serde_json::Value so the output
    // matches the consumer's deserialiser exactly.
    let mut rows: Vec<Value> = Vec::new();
    for sample in timings.iter() {
        let row = json!({
            "egraph_size": sample.egraph_size,
            "tiger_duration_liveon_satelliteon_ns": sample.tiger_duration_liveon_satelliteon_ns,
            "tiger_duration_liveon_satelliteoff_ns": sample.tiger_duration_liveon_satelliteoff_ns,
            "tiger_duration_liveoff_satelliteon_ns": sample.tiger_duration_liveoff_satelliteon_ns,
            "tiger_duration_liveoff_satelliteoff_ns": sample.tiger_duration_liveoff_satelliteoff_ns,
            "ilp_ran": sample.ilp_ran,
            "ilp_duration_ns": sample.ilp_duration_ns,
            "ilp_timed_out": sample.ilp_timed_out,
            "ilp_infeasible": sample.ilp_infeasible,
            "cbc_ilp_duration_ns": sample.cbc_ilp_duration_ns,
            "cbc_ilp_timed_out": sample.cbc_ilp_timed_out,
            "cbc_ilp_infeasible": sample.cbc_ilp_infeasible,
            "ilp_encoding_num_vars": sample.ilp_encoding_num_vars,
            "statewalk_width_liveon_satelliteon_max": sample.statewalk_width_liveon_satelliteon_max,
            "statewalk_width_liveon_satelliteon_avg": sample.statewalk_width_liveon_satelliteon_avg,
            "statewalk_width_liveon_satelliteoff_max": sample.statewalk_width_liveon_satelliteoff_max,
            "statewalk_width_liveon_satelliteoff_avg": sample.statewalk_width_liveon_satelliteoff_avg,
            "statewalk_width_liveoff_satelliteon_max": sample.statewalk_width_liveoff_satelliteon_max,
            "statewalk_width_liveoff_satelliteon_avg": sample.statewalk_width_liveoff_satelliteon_avg,
            "statewalk_width_liveoff_satelliteoff_max": sample.statewalk_width_liveoff_satelliteoff_max,
            "statewalk_width_liveoff_satelliteoff_avg": sample.statewalk_width_liveoff_satelliteoff_avg,
        });
        rows.push(row);
    }
    let top = json!({ "rows": rows });
    let serialised = match serde_json::to_string_pretty(&top) {
        Ok(s) => s,
        Err(_) => return false,
    };
    // Mirror the trailing newline that the C++ ofstream writes.
    let mut out = serialised;
    out.push('\n');
    match std::fs::write(path, out) {
        Ok(_) => true,
        Err(_) => false,
    }
}
