#ifndef DAG_IN_CONTEXT_TIGER_TIME_ILP_H
#define DAG_IN_CONTEXT_TIGER_TIME_ILP_H

#include <optional>
#include <string>
#include <vector>

#include "egraphin.h"

// ilp_duration_ns is cleared only when the configured ILP solver times out. When the
// solver reports infeasibility we still record the runtime so downstream consumers can
// measure how long the attempt took. The CBC-specific fields are populated when a
// separate CBC run is available; otherwise they remain empty/default-initialized.
struct ExtractRegionTiming {
    size_t egraph_size;
    long long tiger_duration_liveon_satelliteon_ns;
    long long tiger_duration_liveon_satelliteoff_ns;
    long long tiger_duration_liveoff_satelliteon_ns;
    long long tiger_duration_liveoff_satelliteoff_ns;
    std::optional<long long> ilp_duration_ns;
    bool ilp_timed_out;
    bool ilp_infeasible;
    std::optional<long long> cbc_ilp_duration_ns;
    bool cbc_ilp_timed_out;
    bool cbc_ilp_infeasible;
    size_t ilp_encoding_num_vars;
    size_t statewalk_width_liveon_satelliteon_max;
    double statewalk_width_liveon_satelliteon_avg;
    size_t statewalk_width_liveon_satelliteoff_max;
    double statewalk_width_liveon_satelliteoff_avg;
    size_t statewalk_width_liveoff_satelliteon_max;
    double statewalk_width_liveoff_satelliteon_avg;
    size_t statewalk_width_liveoff_satelliteoff_max;
    double statewalk_width_liveoff_satelliteoff_avg;
};

std::vector<ExtractRegionTiming> compute_extract_region_timings(
    const EGraph &g,
    const std::vector<EClassId> &fun_roots,
    bool primary_use_gurobi);

bool write_extract_region_timings_json(
    const std::vector<ExtractRegionTiming> &timings,
    const std::string &path);

#endif  // DAG_IN_CONTEXT_TIGER_TIME_ILP_H
