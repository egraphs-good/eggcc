#include "tiger.h"
#include "statewalkdp.h"
#include "greedy.h"
#include "debug.h"

pair<EGraph, EGraphMapping> rebuild_egraph_statewalk(const EGraph &g, const Statewalk &sw) {
    EGraph gp;
    EGraphMapping gp2g;
    gp.eclasses.resize(g.neclasses());
    gp2g.eclassidmp.resize(g.neclasses());
    gp2g.enodeidmp.resize(g.neclasses());
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        gp2g.eclassidmp[i] = i;
        const EClass &c = g.eclasses[i];
        if (!c.isEffectful) {
            gp.eclasses[i] = c;
            gp2g.enodeidmp[i].resize(c.nenodes());
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                gp2g.enodeidmp[i][j] = j;
            }
        } else {
            gp.eclasses[i].isEffectful = true;
        }
    }
    EClassId last = -1;
	for (int i = sw.size() - 1; i >= 0; --i) {
		EClassId uc = sw[i].first, vc;
		ENodeId un = sw[i].second;
		if (gp.eclasses[uc].nenodes() == 0) {
			gp.eclasses[uc].enodes.push_back(g.eclasses[uc].enodes[un]);
			vc = uc;
            gp2g.enodeidmp[vc].push_back(un);
		} else {
            vc = gp.neclasses();
			EClass c;
			c.isEffectful = true;
			c.enodes.push_back(g.eclasses[uc].enodes[un]);
            c.enodes[0].eclass = vc;
			gp.eclasses.push_back(c);
            gp2g.eclassidmp.push_back(uc);
            gp2g.enodeidmp.push_back(vector<ENodeId>(1, un));
		}
		for (size_t j = 0; j < gp.eclasses[vc].enodes[0].ch.size(); ++j) {
			EClassId chc = gp.eclasses[vc].enodes[0].ch[j];
			if (g.eclasses[chc].isEffectful) {
				gp.eclasses[vc].enodes[0].ch[j] = last;
			}
		}
		last = vc;
	}
    DEBUG_ASSERT(is_wellformed_egraph(gp, true, false));
    DEBUG_ASSERT(is_valid_egraph_mapping(gp2g, gp, g, false, false, false, true));
    return make_pair(gp, gp2g);
}

Extraction extract_regionalized_egraph_tiger(const EGraph &g, const EClassId root, const vector<vector<Cost> > &statewalk_cost) {
    Statewalk sw = statewalkDP(g, root, statewalk_cost);

    pair<EGraph, EGraphMapping> res = rebuild_egraph_statewalk(g, sw);
    const EGraph &gp = res.first;
    const EGraphMapping &gp2g = res.second;
   	pair<EGraph, EGraphMapping> res2 = prune_unextractable_enodes(gp, root);
    const EGraph &gpp = res2.first;
    const EGraphMapping &gp2gpp = res2.second;
    // root eclass id is unchanged from g to gp
    const EClassId nroot = gp2gpp.eclassidmp[root];
    Extraction e = project_extraction(gp2g, project_extraction(inverse_egraph_mapping(gpp, gp2gpp), statewalk_greedy_extraction(gpp, nroot)));
    DEBUG_ASSERT(is_effect_safe_extraction(g, root, e));
    return e;
}

