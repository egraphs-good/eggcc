#ifndef REGIONALIZE_H
#define REGIONALIZE_H

#include "egraphin.h"
#include "greedy.h"

using RegionId = int;

pair<EGraph, pair<EClassId, EGraphMapping> > construct_regionalized_egraph(const EGraph &g, const EClassId root);

vector<EClassId> find_all_region_roots(const EGraph &g, const vector<EClassId> &fun_roots);

vector<Extraction> extract_all_fun_roots_tiger(const EGraph &g, const vector<EClassId> &fun_roots);

ExtractionENodeId extract_region_tiger(
	const EGraph &g,
	const EClassId root,
	Extraction &e,
	const vector<RegionId> &region_root_id,
	vector<pair<Extraction, ExtractionENodeId>> &region_extraction_cache,
	const vector<vector<Cost>> &statewalk_cost);

#endif