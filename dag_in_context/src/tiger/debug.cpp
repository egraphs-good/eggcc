#include "debug.h"

void json_print_egraph(const EGraph &g) {
    vector<string> name_for_eclass;
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const ENode &n = g.eclasses[i].enodes[0];
        name_for_eclass.push_back(n.get_name());
    }

    fprintf(stderr, ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>begin:json_print_egraph\n");
    fprintf(stderr, "{\n");
    fprintf(stderr, "\t\"nodes\": {\n");
    bool isfirst = true;
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        for (int j = 0; j < (ENodeId)g.eclasses[i].nenodes(); ++j) {
            const ENode &n = g.eclasses[i].enodes[j];
            if (!isfirst) {
                fprintf(stderr, "\t\t},\n");
            }
            isfirst = false;
            fprintf(stderr, "\t\t\"%s\": {\n", n.get_name().c_str());
            fprintf(stderr, "\t\t\t\"op\": \"%s\",\n", n.get_op().c_str());
            fprintf(stderr, "\t\t\t\"children\": [\n");
            for (size_t k = 0; k < n.ch.size(); ++k) {
                fprintf(stderr, "\t\t\t\t\"%s\"%s\n", name_for_eclass[n.ch[k]].c_str(), k == (int)n.ch.size() - 1 ? "" : ",");
            }
            fprintf(stderr, "\t\t\t],\n");
            fprintf(stderr, "\t\t\t\"eclass\": \"%d\"\n", i);
        }
    }
    isfirst = true;
    fprintf(stderr, "\t\t}\n");
    fprintf(stderr, "\t},\n");
    fprintf(stderr, "\t\"class_data\": {\n");
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        if (!isfirst) {
            fprintf(stderr, "\t\t},\n");
        }
        isfirst = false;
        fprintf(stderr, "\t\t\"%d\": {\n", i);
        fprintf(stderr, "\t\t\t\"type\": \"%s\"\n", g.eclasses[i].isEffectful ? "Effectful" : "Pure");
    }
    fprintf(stderr, "\t\t}\n");
    fprintf(stderr, "\t}\n");
    fprintf(stderr, "}\n");
    fprintf(stderr, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<end:json_print_egraph\n");
    fflush(stderr);
}

void debug_print_egraph(const EGraph &g) {
    fprintf(stderr, ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>begin:debug_print_egraph\n");
    size_t cnt = 0;
	for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
		cnt += g.eclasses[i].nenodes();
	}
	fprintf(stderr, "# eclasses: %zu\n# enodes: %zu\n", g.neclasses(), cnt);
	for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
		fprintf(stderr, "# eclass %d\n", i);
		const EClass &c = g.eclasses[i];
		int f = c.isEffectful ? 1 : 0,
			m = c.enodes.size();
		fprintf(stderr, "%d %d\n", f, m);
		for (ENodeId j = 0; j < m; ++j) {
			const ENode &n = c.enodes[j];
			size_t l = n.ch.size();
			fprintf(stderr, "%s\n%zu%c", n.head.c_str(), l, l == 0 ? '\n' : ' ');
			for (size_t k = 0; k < l; ++k) {
				fprintf(stderr, "%s%d%c", g.eclasses[n.ch[k]].isEffectful ? "!" : " ", n.ch[k], k == l - 1 ? '\n' : ' ');
			}
			//printf("%d\n", n.cost);
		}
		fprintf(stderr,"\n");
	}
    fprintf(stderr, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<end:debug_print_egraph\n");
    fflush(stderr);
}

bool is_wellformed_egraph(const EGraph &g, bool allow_unextractable_child, bool allow_subregion_child) {
    bool ret = true;
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        if (!(allow_unextractable_child || c.nenodes() > 0)) {
            ret = false;
            cerr << "Error: Found empty eclass" << endl;
        }
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            if (!(n.eclass == i)) {
                ret = false;
                cerr << "Error: Wrong eclass for an enode" << endl;
            }
            int effectful_ch_cnt = 0;
            for (size_t k = 0; k < n.ch.size(); ++k) {
                const EClassId chc = n.ch[k];
                if (!((allow_unextractable_child && chc == UNEXTRACTABLE_ECLASS) || (0 <= chc && chc < g.neclasses()))) {
                    ret = false;
                    cerr << "Error: Invalid child edge " << i << "," << j << "," << k << endl; 
                }
                // assuming unextractable eclasses are effectful
                if (chc == UNEXTRACTABLE_ECLASS || g.eclasses[chc].isEffectful) {
                    ++effectful_ch_cnt;
                }
            }
            if (!allow_subregion_child && effectful_ch_cnt > 1) {
                ret = false;
                cerr << "Error: Found subregion child at enode " <<  i << "," << j << endl;
            }
        }
    }
    if (!ret) {
        debug_print_egraph(g);
    }
    return ret;
}

