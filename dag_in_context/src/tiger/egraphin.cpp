// utilities for egraphin format egraphs

#include<queue>
#include<cstring>

#include "egraphin.h"
#include "debug.h"

EClassParents compute_reverse_index(const EGraph &g) {
    EClassParents ret(g.neclasses());
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            for (size_t k = 0; k < n.ch.size(); ++k) {
				if (n.ch[k] != UNEXTRACTABLE_ECLASS) {
	                ret[n.ch[k]].push_back(make_pair(i, j));
				}
            }
        }
    }
    return ret;
}

ENodeCounters initialize_enode_counters(const EGraph &g) {
    ENodeCounters ret(g.neclasses());
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        ret[i].resize(c.nenodes());
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            ret[i][j] = c.enodes[j].ch.size();
        }
    }
    return ret;
}

EGraphMapping inverse_egraph_mapping(const EGraph &gp, const EGraphMapping &g2gp) {
    EGraphMapping gp2g(gp);
    for (EClassId i = 0; i < (EClassId)g2gp.eclassidmp.size(); ++i) {
        if (g2gp.eclassidmp[i] != UNEXTRACTABLE_ECLASS) {
            DEBUG_ASSERT(0 <= g2gp.eclassidmp[i] && g2gp.eclassidmp[i] < gp.neclasses());
            gp2g.eclassidmp[g2gp.eclassidmp[i]] = i;
        }
    }
    for (EClassId i = 0; i < (EClassId)g2gp.eclassidmp.size(); ++i) {
        for (ENodeId j = 0; j < (ENodeId)g2gp.enodeidmp[i].size(); ++j) {
            if (g2gp.enodeidmp[i][j] != UNEXTRACTABLE_ECLASS) {
                DEBUG_ASSERT(0 <= g2gp.enodeidmp[i][j] && g2gp.enodeidmp[i][j] < gp.eclasses[g2gp.eclassidmp[i]].nenodes());
                gp2g.enodeidmp[g2gp.eclassidmp[i]][g2gp.enodeidmp[i][j]] = j;
            }
        }
    }
    return gp2g;
}

Extraction project_extraction(const EGraphMapping &f, const Extraction &e) {
	Extraction ne(e);
	for (ExtractionENodeId i = 0; i < (ExtractionENodeId)e.size(); ++i) {
		ne[i].n = f.enodeidmp[ne[i].c][ne[i].n];
		ne[i].c = f.eclassidmp[ne[i].c];
	}
	return ne;
}

// return a mapping from the old egraph ids to the new one
pair<EGraph, EGraphMapping> prune_unextractable_enodes(const EGraph &g, const EClassId root) {
	vector<bool> extractable(g.neclasses(), false);
	EClassParents parents = compute_reverse_index(g);
	ENodeCounters cnts = initialize_enode_counters(g);
	queue<EClassId> q;
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
			if (cnts[i][j] == 0) {
				if (!extractable[i]) {
					extractable[i] = true;
					q.push(i);
				}
			}
		}
	}
	while (q.size()) {
		EClassId u = q.front();
		q.pop();
		for (size_t i = 0; i < parents[u].size(); ++i) {
			EClassId vc = parents[u][i].first;
			ENodeId vn = parents[u][i].second;
			if ((--cnts[vc][vn]) == 0) {
				if (!extractable[vc]) {
					extractable[vc] = true;
					q.push(vc);
				}
			}
		}
	}
	vector<bool> reachable(g.neclasses(), root == -1 ? true : false);
	if (root != -1) {
		reachable[root] = true;
		q.push(root);
		while (q.size()) {
			EClassId u = q.front();
			q.pop();
			const EClass &c = g.eclasses[u];
			for (ENodeId j = 0; j < c.nenodes(); ++j) {
				const ENode &n = c.enodes[j];
				bool isExtractable = true;
				for (size_t k = 0; k < n.ch.size(); ++k) {
					EClassId v = n.ch[k];
					if (v == UNEXTRACTABLE_ECLASS || !extractable[v]) {
						isExtractable = false;
						break;
					}
				}
				if (isExtractable) {
					for (size_t k = 0; k < n.ch.size(); ++k) {
						EClassId v = n.ch[k];
						if (!reachable[v]) {
							reachable[v] = true;
							q.push(v);
						}
					}
				}
			}
		}
	}
    EGraph gp;
    EGraphMapping mp(g);
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        if (reachable[i] && extractable[i]) {
            const EClass &c = g.eclasses[i];
            EClass nc;
            nc.isEffectful = c.isEffectful;
            mp.eclassidmp[i] = gp.neclasses();
            gp.eclasses.push_back(nc);
        }
    }
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        if (mp.eclassidmp[i] != UNEXTRACTABLE_ECLASS) {
            const EClass &c = g.eclasses[i];
            EClass &nc = gp.eclasses[mp.eclassidmp[i]];
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
				const ENode &n = c.enodes[j];
				bool isExtractable = true;
				for (size_t k = 0; k < n.ch.size(); ++k) {
					EClassId v = n.ch[k];
					if (v == UNEXTRACTABLE_ECLASS || mp.eclassidmp[v] == UNEXTRACTABLE_ECLASS) {
						isExtractable = false;
						break;
					}
				}
				if (isExtractable) {
                    const ENode &n = c.enodes[j];
                    ENode nn;
                    nn.head = n.head;
                    nn.ch.resize(n.ch.size());
                    for (size_t k = 0; k < n.ch.size(); ++k) {
                        nn.ch[k] = mp.eclassidmp[n.ch[k]];
                    }
                    nn.eclass = mp.eclassidmp[i];
                    mp.enodeidmp[i][j] = nc.nenodes();
                    nc.enodes.push_back(nn);
                }
            }
        }
    }
	DEBUG_ASSERT(is_wellformed_egraph(gp, false, true));
	DEBUG_ASSERT(is_valid_egraph_mapping(mp, g, gp, true, true, true, true));
    return make_pair(gp, mp);
}

