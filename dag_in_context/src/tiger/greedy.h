#ifndef GREEDY_H
#define GREEDY_H

#include "egraphin.h"

using Cost = unsigned long long;

const Cost INF = ~0ull;

vector<Cost> greedy_extract_estimate_all_eclasses_cost(const EGraph &g);

Cost get_enode_cost(const ENode &n);

vector<vector<Cost> > project_statewalk_cost(const EGraphMapping &gr2g, const vector<vector<Cost> > &statewalk_cost);

vector<vector<Cost> > compute_statewalk_cost(const EGraph &g);

Extraction statewalk_greedy_extraction(const EGraph &g, const EClassId root);

#endif  // GREEDY_H