void debug_print_egraph_mapping(const EGraphMapping &g2gp, const EGraph &g, const EGraph &gp) {
    fprintf(stderr, ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>begin:debug_print_egraph_mapping\n");
    debug_print_egraph(g);
    debug_print_egraph(gp);
    fprintf(stderr, "%zu\n", g2gp.eclassidmp.size());
    for (EClassId i = 0; i < (EClassId)g2gp.eclassidmp.size(); ++i) {
        fprintf(stderr, "\t%d -> %d\n", i, g2gp.eclassidmp[i]);
    }
    fprintf(stderr, "---\n");
    fprintf(stderr, "%zu\n", g2gp.enodeidmp.size());
    for (EClassId i = 0; i < (EClassId)g2gp.enodeidmp.size(); ++i) {
        fprintf(stderr, "\t%d: %zu\n", i, g2gp.enodeidmp[i].size());
        for (ENodeId j = 0; j < (ENodeId)g2gp.enodeidmp[i].size(); ++j) {
            fprintf(stderr, "\t\t%d -> %d\n", j, g2gp.enodeidmp[i][j]);
        }
    }
    fprintf(stderr, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<end:debug_print_egraph_mapping\n");
    fflush(stderr);
}

bool is_valid_egraph_mapping_helper(const EGraphMapping &g2gp, const EGraph &g, const EGraph &gp, bool isPartial, bool isInjective, bool isSurjective, bool checkChildrenConsistency) {
    if (g2gp.eclassidmp.size() != g.neclasses() || g2gp.enodeidmp.size() != g.neclasses()) {
        cerr << "Error: Wrong domain #eclasses" << endl;
        return false;
    }
    vector<vector<bool> > vis_gp(gp.neclasses());
    for (EClassId i = 0; i < (EClassId)gp.neclasses(); ++i) {
        vis_gp[i].resize(gp.eclasses[i].nenodes(), false);
    }
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        if (g2gp.enodeidmp[i].size() != c.nenodes()) {
            cerr << "Error: Wrong domain #enodes in eclass #" << i << endl;
            return false;
        }
        const EClassId cpid = g2gp.eclassidmp[i];
        if (isPartial && cpid == UNEXTRACTABLE_ECLASS) {
            continue;
        }
        if (!(0 <= cpid && cpid < gp.neclasses())) {
            cerr << "Error: Invalid codomain eclass for eclass #" << i << " mapped to #" << cpid << endl;
            return false;
        }
        const EClass &cp = gp.eclasses[cpid];
        if (c.isEffectful != cp.isEffectful) {
            cerr << "Error: Mismatching effectful flags for eclass #" << i << " mapped to #" << cpid << endl;
            return false;
        }
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            const ENodeId npid = g2gp.enodeidmp[i][j];
            if (isPartial && npid == UNEXTRACTABLE_ECLASS) {
                continue;
            }
            if (!(0 <= npid && npid < cp.nenodes())) {
                cerr << "Error: Invalid codomain enode for enode #" << i << "," << j << " mapped to #" << cpid << "," << npid << endl;
                return false;
            }
            if (isInjective && vis_gp[cpid][npid]) {
                cerr << "Error: egraph mapping not injective / multiple enodes map into the same enode" << endl;
                return false;
            }
            vis_gp[cpid][npid] = true;
            if (checkChildrenConsistency) {
                const ENode &np = cp.enodes[npid];
                if (n.ch.size() != np.ch.size()) {
                    cerr << "Error: Children inconsistency: different number of children of enode #" << i << "," << j << " mapped to #" << cpid << "," << npid << endl;
                    return false;
                }
                for (size_t k = 0; k < n.ch.size(); ++k) {
                    EClassId chc = n.ch[k], chcp = np.ch[k];
                    if (g2gp.eclassidmp[chc] != chcp) {
                        cerr << "Error: Children inconsistency: bad child eclass of enode #" << i << "," << j << "," << chc << " mapped to #" << cpid << "," << npid << "," << chcp << endl;
                        return false;
                    }
                }
            }
        }
    }
    if (isSurjective) {
        for (EClassId i = 0; i < (EClassId)gp.neclasses(); ++i) {
            for (ENodeId j = 0; j < (ENodeId)gp.eclasses[i].nenodes(); ++j) {
                if (!vis_gp[i][j]) {
                    cerr << "Error: egraph mapping not surjective / enode in codomain not mapped " << i << "," << j << endl;
                    return false;
                }
            }
        }
    }
    return true;
}

