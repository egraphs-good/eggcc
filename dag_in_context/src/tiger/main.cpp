#include<cassert>
#include<cstdio>
#include<cstring>

#include "main.h"
#include "json2egraphin.h"
#include "regionalize.h"
#include "toegglog.h"
#include "ilp.h"
#include "time_ilp.h"

Config g_config;

/*
Extractor flags:

--report-region-timings
    a file path should follow
    When on, write the tiger timing information into the designated file
*/

int main(int argc, char *argv[]) {
    bool requested_ilp_no_minimize = false;
    bool use_gurobi_solver = true;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--report-region-timings") == 0) {
            assert(i + 1 < argc);
            g_config.extract_region_timings_path = argv[i + 1];
            ++i;
        } else if (strcmp(argv[i], "--ilp-mode") == 0) {
            g_config.ilp_mode = true;
        } else if (strcmp(argv[i], "--ilp-no-minimize") == 0) {
            requested_ilp_no_minimize = true;
        } else if (strcmp(argv[i], "--time-ilp") == 0) {
            g_config.time_ilp = true;
        } else if (strcmp(argv[i], "--ilp-solver") == 0) {
            assert(i + 1 < argc);
            const char *solver = argv[i + 1];
            if (strcmp(solver, "gurobi") == 0) {
                use_gurobi_solver = true;
            } else if (strcmp(solver, "cbc") == 0) {
                use_gurobi_solver = false;
            } else {
                std::fprintf(stderr, "Unknown ILP solver '%s'. Expected 'gurobi' or 'cbc'.\n", solver);
                return 1;
            }
            ++i;
        }
    }

    if (requested_ilp_no_minimize) {
        if (!g_config.ilp_mode) {
            std::fprintf(stderr, "--ilp-no-minimize requires --ilp-mode\n");
            return 1;
        }
        g_config.ilp_minimize_objective = false;
    }
    pair<EGraph, vector<EClassId> > res = parse_egglog_json();
    EGraph &g = res.first;
    vector<EClassId> &roots = res.second;
    if (g_config.time_ilp && g_config.extract_region_timings_path.empty()) {
        std::fprintf(stderr, "--time-ilp requires --report-region-timings\n");
        return 1;
    }

    vector<Extraction> extractions;
    if (g_config.ilp_mode) {
        extractions = extractAllILP(g, roots, use_gurobi_solver);
    } else {
        extractions = extract_all_fun_roots_tiger(g, roots);
    }

    if (g_config.time_ilp) {
        vector<ExtractRegionTiming> timings = compute_extract_region_timings(g, roots, use_gurobi_solver);
        if (!write_extract_region_timings_json(timings, g_config.extract_region_timings_path)) {
            std::fprintf(stderr, "failed to write extract-region timings to %s\n", g_config.extract_region_timings_path.c_str());
            return 1;
        }
    }

    output_egglog(g, extractions);
    return 0;
}