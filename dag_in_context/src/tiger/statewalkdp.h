#ifndef STATEWALKDP_H
#define STATEWALKDP_H

#include "egraphin.h"

using Cost = unsigned long long;

using Statewalk = vector<pair<EClassId, ENodeId> >;

using StatewalkWidthStat = vector<size_t>;

Statewalk statewalkDP(const EGraph &g, const EClassId root, const vector<vector<Cost> > &statewalk_cost, const bool use_liveness = true, StatewalkWidthStat *const stat = nullptr);

#endif