bool is_valid_egraph_mapping(const EGraphMapping &g2gp, const EGraph &g, const EGraph &gp, bool isPartial, bool isInjective, bool isSurjective, bool checkChildrenConsistency) {
    bool ret = is_valid_egraph_mapping_helper(g2gp, g, gp, isPartial, isInjective, isSurjective, checkChildrenConsistency);
    if (!ret) {
        debug_print_egraph_mapping(g2gp, g, gp);
    }
    return ret;
}

bool arg_check_regionalized_egraph(const EGraph &g) {
    bool ret = true;
    int cntn = 0, cntc = 0;
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        if (c.isEffectful) {
            bool found_arg = false;
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                const ENode &n = c.enodes[j];
                if (n.ch.size() == 0) {
                    found_arg = true;
                    ++cntn;
                }
            }
            if (found_arg) {
                ++cntc;
            }
        }
    }
    if (cntn == 0) {
        ret = false;
        cerr << "Error: Found no arg in a regionalized egraph" << endl;
    }
    if (cntn > 1) {
        ret = false;
        cerr << "Error: Found multiple arg enodes in a regionalized egraph #" << cntn << endl;
        if (cntc > 1) {
            cerr << "Error: Found multiple arg eclasses in a regionalized egraph #" << cntc << endl;
        }
    }
    if (!ret) {
        debug_print_egraph(g);
    }
    return ret;
}

bool is_valid_statewalk(const EGraph &g, const EClassId root, const Statewalk &sw) {
    bool ret = true;
    if (sw.size() == 0 || sw[0].first != root) {
        ret = false;
        cerr << "Error: Statewalk does not start with the root eclass" << endl;
    }
    for (size_t i = 0; i < sw.size(); ++i) {
        EClassId cid = sw[i].first;
        ENodeId nid = sw[i].second;
        if (!(0 <= cid && cid < g.neclasses())) {
            ret = false;
            cerr << "Error: Invalid eclassid" << endl;
            break;
        }
        const EClass &c = g.eclasses[cid];
        if (!(0 <= nid && nid < c.nenodes())) {
            ret = false;
            cerr << "Error: Invalid enodeid" << endl;
            break;
        }
        const ENode &n = g.eclasses[cid].enodes[nid];
        if (i + 1 < sw.size()) {
            EClassId efchcid = UNEXTRACTABLE_ECLASS;
            for (size_t j = 0; j < n.ch.size(); ++j) {
                EClassId chc = n.ch[j];
                if (g.eclasses[chc].isEffectful) {
                    efchcid = chc;
                }
            }
            if (efchcid == UNEXTRACTABLE_ECLASS) {
                ret = false;
                cerr << "Error: Invalid prefix with no connection" << endl;
                break;
            }
            if (efchcid != sw[i + 1].first) {
                ret = false;
                cerr << "Error: Mismatched child eclass" << endl;
                break;
            }
        } else {
            if (n.ch.size() != 0) {
                ret = false;
                cerr << "Error: Statewalk does not end with an arg" << endl;
            }
        }
    }
    if (!ret) {
        debug_print_egraph(g);
        for (size_t i = 0; i < sw.size(); ++i) {
            fprintf(stderr, "%d %d", sw[i].first , sw[i].second);
        }
    }
    return ret;
}

