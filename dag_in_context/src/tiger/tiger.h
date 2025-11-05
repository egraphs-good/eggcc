#ifndef TIGER_H
#define TIGER_H

#include<numeric>
#include<algorithm>

#include "egraphin.h"

using Cost = unsigned long long;

Extraction extract_regionalized_egraph_tiger(const EGraph &g, const EClassId root, const vector<vector<Cost> > &statewalk_cost);

// add more stats here if need
struct StatewalkWidthReport {
    size_t max_width;
    double avg_width;

    StatewalkWidthReport(const vector<size_t> &data) {
        max_width = *max_element(data.begin(), data.end());
        avg_width = (double)accumulate(data.begin(), data.end(), 0) / data.size();
    }
};

// the first one is with liveness on, the second one is with liveness off
pair<StatewalkWidthReport, StatewalkWidthReport> get_stat_regionalized_egraph_tiger(const EGraph &g, const EClassId root, const vector<vector<Cost> > &statewalk_cost);

#endif