EGraph read_egraph(FILE* ppin) {
	EGraph g;
	int n, cnt = 0;
	char buf[505];
	fscanf(ppin, "%d", &n);
	g.eclasses.resize(n);
	for (EClassId i = 0; i < n; ++i) {
		int f, m;
		EClass &c = g.eclasses[i];
		fscanf(ppin, "%d%d", &f, &m);
		cnt += m;
		c.isEffectful = f != 0;	
		c.enodes.resize(m);
		for (ENodeId j = 0; j < m; ++j) {
			ENode &n = c.enodes[j];
			//handle names with spaces
			fgets(buf, sizeof(buf), ppin);
			fgets(buf, sizeof(buf), ppin);
			DEBUG_ASSERT(strlen(buf) > 1);
			buf[strlen(buf) - 1] = '\0';
			n.head = buf;
			n.eclass = i;
			size_t l;
			fscanf(ppin, "%zd", &l);
			n.ch.resize(l);
			for (size_t k = 0; k < l; ++k) {
				fscanf(ppin, "%d", &n.ch[k]);
			}
			//scanf("%d", &n.cost);
		}
	}
	DEBUG_CERR(" # eclasses: " << n << "  # enodes : " << cnt);
	return g;
}

void print_egraph(const EGraph &g) {
	printf("%zu\n", g.neclasses());
	for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
		const EClass &c = g.eclasses[i];
		int f = c.isEffectful ? 1 : 0,
			m = c.nenodes();
		printf("%d %d\n", f, m);
		for (ENodeId j = 0; j < m; ++j) {
			const ENode &n = c.enodes[j];
			size_t l = n.ch.size();
			printf("%s\n%zu%c", n.head.c_str(), l, l == 0 ? '\n' : ' ');
			for (size_t k = 0; k < l; ++k) {
				printf("%d%c", n.ch[k], k == l - 1 ? '\n' : ' ');
			}
			//printf("%d\n", n.cost);
		}
	}
}

void print_extraction(const EGraph &g, const Extraction &e) {
	for (ExtractionENodeId i = 0; i < (ExtractionENodeId)e.size(); ++i) {
		printf("#%d %d%c %d %s%c", i, e[i].c, g.eclasses[e[i].c].isEffectful ? '!' : ' ', e[i].n, g.eclasses[e[i].c].enodes[e[i].n].head.c_str(), e[i].ch.size() == 0 ? '\n' : ' ');
		for (size_t j = 0; j < e[i].ch.size(); ++j) {
			printf("#%d%c", e[i].ch[j], j == e[i].ch.size() - 1 ? '\n' : ' ');
		}
	}
}