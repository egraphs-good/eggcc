#ifndef DAG_IN_CONTEXT_TIGER_TIME_ILP_H
#define DAG_IN_CONTEXT_TIGER_TIME_ILP_H

#include <optional>
#include <string>
#include <vector>

#include "egraphin.h"

// ilp_duration_ns is invalid when ilp_timed_out or ilp_infeasible is true
struct ExtractRegionTiming {
    size_t egraph_size;
    long long tiger_duration_liveon_satelliteon_ns;
    long long tiger_duration_liveon_satelliteoff_ns;
    long long tiger_duration_liveoff_satelliteon_ns;
    long long tiger_duration_liveoff_satelliteoff_ns;
    std::optional<long long> ilp_duration_ns;
    bool ilp_timed_out;
    bool ilp_infeasible;
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
    const std::vector<EClassId> &fun_roots);

bool write_extract_region_timings_json(
    const std::vector<ExtractRegionTiming> &timings,
    const std::string &path);

#endif  // DAG_IN_CONTEXT_TIGER_TIME_ILP_H
