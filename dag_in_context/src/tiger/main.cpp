#include<cassert>
#include<cstdio>
#include<cstring>

#include "main.h"
#include "json2egraphin.h"
#include "regionalize.h"
#include "toegglog.h"
#include "ilp.h"

Config g_config;

/*
Extractor flags:

--report-region-timings
    a file path should follow
    When on, write the tiger timing information into the designated file
*/

int main(int argc, char *argv[]) {
    bool requested_ilp_no_minimize = false;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--report-region-timings") == 0) {
            assert(i + 1 < argc);
            g_config.report_file = argv[i + 1];
            ++i;
        } else if (strcmp(argv[i], "--ilp-mode") == 0) {
            g_config.ilp_mode = true;
        } else if (strcmp(argv[i], "--ilp-no-minimize") == 0) {
            requested_ilp_no_minimize = true;
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
    vector<Extraction> extractions;
    if (g_config.ilp_mode) {
        g_ilp_minimize_objective = g_config.ilp_minimize_objective;
        extractions = extractAllILP(g, roots);
    } else {
        extractions = extract_all_fun_roots_tiger(g, roots);
    }
    output_egglog(g, extractions);
    return 0;
}