#ifndef DAG_IN_CONTEXT_TIGER_ILP_H
#define DAG_IN_CONTEXT_TIGER_ILP_H

#include <iosfwd>
#include <string>
#include <utility>
#include <vector>

#include "egraphin.h"

Extraction extractRegionILP(const EGraph &g, EClassId root);

long long extract_region_ilp_with_timing(const EGraph &g, EClassId root, Extraction &out, bool &timed_out);

vector<Extraction> extractAllILP(EGraph g, vector<EClassId> fun_roots);

#endif  // DAG_IN_CONTEXT_TIGER_ILP_H
