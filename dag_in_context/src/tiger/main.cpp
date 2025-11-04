#include<cassert>

#include "main.h"
#include "json2egraphin.h"
#include "regionalize.h"
#include "toegglog.h"

Config g_config;

/*
Extractor flags:

--report-region-timings
    a file path should follow
    When on, write the tiger timing information into the designated file
*/

int main(int argc, char *argv[]) {
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--report-region-timings") == 0) {
            assert(i + 1 < argc);
            g_config.report_file = argv[i + 1];
        }
    }
    pair<EGraph, vector<EClassId> > res = parse_egglog_json();
    EGraph &g = res.first;
    vector<EClassId> &roots = res.second;
    vector<Extraction> extractions = extract_all_fun_roots_tiger(g, roots);
    output_egglog(g, extractions);
    return 0;
}