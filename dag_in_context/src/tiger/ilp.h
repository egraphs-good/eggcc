#ifndef DAG_IN_CONTEXT_TIGER_ILP_H
#define DAG_IN_CONTEXT_TIGER_ILP_H

#include <iosfwd>
#include <string>
#include <utility>
#include <vector>

using EClassId = int;
using ENodeId = int;
using ExtractionENodeId = int;

struct ENode {
    std::string head;
    EClassId eclass;
    std::vector<EClassId> ch;
};

struct EClass {
    std::vector<ENode> enodes;
    bool isEffectful;
};

struct EGraph {
    std::vector<EClass> eclasses;
};

struct ExtractionENode {
    EClassId c;
    ENodeId n;
    std::vector<ExtractionENodeId> ch;
};

using Extraction = std::vector<ExtractionENode>;
using StateWalk = std::vector<std::pair<EClassId, ENodeId>>;

bool validExtraction(const EGraph &g, EClassId root, const Extraction &e);
std::pair<bool, Extraction> regionExtractionWithStateWalk(const EGraph &g, EClassId root, const StateWalk &sw);
StateWalk UnguidedFindStateWalk(const EGraph &g, EClassId initc, ENodeId initn, EClassId root, const std::vector<std::vector<int>> &nsubregion);
Extraction extractRegionILP(const EGraph &g, EClassId initc, ENodeId initn, EClassId root, const std::vector<std::vector<int>> &nsubregion);

extern bool g_use_gurobi;
extern int g_ilp_timeout_seconds;
extern bool g_ilp_minimize_objective;

#endif  // DAG_IN_CONTEXT_TIGER_ILP_H
