#ifndef DEBUG_H
#define DEBUG_H

#include<cassert>
#include<iostream>

#include "egraphin.h"

#ifdef DEBUG
    #define DEBUG_CERR(x) { std::cerr << x << std::endl; }
#else
    #define DEBUG_CERR(x) {}
#endif

#ifdef DEBUG
    #define DEBUG_ASSERT(x) { assert(x); }
#else
    #define DEBUG_ASSERT(x) {}
#endif

void debug_print_egraph(const EGraph &g);

void debug_print_extraction(const EGraph &g, const Extraction &e);

bool is_wellformed_egraph(const EGraph &g, bool allow_unextractable_child, bool allow_subregion_child);

bool is_valid_egraph_mapping(const EGraphMapping &g2gp, const EGraph &g, const EGraph &gp, bool isPartial, bool isInjective, bool isSurjective, bool checkChildrenConsistentcy);

bool arg_check_regionalized_egraph(const EGraph &g);

using Statewalk = vector<pair<EClassId, ENodeId> >;

bool is_valid_statewalk(const EGraph &g, const EClassId root, const Statewalk &sw);

bool is_valid_extraction(const EGraph &g, const EClassId root, const Extraction &e);

bool is_effect_safe_extraction(const EGraph &g, const EClassId root, const Extraction &e);

#endif