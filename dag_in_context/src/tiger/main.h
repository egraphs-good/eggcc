#ifndef MAIN_H
#define MAIN_H

#include<string>

using namespace std;

struct Config {
    string report_file;
    bool ilp_mode = false;
    bool ilp_minimize_objective = true;

    bool skip_report();
};

bool Config::skip_report() {
    return report_file.empty();
}

extern Config g_config;

#endif