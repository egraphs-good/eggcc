#ifndef MAIN_H
#define MAIN_H

#include<string>

using namespace std;

struct Config {
    string extract_region_timings_path;
    bool ilp_mode = false;
    bool ilp_minimize_objective = true;
    bool use_gurobi = true;
    int ilp_timeout_seconds = 10;
    int ilp_timeout_gurobi = 5 * 60;
    bool time_ilp = false;
};

extern Config g_config;

#endif