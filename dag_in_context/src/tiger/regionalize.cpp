#include "egraphin.h"
#include "regionalize.h"
#include "greedy.h"
#include "tiger.h"

#include "debug.h"

using namespace std;

using RegionId = int;

static int timestamp = 0;
vector<int> region_vis;
vector<EClassId> region_eclass_inv;

pair<EGraph, pair<EClassId, EGraphMapping> > construct_regionalized_egraph(const EGraph &g, const EClassId root) {
    region_vis.resize(g.neclasses(), 0);
    region_eclass_inv.resize(g.neclasses());
    ++timestamp;
    vector<EClassId> gr2g;
    region_vis[root] = timestamp;
    region_eclass_inv[root] = gr2g.size();
    gr2g.push_back(root);
    for (size_t _ = 0; _ < gr2g.size(); ++_) {
        EClassId i = gr2g[_];
        const EClass &c = g.eclasses[i];
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            bool isSubregionChild = false;
            for (size_t k = 0; k < n.ch.size(); ++k) {
                EClassId v = n.ch[k];
                if (g.eclasses[v].isEffectful && !isSubregionChild) {
                    if (region_vis[v] != timestamp) {
                        region_vis[v] = timestamp;
                        region_eclass_inv[v] = gr2g.size();
                        gr2g.push_back(v);
                    }
                    isSubregionChild = true;
                }
            }
        }
    }
    for (size_t _ = 0; _ < gr2g.size(); ++_) {
        EClassId i = gr2g[_];
        const EClass &c = g.eclasses[i];
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            for (size_t k = 0; k < n.ch.size(); ++k) {
                EClassId v = n.ch[k];
                if (!g.eclasses[v].isEffectful) {
                    if (region_vis[v] != timestamp) {
                        region_vis[v] = timestamp;
                        region_eclass_inv[v] = gr2g.size();
                        gr2g.push_back(v);
                    }
                }
            }
        }
    }
	EGraph gr;
    for (EClassId i = 0; i < (EClassId)gr2g.size(); ++i) {
        EClassId u = gr2g[i];
        const EClass &c = g.eclasses[u];
        EClass nc;
        nc.isEffectful = c.isEffectful;
        nc.enodes.resize(c.nenodes());
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            ENode &nn = nc.enodes[j];
            nn.head = n.head;
            nn.eclass = i;
            bool isSubregionchild = false;
            for (size_t k = 0; k < n.ch.size(); ++k) {
                EClassId v = n.ch[k];
                if (g.eclasses[v].isEffectful) {
                    if (isSubregionchild) {
                        continue;
                    }
                    isSubregionchild = true;
                }
                if (region_vis[v] != timestamp) {
                    nn.ch.push_back(UNEXTRACTABLE_ECLASS);
                } else {
                    nn.ch.push_back(region_eclass_inv[v]);
                }
            }
        }
        gr.eclasses.push_back(nc);
    }
    DEBUG_ASSERT(is_wellformed_egraph(gr, true, false));
    // region_eclass_inv[root] = 0
    pair<EGraph, EGraphMapping> res = prune_unextractable_enodes(gr, 0);
    EGraph &grp = res.first;
    EGraphMapping &gr2grp = res.second;
    EClassId nroot = gr2grp.eclassidmp[0];
    EGraphMapping grp2gr = inverse_egraph_mapping(grp, res.second);
    for (EClassId i = 0; i < grp.neclasses(); ++i) {
        //composes to grp2g
        grp2gr.eclassidmp[i] = gr2g[grp2gr.eclassidmp[i]];        
    }
    DEBUG_ASSERT(is_valid_egraph_mapping(grp2gr, grp, g, false, true, false, false));
    return make_pair(grp, make_pair(nroot, grp2gr));
}

