#include "time_ilp.h"

#include <chrono>
#include <fstream>
#include <unordered_set>

#include "greedy.h"
#include "ilp.h"
#include "regionalize.h"
#include "tiger.h"

using namespace std;

namespace {
using Clock = chrono::steady_clock;
}

vector<ExtractRegionTiming> compute_extract_region_timings(
    const EGraph &g,
    const vector<EClassId> &fun_roots) {
    vector<ExtractRegionTiming> timings;

    vector<EClassId> region_roots = find_all_region_roots(g, fun_roots);
    

    timings.reserve(region_roots.size());
    for (size_t idx = 0; idx < region_roots.size(); ++idx) {
        auto tiger_start = Clock::now();

        Extraction tiger_extraction;

        vector<vector<Cost>> statewalk_cost = compute_statewalk_cost(g);
        vector<pair<Extraction, ExtractionENodeId>> region_extraction_cache(region_roots.size());
        for (size_t j = 0; j < region_extraction_cache.size(); ++j) {
            region_extraction_cache[j].second = -1;
        }
        pair<EGraph, pair<EClassId, EGraphMapping>> regionalized = construct_regionalized_egraph(g, region_roots[idx]);

        Extraction tmpe = extract_regionalized_egraph_tiger(regionalized.first, region_roots[idx], project_statewalk_cost(regionalized.second.second, statewalk_cost));

        auto tiger_end = Clock::now();
        long long tiger_ns = chrono::duration_cast<chrono::nanoseconds>(tiger_end - tiger_start).count();

        Extraction ilp_extraction;
        bool ilp_timed_out = false;
        long long ilp_ns = extract_region_ilp_with_timing(
            g,
            region_roots[idx],
            ilp_extraction,
            ilp_timed_out);


        ExtractRegionTiming sample;
        sample.egraph_size = g.eclasses.size();
        sample.tiger_duration_ns = tiger_ns;
        if (ilp_timed_out) {
            sample.ilp_duration_ns = nullopt;
        } else {
            sample.ilp_duration_ns = ilp_ns;
        }
        sample.ilp_timed_out = ilp_timed_out;
        // TODO implement statewalk width computation
        sample.statewalk_width = 0;
        timings.push_back(sample);
    }

    return timings;
}

bool write_extract_region_timings_json(
    const vector<ExtractRegionTiming> &timings,
    const string &path) {
    ofstream out(path.c_str());
    if (!out.good()) {
        return false;
    }

    out << "{\n  \"rows\": [";
    if (!timings.empty()) {
        for (size_t i = 0; i < timings.size(); ++i) {
            const auto &sample = timings[i];
            out << (i == 0 ? "\n" : ",\n");
            out << "    {\"egraph_size\": " << sample.egraph_size
                << ", \"tiger_duration_ns\": " << sample.tiger_duration_ns
                << ", \"ilp_duration_ns\": ";
            if (sample.ilp_duration_ns.has_value()) {
                out << sample.ilp_duration_ns.value();
            } else {
                out << "null";
            }
            out << ", \"ilp_timed_out\": " << (sample.ilp_timed_out ? "true" : "false")
                << ", \"statewalk_width\": " << static_cast<unsigned long long>(sample.statewalk_width)
                << "}";
        }
        out << "\n  ";
    }
    out << "]\n}\n";
    return true;
}
