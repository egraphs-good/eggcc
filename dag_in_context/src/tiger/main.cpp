#include<cassert>
#include<cmath>
#include<cstdio>
#include<cstring>
#include<stdexcept>

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
        } else if (strcmp(argv[i], "--percent-regions") == 0) {
            if (i + 1 >= argc) {
                std::fprintf(stderr, "--percent-regions requires a value\n");
                return 1;
            }
            try {
                g_config.percent_regions = std::stod(argv[i + 1]);
            } catch (const std::exception &e) {
                std::fprintf(stderr, "Invalid value for --percent-regions: %s\n", argv[i + 1]);
                return 1;
            }
            if (!std::isfinite(g_config.percent_regions) || g_config.percent_regions < 0.0 || g_config.percent_regions > 100.0) {
                std::fprintf(stderr, "--percent-regions must be a finite number between 0.0 and 100.0, got: %s\n", argv[i + 1]);
                return 1;
            }
            ++i;
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

    // When --time-ilp is combined with --ilp-solver cbc, skip the Gurobi run so timing
    // works on machines without gurobi_cl. CBC is always timed.
    g_config.time_ilp_run_gurobi = use_gurobi_solver;
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
        vector<ExtractRegionTiming> timings = compute_extract_region_timings(g, roots);
        if (!write_extract_region_timings_json(timings, g_config.extract_region_timings_path)) {
            std::fprintf(stderr, "failed to write extract-region timings to %s\n", g_config.extract_region_timings_path.c_str());
            return 1;
        }
    }

    output_egglog(g, extractions);
    return 0;
}