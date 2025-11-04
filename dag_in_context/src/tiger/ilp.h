#ifndef DAG_IN_CONTEXT_TIGER_ILP_H
#define DAG_IN_CONTEXT_TIGER_ILP_H

#include <iosfwd>
#include <string>
#include <utility>
#include <vector>

#include "egraphin.h"

Extraction extractRegionILP(const EGraph &g, EClassId root);

vector<Extraction> extractAllILP(EGraph g, vector<EClassId> fun_roots);

extern bool g_use_gurobi;
extern int g_ilp_timeout_seconds;
extern bool g_ilp_minimize_objective;

#endif  // DAG_IN_CONTEXT_TIGER_ILP_H