ExtractionENodeId extract_region_tiger(const EGraph &g, const EClassId root, Extraction &e, const vector<RegionId> &region_root_id, vector<pair<Extraction, ExtractionENodeId> > &region_extraction_cache, const vector<vector<Cost> > &statewalk_cost) {
    RegionId rid = region_root_id[root];
    if (region_extraction_cache[rid].second != -1) {
        return region_extraction_cache[rid].second;
    }
    if (region_extraction_cache[rid].first.size() == 0) {
        // if has not been computed yet
        pair<EGraph, pair<EClassId, EGraphMapping> > res = construct_regionalized_egraph(g, root);
        const EGraph &gr = res.first;
        const EClassId nroot = res.second.first;
        const EGraphMapping &gr2g = res.second.second;
        Extraction tmpe = extract_regionalized_egraph_tiger(gr, nroot, project_statewalk_cost(gr2g, statewalk_cost), true, true);
        region_extraction_cache[rid].first = project_extraction(gr2g, tmpe);
    }

    vector<ExtractionENodeId> subregions;
    for (ExtractionENodeId i = 0; i < (ExtractionENodeId)region_extraction_cache[rid].first.size(); ++i) {
        ExtractionENode &en = region_extraction_cache[rid].first[i];
        const ENode &n = g.eclasses[en.c].enodes[en.n];
        bool isSubregionChild = false;
        for (size_t j = 0; j < n.ch.size(); ++j) {
            const EClassId v = n.ch[j];
            if (g.eclasses[v].isEffectful) {
                if (isSubregionChild) {
                    subregions.push_back(extract_region_tiger(g, v, e, region_root_id, region_extraction_cache, statewalk_cost));
                } else {
                    isSubregionChild = true;
                }
            }
        }
    }
    ExtractionENodeId base = e.size();
    e.resize(base + region_extraction_cache[rid].first.size());
    for (ExtractionENodeId i = 0, l = 0; i < (ExtractionENodeId)region_extraction_cache[rid].first.size(); ++i) {
        ExtractionENode &nen = e[base + i], &en = region_extraction_cache[rid].first[i];
        nen.n = en.n;
        nen.c = en.c;
        const ENode &n = g.eclasses[en.c].enodes[en.n];
        nen.ch.resize(n.ch.size());
        bool isSubregionChild = false;
        for (size_t j = 0, k = 0; j < n.ch.size(); ++j) {
            const EClassId v = n.ch[j];
            if (g.eclasses[v].isEffectful) {
                if (isSubregionChild) {
                    nen.ch[j] = subregions[l++];
                } else {
                    nen.ch[j] = base + en.ch[k++];
                    isSubregionChild = true;
                }
            } else {
                nen.ch[j] = base + en.ch[k++];
            }
        }
    }
    return region_extraction_cache[rid].second = e.size() - 1;
}

vector<EClassId> find_all_region_roots(const EGraph &g, const vector<EClassId> &fun_roots) {
    region_vis.resize(g.neclasses(), 0);
    ++timestamp;
    vector<EClassId> ret;
    for (size_t i = 0; i < fun_roots.size(); ++i) {
        const EClassId v = fun_roots[i];
        if (region_vis[v] != timestamp) {
            region_vis[v] = timestamp;
            ret.push_back(v);
        }
    }
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        if (c.isEffectful) {
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                const ENode &n = c.enodes[j];
                bool isSubregionroot = false;
                for (size_t k = 0; k < n.ch.size(); ++k) {
                    EClassId v = n.ch[k];
                    if (g.eclasses[v].isEffectful) {
                        if (!isSubregionroot) {
                            isSubregionroot = true;
                        } else {
                            if (region_vis[v] != timestamp) {
                                region_vis[v] = timestamp;
                                ret.push_back(v);
                            }
                        }
                    }
                }
            }
        }
    }
    return ret;
}

vector<Extraction> extract_all_fun_roots_tiger(const EGraph &g, const vector<EClassId> &fun_roots) {

    vector<EClassId> region_roots = find_all_region_roots(g, fun_roots);
    
    vector<RegionId> region_root_id(g.neclasses(), -1);
    for (size_t i = 0; i < region_roots.size(); ++i) {
        region_root_id[region_roots[i]] = i;
    }

    vector<vector<Cost> > statewalk_cost = compute_statewalk_cost(g);

    vector<pair<Extraction, ExtractionENodeId> > region_extraction_cache(region_roots.size());    
    vector<Extraction> ret(fun_roots.size());
    for (size_t i = 0; i < fun_roots.size(); ++i) {
        EClassId fun_root = fun_roots[i];
        for (RegionId j = 0; j < (RegionId)region_roots.size(); ++j) {
            region_extraction_cache[j].second = -1;
        }
        extract_region_tiger(g, fun_root, ret[i], region_root_id, region_extraction_cache, statewalk_cost);
        DEBUG_ASSERT(is_effect_safe_extraction(g, fun_root, ret[i]));
    }
    return ret;
}
