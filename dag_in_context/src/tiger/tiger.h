#ifndef TIGER_H
#define TIGER_H

#include<numeric>
#include<algorithm>

#include "egraphin.h"

using Cost = unsigned long long;

Extraction extract_regionalized_egraph_tiger(const EGraph &g, const EClassId root,
                                             const vector<vector<Cost> > &statewalk_cost,
                                             bool use_liveness,
                                             bool use_satellite_opt);

struct StatewalkWidthReport {
    size_t max_width;
    double avg_width;

    StatewalkWidthReport(const vector<size_t> &data) {
        max_width = *max_element(data.begin(), data.end());
        avg_width = (double)accumulate(data.begin(), data.end(), 0) / data.size();
    }
};

struct StatewalkWidthReports {
    StatewalkWidthReport liveon_satelliteon;
    StatewalkWidthReport liveon_satelliteoff;
    StatewalkWidthReport liveoff_satelliteon;
    StatewalkWidthReport liveoff_satelliteoff;

    StatewalkWidthReports(const StatewalkWidthReport &liveon_sat_on,
                          const StatewalkWidthReport &liveon_sat_off,
                          const StatewalkWidthReport &liveoff_sat_on,
                          const StatewalkWidthReport &liveoff_sat_off)
        : liveon_satelliteon(liveon_sat_on),
          liveon_satelliteoff(liveon_sat_off),
          liveoff_satelliteon(liveoff_sat_on),
          liveoff_satelliteoff(liveoff_sat_off) {}
};

StatewalkWidthReports get_stat_regionalized_egraph_tiger(const EGraph &g, const EClassId root, const vector<vector<Cost> > &statewalk_cost);

#endif