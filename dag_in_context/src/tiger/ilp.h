#ifndef DAG_IN_CONTEXT_TIGER_ILP_H
#define DAG_IN_CONTEXT_TIGER_ILP_H

#include <iosfwd>
#include <string>
#include <utility>
#include <vector>

#include "egraphin.h"
#include "greedy.h"

Extraction extractRegionILP(const EGraph &g, EClassId root, const vector<vector<Cost> > &rstatewalk_cost, bool use_gurobi);

long long extract_region_ilp_with_timing(const EGraph &g, EClassId root, const vector<vector<Cost> > &rstatewalk_cost, Extraction &out, bool &timed_out, bool &infeasible, size_t &ilp_encoding_num_vars, bool use_gurobi);

vector<Extraction> extractAllILP(EGraph g, vector<EClassId> fun_roots, bool use_gurobi);

#endif  // DAG_IN_CONTEXT_TIGER_ILP_H
