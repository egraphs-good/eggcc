#ifndef DAG_IN_CONTEXT_TIGER_ILP_H
#define DAG_IN_CONTEXT_TIGER_ILP_H

#include <iosfwd>
#include <string>
#include <utility>
#include <vector>

#include "egraphin.h"
#include "greedy.h"

Extraction extractRegionILP(const EGraph &g, EClassId root, const vector<vector<Cost> > &rstatewalk_cost);

long long extract_region_ilp_with_timing(const EGraph &g, EClassId root, const vector<vector<Cost> > &rstatewalk_cost, Extraction &out, bool &timed_out, bool &infeasible, size_t &edge_variable_count);

vector<Extraction> extractAllILP(EGraph g, vector<EClassId> fun_roots);

#endif  // DAG_IN_CONTEXT_TIGER_ILP_H
