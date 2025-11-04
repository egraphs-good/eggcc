#ifndef TIGER_H
#define TIGER_H

#include "egraphin.h"

using Cost = unsigned long long;

Extraction extract_regionalized_egraph_tiger(const EGraph &g, const EClassId root, const vector<vector<Cost> > &statewalk_cost);

#endif