bool is_valid_extraction_strict_helper(const EGraph &g, const EClassId root, const Extraction &e) {
	if (e.size() == 0 || e.back().c != root) { // root
		cerr << "Error: The last element of the extraction must be the root." << endl;
		return false;
	}
	for (ExtractionENodeId i = (ExtractionENodeId)e.size() - 1; i >= 0; --i) {
		const ExtractionENode &n = e[i];
		if (n.c < 0 || n.c >= g.eclasses.size()) {
			cerr << "Error: Extraction referring to an eclass outside of bounds." << endl;
			return false;
		}
		if (n.n < 0 || n.n >= g.eclasses[n.c].enodes.size()) {
			cerr << "Error: Extraction referring to an enode outside of bounds." << endl;
			return false;
		}
		if (n.ch.size() != g.eclasses[n.c].enodes[n.n].ch.size()) {
			cerr << "Error: Extraction referring to a wrong number of children." << endl;
			return false;
		}
		for (size_t j = 0; j < n.ch.size(); ++j) {
			ExtractionENodeId ch = n.ch[j];
			if (ch < 0 || ch >= e.size()) { // child present
				cerr << "Error: Extraction referring to an index outside of bounds." << endl;
				cerr << "Found: " << ch << endl;
				return false;
			}
			EClassId expected_child = g.eclasses[n.c].enodes[n.n].ch[j];
			if (e[ch].c != expected_child) {
				cerr << "Error: Extraction referring to a child of wrong eclass." << endl;
				return false;
			}
			if (ch >= i) { // acyclicity
				cerr << "Error: Extraction may contain a loop." << endl;
				return false;
			}
		}
	}
	// reachability does not really matter
	// unique choice not required 
	return true;
}


void debug_print_extraction(const EGraph &g, const Extraction &e) {
	for (ExtractionENodeId i = 0; i < (ExtractionENodeId)e.size(); ++i) {
		fprintf(stderr, "#%d %d%c %d %s%c", i, e[i].c, g.eclasses[e[i].c].isEffectful ? '!' : ' ', e[i].n, g.eclasses[e[i].c].enodes[e[i].n].head.c_str(), e[i].ch.size() == 0 ? '\n' : ' ');
		for (size_t j = 0; j < e[i].ch.size(); ++j) {
			fprintf(stderr, "#%d%c", e[i].ch[j], j == e[i].ch.size() - 1 ? '\n' : ' ');
		}
	}
}

bool is_valid_extraction(const EGraph &g, const EClassId root, const Extraction &e) {
    bool ret = is_valid_extraction_strict_helper(g, root, e);
    if (!ret) {
        debug_print_egraph(g);
        debug_print_extraction(g, e);
    }
    return ret;
}

bool is_effect_safe_extraction_helper(const EGraph &g, const ExtractionENodeId rootid, const Extraction &e, vector<bool> &subregion_checked) {
	vector<ExtractionENodeId> statewalk;
	vector<bool> vis(e.size(), false);
	vector<bool> onpath(e.size(), false);
	queue<ExtractionENodeId> q;
	statewalk.push_back(rootid);
	onpath[rootid] = true;
	for (size_t i = 0; i < statewalk.size(); ++i) {
		ExtractionENodeId u = statewalk[i];
		ExtractionENodeId nxt = -1;
		for (size_t j = 0; j < e[u].ch.size(); ++j) {
            ExtractionENodeId che = e[u].ch[j];
			if (g.eclasses[e[che].c].isEffectful) {
				if (nxt == -1) {
					nxt = che;
					statewalk.push_back(nxt);
					onpath[nxt] = true;
				} else if (!is_effect_safe_extraction_helper(g, che, e, subregion_checked)) {
			        return false;
				}
			} else {
				if (!vis[che]) {
					vis[che] = true;
					q.push(che);
				}
			}
		}
	}
	// Check pure enodes only depend on the effectful walk in this region
	while (q.size()) {
		ExtractionENodeId u = q.front();
		q.pop();
		for (size_t i = 0; i < e[u].ch.size(); ++i) {
			ExtractionENodeId v = e[u].ch[i];
			// assuming pure enodes can only have one effectful child
			if (g.eclasses[e[v].c].isEffectful) {
				if (!onpath[v]) {
					// using a effectul enode not in this region
                    cerr << "Error: Using an effectul node not on the region statewalk" << endl;
					return false;
				}
			} else {
				if (!vis[v]) {
					vis[v] = true;
					q.push(v);
				}
			}
		}
	}
    subregion_checked[rootid] = true;
    return true;
}

bool is_effect_safe_extraction(const EGraph &g, const EClassId root, const Extraction &e) {
    if (!is_valid_extraction(g, root, e)) {
        return false;
    }
    // prevent double-checking each subregion
    vector<bool> subregion_checked(e.size(), false);
    bool ret = is_effect_safe_extraction_helper(g, e.size() - 1, e, subregion_checked);
    if (!ret) {
        debug_print_egraph(g);
        debug_print_extraction(g, e);
    }
    return ret;
}
