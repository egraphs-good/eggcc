#ifndef DAG_IN_CONTEXT_TIGER_TIME_ILP_H
#define DAG_IN_CONTEXT_TIGER_TIME_ILP_H

#include <optional>
#include <string>
#include <vector>

#include "egraphin.h"

struct ExtractRegionTiming {
    size_t egraph_size;
    long long tiger_duration_ns;
    std::optional<long long> ilp_duration_ns;
    bool ilp_timed_out;
    size_t statewalk_width_liveon_max;
    double statewalk_width_liveon_avg;
    size_t statewalk_width_liveoff_max;
    double statewalk_width_liveoff_avg;
};

std::vector<ExtractRegionTiming> compute_extract_region_timings(
    const EGraph &g,
    const std::vector<EClassId> &fun_roots);

bool write_extract_region_timings_json(
    const std::vector<ExtractRegionTiming> &timings,
    const std::string &path);

#endif  // DAG_IN_CONTEXT_TIGER_TIME_ILP_H
