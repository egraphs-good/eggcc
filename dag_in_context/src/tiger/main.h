#ifndef MAIN_H
#define MAIN_H

#include<string>

using namespace std;

struct Config {
    string extract_region_timings_path;
    bool ilp_mode = false;
    bool ilp_minimize_objective = true;
    int ilp_timeout_seconds = 5 * 60; // 5 minutes
    bool time_ilp = false;
    double percent_regions = 100.0; // Percentage of regions to run ILP timing on (0.0 to 100.0)
};

extern Config g_config;

#endif