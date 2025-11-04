#ifndef EGRAPHIN_H
#define EGRAPHIN_H

// Definition of a minimal egraph and related data formats

#include <string>
#include <vector>

using EClassId = int;
using ENodeId = int;
using ExtractionENodeId = int;

using namespace std;

// -1 used to denote an unextractable eclass
const EClassId UNEXTRACTABLE_ECLASS = -1;

struct ENode {
    string head;
    EClassId eclass;
    vector<EClassId> ch;

    string get_name() const {
        int pos = head.find("###");
        return head.substr(0, pos);
    }

    string get_op() const {
        int pos = head.find("###");
        return head.substr(pos + 3, head.length() - pos - 3);
    }
};

struct EClass {
    vector<ENode> enodes;
    bool isEffectful;

    size_t nenodes() const {
        return enodes.size();
    }
};

struct EGraph {
    vector<EClass> eclasses;

    size_t neclasses() const {
        return eclasses.size();
    }
};

EGraph read_egraph(FILE* ppin);

void print_egraph(const EGraph &g);

// An extraction corespondes to a particular egraph

struct ExtractionENode {
    EClassId c;
    ENodeId n;
    vector<ExtractionENodeId> ch;
};

using Extraction = vector<ExtractionENode>;

void print_extraction(const EGraph &g, const Extraction &e);

// A mapping from egraph A to B

struct EGraphMapping {
    vector<EClassId> eclassidmp;
    vector<vector<ENodeId>> enodeidmp;

    EGraphMapping() {}

    EGraphMapping(const EGraph &g) {
        eclassidmp.resize(g.neclasses(), UNEXTRACTABLE_ECLASS);
        enodeidmp.resize(g.neclasses());
        for (EClassId i = 0; i < g.neclasses(); ++i) {
            const EClass &c = g.eclasses[i];
            enodeidmp[i].resize(c.nenodes(), UNEXTRACTABLE_ECLASS);
        }
    }

};

pair<EGraph, EGraphMapping> prune_unextractable_enodes(const EGraph &g, const EClassId root = -1);

EGraphMapping inverse_egraph_mapping(const EGraph &gp, const EGraphMapping &g2gp);

Extraction project_extraction(const EGraphMapping &f, const Extraction &e);

// A reverse index for speed up things

using EClassParents = vector<vector<pair<EClassId, ENodeId> > >;

EClassParents compute_reverse_index(const EGraph &g);

// An accompanying counter for optimizations

using ENodeCounters = vector<vector<int>>;

ENodeCounters initialize_enode_counters(const EGraph &g);

#endif  // EGRAPHIN_H
