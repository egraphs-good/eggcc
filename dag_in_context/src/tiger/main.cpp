#include<map>
#include<set>
#include<queue>
#include<vector>
#include<cstdio>
#include<climits>
#include<cassert>
#include<cstring>
#include<string>
#include<iostream>
#include<algorithm>

using namespace std;

const bool DEBUG = false;

bool g_ilp_mode = false;

const char* TMPFILENAME = "extract.tmp";

FILE* preprocessing() {
	FILE *out = fopen(TMPFILENAME, "w");
	char buf[505];
	while (fgets(buf, sizeof(buf), stdin) != NULL) {
		if (buf[0] != '#') {
			fprintf(out, "%s", buf);
		}
	}
	fclose(out);
	return fopen(TMPFILENAME, "r");
}

typedef int EClassId;

struct ENode {
	string head;
	EClassId eclass;
	vector<EClassId> ch;
	//int cost;
};

struct EClass {
	vector<ENode> enodes;
	bool isEffectful;
};

struct EGraph {
	vector<EClass> eclasses;
};

EGraph read_egraph(FILE* ppin) {
	EGraph g;
	int n, cnt = 0;
	fscanf(ppin, "%d", &n);
	g.eclasses.resize(n);
	for (int i = 0; i < n; ++i) {
		int f, m;
		EClass &c = g.eclasses[i];
		fscanf(ppin, "%d%d", &f, &m);
		cnt += m;
		c.isEffectful = f != 0;	
		c.enodes.resize(m);
		for (int j = 0; j < m; ++j) {
			ENode &n = c.enodes[j];
			char buf[505];
			//handle names with spaces
			fgets(buf, sizeof(buf), ppin);
			fgets(buf, sizeof(buf), ppin);
			assert(strlen(buf) > 1);
			buf[strlen(buf) - 1] = '\0';
			n.head = buf;
			n.eclass = i;
			int l;
			fscanf(ppin, "%d", &l);
			n.ch.resize(l);
			for (int k = 0; k < l; ++k) {
				fscanf(ppin, "%d", &n.ch[k]);
			}
			//scanf("%d", &n.cost);
		}
	}
	cerr << " # eclasses: " << n << "  # enodes : " << cnt << endl;
	return g;
}

void print_egraph(const EGraph &g) {
	int n = g.eclasses.size();
	printf("%d\n", n);
	for (int i = 0; i < n; ++i) {
		const EClass &c = g.eclasses[i];
		int f = c.isEffectful ? 1 : 0,
			m = c.enodes.size();
		printf("%d %d\n", f, m);
		for (int j = 0; j < m; ++j) {
			const ENode &n = c.enodes[j];
			int l = n.ch.size();
			printf("%s\n%d%c", n.head.c_str(), l, l == 0 ? '\n' : ' ');
			for (int k = 0; k < l; ++k) {
				printf("%d%c", n.ch[k], k == l - 1 ? '\n' : ' ');
			}
			//printf("%d\n", n.cost);
		}
	}
}

void debugprint_egraph(const EGraph &g) {
	int n = g.eclasses.size(), cnt = 0;
	for (int i = 0; i < n; ++i) {
		cnt += g.eclasses[i].enodes.size();
	}
	printf("# eclasses: %d\n# enodes: %d\n", n, cnt);
	for (int i = 0; i < n; ++i) {
		printf("# eclass %d\n", i);
		const EClass &c = g.eclasses[i];
		int f = c.isEffectful ? 1 : 0,
			m = c.enodes.size();
		printf("%d %d\n", f, m);
		for (int j = 0; j < m; ++j) {
			const ENode &n = c.enodes[j];
			int l = n.ch.size();
			printf("%s\n%d%c", n.head.c_str(), l, l == 0 ? '\n' : ' ');
			for (int k = 0; k < l; ++k) {
				printf("%s%d%c", g.eclasses[n.ch[k]].isEffectful ? "!" : " ", n.ch[k], k == l - 1 ? '\n' : ' ');
			}
			//printf("%d\n", n.cost);
		}
		printf("\n");
	}
}

typedef int ENodeId;

typedef int ExtractionENodeId;

struct ExtractionENode {
	EClassId c;
	ENodeId n;
	vector<ExtractionENodeId> ch;
};

typedef vector<ExtractionENode> Extraction;

bool validExtraction(const EGraph &g, const EClassId root, const Extraction &e) {
	if (e.size() == 0 || e.back().c != root) { // root
		cerr << "Error: The first element of the extraction must be the root." << endl;
		return false;
	}
	for (int i = (int)e.size() - 1; i >= 0; --i) {
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
		for (int j = 0; j < (int)n.ch.size(); ++j) {
			ExtractionENodeId ch = n.ch[j];
			if (ch < 0 || ch >= e.size()) { // child present
				cerr << "Error: Extraction referring to an index outside of bounds." << endl;
				cerr << "Found: " << ch << endl;
				return false;
			}
			if (e[ch].c != g.eclasses[n.c].enodes[n.n].ch[j]) {
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

void print_extraction(const EGraph &g, const Extraction &e) {
	for (int i = 0; i < (int)e.size(); ++i) {
		printf("#%d %d%c %d %s%c", i, e[i].c, g.eclasses[e[i].c].isEffectful ? '!' : ' ', e[i].n, g.eclasses[e[i].c].enodes[e[i].n].head.c_str(), e[i].ch.size() == 0 ? '\n' : ' ');
		for (int j = 0; j < (int)e[i].ch.size(); ++j) {
			printf("#%d%c", e[i].ch[j], j == e[i].ch.size() - 1 ? '\n' : ' ');
		}
	}
}

typedef set<EClassId> SCost;

SCost& operator += (SCost &a, const SCost &b) {
	for (SCost::iterator it = b.begin(); it != b.end(); ++it) {
		a.insert(*it);
	}
	return a;
}

SCost singleton(EClassId i) {
	SCost ret;
	ret.insert(i);
	return ret;
}

pair<bool, Extraction> NormalGreedyExtraction(const EGraph &g, EClassId root) {
	vector<SCost> dis(g.eclasses.size());
	vector<ENodeId> pick(g.eclasses.size(), -1);
	priority_queue<pair<int, EClassId> > maxheap;
	for (int i = 0; i < (int)g.eclasses.size(); ++i) {
		for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
			if (g.eclasses[i].enodes[j].ch.size() == 0) {
				dis[i] = singleton(i);
				pick[i] = j;
				maxheap.push(make_pair(-dis[i].size(), i));
				break;
			}
		}
	}

	// crazy optimization, don't bother
	vector<vector<pair<EClassId, ENodeId> > > rev_ind(g.eclasses.size());
	vector<vector<pair<int, SCost> > > counters(g.eclasses.size());
	for (EClassId i = 0; i < (int)g.eclasses.size(); ++i) {
		counters[i].resize(g.eclasses[i].enodes.size());
		for (ENodeId j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
			counters[i][j] = make_pair(g.eclasses[i].enodes[j].ch.size(), singleton(i));
			for (int k = 0; k < (int)g.eclasses[i].enodes[j].ch.size(); ++k) {
				rev_ind[g.eclasses[i].enodes[j].ch[k]].push_back(make_pair(i, j));
			}
		}
	}

	while (maxheap.size() > 0) {
		int d = -maxheap.top().first;
		EClassId i = maxheap.top().second;
		maxheap.pop();
		if (d == dis[i].size()) {
			//cerr << i << ' ' << d << endl;
			for (int j = 0; j < (int)rev_ind[i].size(); ++j) {
				EClassId vc = rev_ind[i][j].first;
				ENodeId vn = rev_ind[i][j].second;
				--counters[vc][vn].first;
				counters[vc][vn].second += dis[i];
				if (counters[vc][vn].first == 0 && (dis[vc].size() == 0 || (counters[vc][vn].second.size() < dis[vc].size()))) {
					dis[vc] = counters[vc][vn].second;
					pick[vc] = vn;
					maxheap.push(make_pair(-dis[vc].size(), vc));
				}
			}
		}
	}

	if (dis[root].size() == 0) {
		return make_pair(false, Extraction());
	}

	vector<bool> inExtraction(g.eclasses.size(), false);
	queue<EClassId> q;
	inExtraction[root] = true;
	q.push(root);
	vector<pair<int, EClassId> > ord;
	while (q.size() > 0) {
		EClassId c = q.front();
		ord.push_back(make_pair(dis[c].size(), c));
		q.pop();
		for (int i = 0; i < (int)g.eclasses[c].enodes[pick[c]].ch.size(); ++i) {
			EClassId chc = g.eclasses[c].enodes[pick[c]].ch[i];
			if (!inExtraction[chc]) {
				inExtraction[chc] = true;
				q.push(chc);
			}
		}
	}
	sort(ord.begin(), ord.end());

	Extraction e;
	vector<ExtractionENodeId> idmap(g.eclasses.size(), -1);
	for (int i = 0; i < (int)ord.size(); ++i) {
		ExtractionENode n;
		n.c = ord[i].second;
		n.n = pick[n.c];
		for (int j = 0; j < (int)g.eclasses[n.c].enodes[n.n].ch.size(); ++j) {
			//cerr << dis[n.c].size() << ' ' << dis[g.eclasses[n.c].enodes[n.n].ch[j]].size() << endl;
			assert(dis[n.c].size() > dis[g.eclasses[n.c].enodes[n.n].ch[j]].size());
			n.ch.push_back(idmap[g.eclasses[n.c].enodes[n.n].ch[j]]);
		}
		e.push_back(n);
		idmap[n.c] = i;
	}
	assert(validExtraction(g, root, e));
	return make_pair(true, e);
}

typedef int Cost;

struct SubEGraphMap {
	vector<EClassId> eclassmp;
	map<EClassId, EClassId> inv;
	vector<vector<int> > nsubregion;
};

pair<EGraph, SubEGraphMap> createRegionEGraph(const EGraph &g, const EClassId region_root) {
	SubEGraphMap mp;
	mp.inv[region_root] = 0;
	mp.eclassmp.push_back(region_root);
	mp.nsubregion.push_back(vector<int>(g.eclasses[region_root].enodes.size(), 0));
	// First BFS: only follow non-subregion child edges to get connected effectful eclasses
	for (int _ = 0; _ < (int)mp.eclassmp.size(); ++_) {
		EClassId u = mp.eclassmp[_];
		assert(mp.nsubregion[mp.inv[u]].size() == g.eclasses[u].enodes.size());
		for (int i = 0; i < (int)g.eclasses[u].enodes.size(); ++i) {
			bool subregionchild = false;
			for (int j = 0; j < (int)g.eclasses[u].enodes[i].ch.size(); ++j) {
				EClassId v = g.eclasses[u].enodes[i].ch[j];
				if (g.eclasses[v].isEffectful) {
					if (subregionchild) {
						mp.nsubregion[mp.inv[u]][i]++;
						continue;
					}
					subregionchild = true;
					if (!mp.inv.count(v)) {
						mp.inv[v] = mp.eclassmp.size();
						mp.eclassmp.push_back(v);
						mp.nsubregion.push_back(vector<int>(g.eclasses[v].enodes.size(), 0));
					}
				}
			}
		}
	}
	// Second BFS: only look at pure children
	for (int _ = 0; _ < (int)mp.eclassmp.size(); ++_) {
		EClassId u = mp.eclassmp[_];
		for (int i = 0; i < (int)g.eclasses[u].enodes.size(); ++i) {
			for (int j = 0; j < (int)g.eclasses[u].enodes[i].ch.size(); ++j) {
				EClassId v = g.eclasses[u].enodes[i].ch[j];
				if (!g.eclasses[v].isEffectful) {
					if (!mp.inv.count(v)) {
						mp.inv[v] = mp.eclassmp.size();
						mp.eclassmp.push_back(v);
						mp.nsubregion.push_back(vector<int>(g.eclasses[v].enodes.size(), 0));
					}
				}
			}
		}
	}
	// Add all the enodes
	EGraph gr;
	for (int i = 0; i < (int)mp.eclassmp.size(); ++i) {
		EClass c;
		c.isEffectful = g.eclasses[mp.eclassmp[i]].isEffectful;
		c.enodes.resize(g.eclasses[mp.eclassmp[i]].enodes.size());
		for (int j = 0; j < (int)g.eclasses[mp.eclassmp[i]].enodes.size(); ++j) {
			bool subregionchild = false;
			c.enodes[j].eclass = i;
			c.enodes[j].head = g.eclasses[mp.eclassmp[i]].enodes[j].head;
			for (int k = 0; k < (int)g.eclasses[mp.eclassmp[i]].enodes[j].ch.size(); ++k) {
				EClassId cp = g.eclasses[mp.eclassmp[i]].enodes[j].ch[k];
				if (g.eclasses[cp].isEffectful) {
					if (subregionchild) {
						continue;
					}
					subregionchild = true;
				}
				c.enodes[j].ch.push_back(mp.inv[cp]);
			}
		}
		gr.eclasses.push_back(c);
	}
	return make_pair(gr, mp);
}

bool checkLinearRegionRec(const EGraph &g, const ExtractionENodeId rootid, const Extraction &e) {
	// cout << "Checking region linearity: " << rootid << endl;
	// Find statewalk and subregions
	vector<ExtractionENodeId> statewalk;
	vector<ExtractionENodeId> subregions;
	vector<bool> vis(e.size(), false);
	vector<bool> onpath(e.size(), false);
	queue<ExtractionENodeId> q;
	statewalk.push_back(rootid);
	onpath[rootid] = true;
	for (int i = 0; i < (int)statewalk.size(); ++i) {
		int u = statewalk[i];
		int nxt = -1;
		for (int j = 0; j < (int)e[u].ch.size(); ++j) {
			if (g.eclasses[e[e[u].ch[j]].c].isEffectful) {
				if (nxt == -1) {
					nxt = e[u].ch[j];
					statewalk.push_back(nxt);
					onpath[nxt] = true;
				} else {
					subregions.push_back(e[u].ch[j]);
				}
			} else {
				if (!vis[e[u].ch[j]]) {
					vis[e[u].ch[j]] = true;
					q.push(e[u].ch[j]);
				}
			}
		}
	}
	// Check pure enodes only depend on the effectful walk in this region
	while (q.size()) {
		int u = q.front();
		q.pop();
		for (int i = 0; i < (int)e[u].ch.size(); ++i) {
			int v = e[u].ch[i];
			// assuming pure enodes can only have one effectful child
			if (g.eclasses[e[v].c].isEffectful) {
				if (!onpath[v]) {
					// using a effectul enode not in this region
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
	// Check all the subregions
	for (int i = 0; i < (int)subregions.size(); ++i) {
		if (!checkLinearRegionRec(g, subregions[i], e)) {
			return false;
		}
	}
	return true;
}

bool linearExtraction(const EGraph &g, const EClassId root, const Extraction &e) {
	if (!validExtraction(g, root, e)) {
		return false;
	}
	assert(g.eclasses[root].isEffectful);
	ExtractionENodeId rootid = e.size() - 1;
	assert(e[rootid].c == root);
	return checkLinearRegionRec(g, rootid, e);
}


typedef vector<pair<EClassId, ENodeId> > StateWalk;

/*
StateWalk getStateWalk(const EGraph &g, const EClassId root, const Extraction &e) {
	assert(validExtraction(g, root, e));
	assert(g.eclasses[root].isEffectful);
	StateWalk sw;
	int cur = e.size() - 1;
	while (cur != -1) {
		sw.push_back(make_pair(e[cur].c, e[cur].n));
		int nxt = -1;
		for (int i = 0; i < (int)e[cur].ch.size(); ++i) {
			if (g.eclasses[e[e[cur].ch[i]].c].isEffectful) {
				nxt = e[cur].ch[i];
				break;
			}
		}
		cur = nxt;
	}
	return sw;
}
*/

pair<bool, Extraction> regionExtractionWithStateWalk(const EGraph &g, const EClassId root, const StateWalk &sw) {
	EGraph gp = g;
	// 1. remove all enodes in effectful eclasses
	for (int i = 0; i < (int)gp.eclasses.size(); ++i) {
		if (gp.eclasses[i].isEffectful) {
			gp.eclasses[i].enodes.clear();
		}
	}
	// 2. add back all the enodes in the statewalk as individual eclasses
	EClassId last = -1;
	vector<int> ecmap(g.eclasses.size() + sw.size());
	for (int i = sw.size() - 1; i >= 0; --i) {
		EClassId cg = sw[i].first, cgp;
		ENodeId ng = sw[i].second;
		if (gp.eclasses[cg].enodes.size() == 0) {
			gp.eclasses[cg].enodes.push_back(g.eclasses[cg].enodes[ng]);
			cgp = cg;
		} else {
			cgp = gp.eclasses.size();
			EClass ec;
			ec.isEffectful = true;
			ec.enodes.push_back(g.eclasses[cg].enodes[ng]);
			gp.eclasses.push_back(ec);
		}
		bool subregionchild = false;
		for (int j = 0; j < (int)gp.eclasses[cgp].enodes[0].ch.size(); ++j) {
			EClassId chc = gp.eclasses[cgp].enodes[0].ch[j];
			if (g.eclasses[chc].isEffectful) {
				assert(!subregionchild); // at most one effectful child in a region egraph
				gp.eclasses[cgp].enodes[0].ch[j] = last;
				subregionchild = true;
			}
		}
		ecmap[cgp] = i;
		last = cgp;
	}
	// 3. is automatically handled by keeping the edges of the pure enodes
	// printf("Reconstructed egraph gprime:\n");
	// print_egraph(gp);
	pair<bool, Extraction> result = NormalGreedyExtraction(gp, root);
	if (result.first == false) {
		return result;
	} else {
		//Reconstruct the extraction in G
		Extraction &e = result.second;
		for (int i = 0; i < (int)e.size(); ++i) {
			if (gp.eclasses[e[i].c].isEffectful) {
				e[i].n = sw[ecmap[e[i].c]].second;
				e[i].c = sw[ecmap[e[i].c]].first;
			}
		}
		assert(validExtraction(g, root, e));
		assert(linearExtraction(g, root, e));
		return result;
	}
}

typedef vector<bool> ExtractableSet;

typedef vector<unsigned long long> CompressedSet;

inline bool isExtractable(const EGraph &g, const ExtractableSet &es, const EClassId c, const ENodeId n) {
	for (int i = 0; i < (int)g.eclasses[c].enodes[n].ch.size(); ++i) {
		EClassId chc = g.eclasses[c].enodes[n].ch[i];
		if (!es[chc]) {
			return false;
		}
	}
	return true;
}

CompressedSet compress(const EGraph &g, const ExtractableSet &es, const EClassId &frontier) {
	// TODO: add liveness analysis and 
	CompressedSet ret((g.eclasses.size() + 63) / 64, 0) ;
	for (int i = 0; i < (int)g.eclasses.size(); ++i) {
		if (!g.eclasses[i].isEffectful && es[i]) {
			ret[i / 64] |= 1ull << (i % 64);
		}
	}
	return ret;
} 

ExtractableSet saturate_pure(const EGraph &g, const ExtractableSet &es) {
	ExtractableSet ret(es.size(), false);
	queue<EClassId> q;
	vector<vector<pair<EClassId, ENodeId> > > edges(g.eclasses.size());
	vector<vector<int> > counters(g.eclasses.size());
	for (int i = 0; i < (int)g.eclasses.size(); ++i) {
		if (es[i]) {
			q.push(i);
			ret[i] = true;
		} else {
			if (!g.eclasses[i].isEffectful) {
				bool canextract = false;
				for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
					if (g.eclasses[i].enodes[j].ch.size() == 0) {
						canextract = true;
					}
				}
				if (canextract) {
					q.push(i);
					ret[i] = true;
				} else {
					counters[i].resize(g.eclasses[i].enodes.size());
					for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
						counters[i][j] = g.eclasses[i].enodes[j].ch.size();
						for (int k = 0; k < (int)g.eclasses[i].enodes[j].ch.size(); ++k) {
							edges[g.eclasses[i].enodes[j].ch[k]].push_back(make_pair(i, j));
						}
					}
				}
			}
		}
	}
	while (q.size() > 0) {
		int u = q.front();
		q.pop();
		for (int i = 0; i < (int)edges[u].size(); ++i) {
			EClassId vc = edges[u][i].first;
			ENodeId vn = edges[u][i].second;
			if (!ret[vc]) {
				--counters[vc][vn];
				if (counters[vc][vn] == 0) {
					q.push(vc);
					ret[vc] = true;
				}
			}
		}
	}
	return ret;
}

typedef pair<EClassId, ExtractableSet> ExVertex;
typedef pair<EClassId, CompressedSet> IndVertex;
typedef int ExVertexId;

StateWalk UnguidedFindStateWalk(const EGraph &g, const EClassId initc, const ENodeId initn, const EClassId root, const vector<vector<int> > &nsubregion) {
	//cout << "!!!" << initc << ' ' << initn << ' ' << root << endl;
	//print_egraph(g);
	//cout << "---" << endl;
	vector<vector<pair<EClassId, ENodeId> > > edges(g.eclasses.size());
	for (int i = 0; i < (int)g.eclasses.size(); ++i) {
		if (g.eclasses[i].isEffectful) {
			for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
				//cout << i << ' ' << j << ' ' << nsubregion[i][j] << endl;
				for (int k = 0; k < (int)g.eclasses[i].enodes[j].ch.size(); ++k) {
					if (g.eclasses[g.eclasses[i].enodes[j].ch[k]].isEffectful) {
						edges[g.eclasses[i].enodes[j].ch[k]].push_back(make_pair(i, j));
					}
				}
			}
		}
	}
	vector<ExVertex> vs;
	map<IndVertex, ExVertexId> vsmap;
	map<EClassId, int> wlcnt;
	vector<pair<Cost, Cost> > dis;
	vector<pair<ENodeId, ExVertexId> > pa;

	ExtractableSet inites(g.eclasses.size(), false);
	inites[initc] = true;
	inites = saturate_pure(g, inites);
	ExVertex initv = make_pair(initc, inites);
	IndVertex initvi = make_pair(initc, compress(g, inites, initc));
	vsmap[initvi] = vs.size();
	++wlcnt[initc];
	vs.push_back(initv);
	dis.push_back(make_pair(0, 1));
	pa.push_back(make_pair(initn, -1));

	priority_queue<pair<pair<Cost, Cost>, ExVertexId> > maxheap;
	maxheap.push(make_pair(make_pair(-dis[0].first, -dis[0].second), 0));

	bool nonWLwarning = false;
	int cnt = 0;
	ExVertexId goal = -1;
	while (maxheap.size() > 0) {
		pair<Cost, Cost> c = maxheap.top().first;
		c.first = -c.first;
		c.second = -c.second;
		ExVertexId i = maxheap.top().second;
		// cout << i << ' ' << c.first << ' ' << c.second << endl;
		maxheap.pop();
		if (dis[i] != c) {
			continue;
		}
		if (goal != -1 && dis[i] == dis[goal]) {
			break;
		}
		ExVertex u = vs[i];
		ExtractableSet &ues = u.second;
		++cnt;
		if (cnt % 100000 == 0) {
			cerr << "CNT : " << cnt << " HEAP : " << maxheap.size() << ' ' << " DIS : " << dis.size() << " TOTAL : " << g.eclasses.size() << endl;
			cerr << c.first << ' ' << c.second << ' ' << endl;
			for (int k = 0; k < (int)ues.size(); ++k) {
				cerr << (ues[k] ? "1" : "0");
			}
			cerr << endl;
		}
		for (int j = 0; j < (int)edges[u.first].size(); ++j) {
			EClassId vc = edges[u.first][j].first;
			ENodeId vn = edges[u.first][j].second;
			if (isExtractable(g, ues, vc, vn)) {
				ExtractableSet ves = ues;
				if (!ues[vc]) {
					ves[vc] = true;
					ves = saturate_pure(g, ves);
				}
				ExVertex v = make_pair(vc, ves);
				IndVertex vi = make_pair(vc, compress(g, ves, vc));
				pair<Cost, Cost> nc = make_pair(c.first + nsubregion[vc][vn], c.second + 1);
				int vid;
				if (!vsmap.count(vi)) {
					vid = vs.size();
					vsmap[vi] = vid;
					if (!nonWLwarning && ++wlcnt[vc] > 1) {
						cerr << "Weak Linearity Violation Found!" << endl;
						//print_egraph(g);
						nonWLwarning = true;
						//assert(false);
					}
					vs.push_back(v);
					dis.push_back(nc);
					pa.push_back(make_pair(vn, i));
					maxheap.push(make_pair(make_pair(-dis[vid].first, -dis[vid].second), vid));
				} else {
					vid = vsmap[vi];
					if (dis[vid] > nc) {
						dis[vid] = nc;
						pa[vid] = make_pair(vn, i);
						maxheap.push(make_pair(make_pair(-dis[vid].first, -dis[vid].second), vid));
					}
				}
				if (vc == root && (goal == -1 || dis[vid] < dis[goal])) {
					goal = vid;
				}
			}
		}
	}
	if (goal == -1) {
		cerr << "!!! Unextractable region !!!" << endl;
		print_egraph(g);
		cout << root << endl;
		cout << initc << endl;
	}
	assert(goal != -1);
	ExVertexId cur = goal;
	StateWalk sw;
	while (cur != -1) {
		sw.push_back(make_pair(vs[cur].first, pa[cur].first));
		cur = pa[cur].second;
	}
	return sw;
}

typedef int Rank;

inline Rank Vrank(const ExtractableSet &es, const vector<EClassId> &V, const int js = 0) {
	for (int i = js; i < (int)V.size(); ++i) {
		if (!es[V[i]]) {
			return i;
		}
	}
	return V.size();
}

typedef pair<EClassId, Rank> Vertex;

//cheating global states for the pick variable heuristics
vector<EClassId> __gfullv;

pair<bool, StateWalk> FindStateWalk(const EGraph &g, const EClassId initc, const ENodeId initn, const EClassId root, const vector<EClassId> &V) {
	vector<vector<pair<EClassId, ENodeId> > > edges(g.eclasses.size());
	for (int i = 0; i < (int)g.eclasses.size(); ++i) {
		if (g.eclasses[i].isEffectful) {
			for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
				for (int k = 0; k < (int)g.eclasses[i].enodes[j].ch.size(); ++k) {
					if (g.eclasses[g.eclasses[i].enodes[j].ch[k]].isEffectful) {
						edges[g.eclasses[i].enodes[j].ch[k]].push_back(make_pair(i, j));
					}
				}
			}
		}
	}
	// using a map here for sparsity
	map<Vertex, pair<ExtractableSet, pair<ENodeId, Vertex> > > dis;
	queue<Vertex> q;
	ExtractableSet inites(g.eclasses.size(), false);
	inites[initc] = true;
	inites = saturate_pure(g, inites);
	Rank initrnk = Vrank(inites, V);
	Vertex initv = make_pair(initc, initrnk);
	Vertex goalv = make_pair(root, V.size());
	Vertex INVALID = make_pair(-1, -1);
	dis[initv] = make_pair(inites, make_pair(initn, INVALID));
	q.push(initv);
	while (q.size() > 0 && !dis.count(goalv)) {
		Vertex u = q.front();
		q.pop();
		ExtractableSet ues = dis[u].first;
		for (int i = 0; i < (int)edges[u.first].size(); ++i) {
			EClassId vc = edges[u.first][i].first;
			ENodeId vn = edges[u.first][i].second;
			if (isExtractable(g, ues, vc, vn)) {
				ExtractableSet ves = ues;
				Rank vrnk = u.second;
				if (!ues[vc]) {
					ves[vc] = true;
					ves = saturate_pure(g, ves);
					vrnk = Vrank(ves, V, u.second);
				}
				Vertex v = make_pair(vc, vrnk);
				if (!dis.count(v)) {
					dis[v] = make_pair(ves, make_pair(vn, u));
					q.push(v);
				}
			}
		}
	}
	if (dis.count(goalv) != 0) {
		Vertex cur = goalv;
		StateWalk sw;
		while (cur != INVALID) {
			sw.push_back(make_pair(cur.first, dis[cur].second.first));
			cur = dis[cur].second.second;
		}
		return make_pair(true, sw);
	} else {
		return make_pair(false, StateWalk());
	}
}

vector<EClassId> analyzeStateWalkOrdering(const EGraph &g, const StateWalk &sw) {
	vector<bool> containsGet(g.eclasses.size(), false), vis(g.eclasses.size(), false);
	vector<vector<pair<EClassId, ENodeId> > > edges(g.eclasses.size());
	vector<vector<int> > counters(g.eclasses.size());
	queue<EClassId> q;
	for (int i = 0; i < (int)g.eclasses.size(); ++i) {
		if (!g.eclasses[i].isEffectful) {
			counters[i].resize(g.eclasses[i].enodes.size());
			for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
				if (vis[i]) {
					break;
				}
				counters[i][j] = g.eclasses[i].enodes[j].ch.size();
				for (int k = 0; k < (int)g.eclasses[i].enodes[j].ch.size(); ++k) {
					edges[g.eclasses[i].enodes[j].ch[k]].push_back(make_pair(i, j));
					if (g.eclasses[g.eclasses[i].enodes[j].ch[k]].isEffectful) {
						containsGet[i] = true;
					}
				}
				if (counters[i][j] == 0 && !vis[i]) {
					q.push(i);
					vis[i] = true;
				}
			}
		}
	}
	while (q.size()) {
		EClassId u = q.front();
		q.pop();
		for (int i = 0; i < (int)edges[u].size(); ++i) {
			EClassId vc = edges[u][i].first;
			ENodeId vn = edges[u][i].second;
			--counters[vc][vn];
			if (counters[vc][vn] == 0 && !vis[vc]) {
				q.push(vc);
				vis[vc] = true;
			}
		}
	}
	vector<EClassId> ret;
	for (int i = (int)sw.size() - 1; i >= 0; --i) {
		if (!vis[sw[i].first]) {
			q.push(sw[i].first);
			vis[sw[i].first] = true;
			while (q.size()) {
				EClassId u = q.front();
				if (i < (int)sw.size() - 1  && containsGet[u]) {
					ret.push_back(u);	
				}
				q.pop();
				for (int i = 0; i < (int)edges[u].size(); ++i) {
					EClassId vc = edges[u][i].first;
					ENodeId vn = edges[u][i].second;
					--counters[vc][vn];
					if (counters[vc][vn] == 0 && !vis[vc]) {
						q.push(vc);
						vis[vc] = true;
					}
				}
			}
		}
	}
	return ret;
}

pair<EClassId, ENodeId> findArg(const EGraph &g) {
	pair<EClassId, ENodeId> ret = make_pair(-1, -1);
	int narg = 0;
	for (int i = 0; i < (int)g.eclasses.size(); ++i) {
		if (g.eclasses[i].isEffectful) {
			for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
				if (g.eclasses[i].enodes[j].ch.size() == 0) {
					if (++narg == 1) {
						ret = make_pair(i, j);
					}
					break;
				}
			}
		}
	}
	if (narg == 0) {
		cerr << "Error: Failed to find arg!" << endl;
		print_egraph(g);
		assert(false);
	} else if (narg > 1) {
		cerr << "Warning: Found mulitple arg in different eclasses!!" << endl;
	}
	return ret;
}

EClassId pick_next_variable_heuristics(const vector<EClassId> &v) {
	// This is currently the stupiest thing
	while (true) {
		int i = rand() % (int)__gfullv.size();
		if (find(v.begin(), v.end(), __gfullv[i]) == v.end()) {
			return __gfullv[i];
		}
	}
}


typedef int RegionId;

// the main function for getting a linear extraction from a region
// this uses ILP in ilp mode or the unguided statewalk search in the normal mode
Extraction extractRegion(const EGraph &g, const EClassId initc, const ENodeId initn, const EClassId root, const vector<vector<int> > &nsubregion) {
	StateWalk sw = UnguidedFindStateWalk(g, initc, initn, root, nsubregion);
	return regionExtractionWithStateWalk(g, root, sw).second;
}

ExtractionENodeId reconstructExtraction(const EGraph &g, const vector<EClassId> &region_roots, const vector<RegionId> &region_root_id, vector<ExtractionENodeId> &extracted_roots, Extraction &e, const RegionId &cur_region) {
	if (extracted_roots[cur_region] != -1) {
		return extracted_roots[cur_region];
	}
	EClassId region_root = region_roots[cur_region];
	//cout << cur_region << " Region root : " << region_root << endl;
	pair<EGraph, SubEGraphMap> res = createRegionEGraph(g, region_root);
	EGraph &gr = res.first;
	SubEGraphMap &rmap = res.second;
	cerr << "Region root : " << region_root << "  Region egraph size : " << g.eclasses.size() << " Top region egraph size : " << gr.eclasses.size() << endl;
	pair<EClassId, ENodeId> arg = findArg(gr);
	EClassId argc = arg.first;
	ENodeId argn = arg.second;
	EClassId root = rmap.inv[region_root];
	Extraction er = extractRegion(gr, argc, argn, root, rmap.nsubregion);
	Extraction ner(er.size());
	for (int i = 0; i < (int)er.size(); ++i) {
		ExtractionENode &en = er[i], &nen = ner[i];
		EClassId oric = rmap.eclassmp[en.c];
		nen.c = oric;
		ENodeId orin = en.n;
		nen.n = orin;
		bool subregionchild = false;
		for (int j = 0, k = 0; j < (int)g.eclasses[oric].enodes[orin].ch.size(); ++j) {
			EClassId orichc = g.eclasses[oric].enodes[orin].ch[j];
			if (g.eclasses[orichc].isEffectful) {
				if (subregionchild) {
					nen.ch.push_back(reconstructExtraction(g, region_roots, region_root_id, extracted_roots, e, region_root_id[orichc]));
				} else {
					subregionchild = true;
					nen.ch.push_back(en.ch[k++]);
				}
			} else {
				nen.ch.push_back(en.ch[k++]);
			}
		}
	}
	int delta = e.size();
	for (int i = 0; i < (int)ner.size(); ++i) {
		bool subregionchild = false;
		for (int j = 0; j < (int)g.eclasses[ner[i].c].enodes[ner[i].n].ch.size(); ++j) {
			EClassId chc = g.eclasses[ner[i].c].enodes[ner[i].n].ch[j];
			if (g.eclasses[chc].isEffectful) {
				if (subregionchild) {
					continue;
				} else {
					subregionchild = true;
					ner[i].ch[j] += delta;
				}
			} else {
				ner[i].ch[j] += delta;
			}
		}
	}
	e.insert(e.end(), ner.begin(), ner.end());
	extracted_roots[cur_region] = e.size() - 1;
	return extracted_roots[cur_region];
}

void print_egg_init() {
	const char* schema = 
R"(
(datatype Expr)

(sort TypeList)

(datatype BaseType
  (IntT)
  (BoolT)
  (FloatT)
  (PointerT BaseType)
  (StateT)
)

(datatype Type
  (Base BaseType)
  (TupleT TypeList)
)

(constructor TNil () TypeList)
(constructor TCons (BaseType TypeList) TypeList)

(let DumT (TupleT (TNil)))

(datatype Assumption    
  (DumC)
)

(constructor Arg (Type Assumption) Expr)

(datatype Constant
  (Int i64)
  (Bool bool)
  (Float f64)
)

(constructor Empty (Type Assumption) Expr)

(constructor Const (Constant Type Assumption) Expr)

(datatype TernaryOp
  (Write)
  (Select)
)

(datatype BinaryOp
  (Bitand)
  (Add)
  (Sub)
  (Div)
  (Mul)
  (LessThan)
  (GreaterThan)
  (LessEq)
  (GreaterEq)
  (Eq)
  (Smin)
  (Smax)
  (Shl)
  (Shr)
  (FAdd)
  (FSub)
  (FDiv)
  (FMul)
  (FLessThan)
  (FGreaterThan) 
  (FLessEq)
  (FGreaterEq)
  (FEq)
  (Fmin)
  (Fmax)
  (And)
  (Or)
  (Load)
  (PtrAdd)
  (Print)
  (Free)
)

(datatype UnaryOp
  (Neg)
  (Abs)
  (Not)
)

(constructor Top   (TernaryOp Expr Expr Expr) Expr)
(constructor Bop   (BinaryOp Expr Expr) Expr)
(constructor Uop   (UnaryOp Expr) Expr)

(constructor Get   (Expr i64) Expr)
(constructor Alloc (i64 Expr Expr BaseType) Expr)
(constructor Call  (String Expr) Expr)

(constructor Single (Expr) Expr)
(constructor Concat (Expr Expr) Expr)

(constructor If (Expr Expr Expr Expr) Expr)

(constructor DoWhile (Expr Expr) Expr)

(constructor Function (String Type Type Expr) Expr)

(ruleset reconstruction)
)";
	printf("%s", schema);
}

void print_egg_extraction(const EGraph &g, const Extraction &e) {
	static int funid = 0, cnt = 0;
	printf("; Function #%d\n", ++funid);
	printf("(rule () (\n");
	vector<string> var(e.size());
	for (int i = 0; i < (int)e.size(); ++i) {
		string head = g.eclasses[e[i].c].enodes[e[i].n].head;
		string name, op;
		int pos = head.find("###");
		name = head.substr(0, pos);
		op = head.substr(pos + 3, head.length() - pos - 3);
		if (name.length() > 9 && name.substr(0, 9) == "primitive") {
			if (op[0] == '\\') {
				var[i] = op.substr(1, op.length() - 3) + "\"";
			} else {
				var[i] = op;
			}
		} else {
			string curvar = string("__tmp") + to_string(cnt++);
			var[i] = curvar;
			printf("\t(let %s (", curvar.c_str());
			if (op == "Arg") {
				assert(e[i].ch.size() == 0);
				printf("Arg DumT (DumC)");
			} else if (op == "Const") {
				assert(e[i].ch.size() == 1);
				printf("Const %s DumT (DumC)", var[e[i].ch[0]].c_str());
			} else if (op == "Empty") {
				assert(e[i].ch.size() == 0);
				printf("Empty DumT (DumC)");
			} else {
				printf("%s", op.c_str());
				for (int j = 0; j < (int)e[i].ch.size(); ++j) {
					printf(" %s", var[e[i].ch[j]].c_str());
				}
			}
			printf("))\n");
		}
	}
	printf(") :ruleset reconstruction)\n");
}

void print_egg_end() {
	printf("(run reconstruction 1)\n");
}

int main(int argc, char** argv) {
	for (int i = 1; i < argc; ++i) {
		string arg = argv[i];
		if (arg == "--ilp-mode") {
			g_ilp_mode = true;
			continue;
		}
		cerr << "Unknown argument: " << arg << endl;
		return 1;
	}
	EGraph g;
	vector<EClassId> fun_roots;
	if (DEBUG) {
		FILE* ppin = preprocessing();
		g = read_egraph(ppin);
		EClassId fun_root;
		while (fscanf(ppin, "%d", &fun_root) != -1) {
			fun_roots.push_back(fun_root);
		}
	} else {
		g = read_egraph(stdin);
		EClassId fun_root;
		while (scanf("%d", &fun_root) != -1) {
			fun_roots.push_back(fun_root);
		}	
	}
	//print_egraph(g);	
	print_egg_init();
	for (int _ = 0; _ < (int)fun_roots.size(); ++_) {
		EClassId fun_root = fun_roots[_];
		if (!g.eclasses[fun_root].isEffectful) {
			//cerr << "Skipping pure function : " << fun_root << endl;
			//TODO extract pure function
			continue;
		}
		cerr << "Function root : " << fun_root << endl;
		vector<RegionId> region_root_id(g.eclasses.size(), -1);
		vector<EClassId> region_roots;
		region_roots.push_back(fun_root);
		region_root_id[fun_root] = 0;
		for (int i = 0; i < (int)g.eclasses.size(); ++i) {
			if (g.eclasses[i].isEffectful) {
				for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
					bool subregionroot = false;
					for (int k = 0; k < (int)g.eclasses[i].enodes[j].ch.size(); ++k) {
						EClassId v = g.eclasses[i].enodes[j].ch[k];
						if (g.eclasses[v].isEffectful) {
							if (subregionroot) {
								if (region_root_id[v] == -1) {
									region_root_id[v] = region_roots.size();
									region_roots.push_back(v);
								}
							} else {
								subregionroot = true;
							}
						}
					}
				}
			}
		}
		vector<ExtractionENodeId> extracted_roots(region_roots.size(), -1);
		Extraction e;
		reconstructExtraction(g, region_roots, region_root_id, extracted_roots, e, region_root_id[fun_root]);
		//print_extraction(g, e);
		print_egg_extraction(g, e);
		assert(linearExtraction(g, fun_root, e));
	}
	print_egg_end();
	/*
	vector<pair<EGraph, SubEGraphMap>> region_egraphs;
	for (int i = 0; i < (int)region_roots.size(); ++i) {
		int u = region_roots[i];
		//cout << "Region root : " << u << endl;
		region_egraphs.push_back(createRegionEGraph(g, u));
		//print_egraph(region_egraphs.back().first);
	}
	vector<Extraction> region_extractions;
	for (int i = 0; i < (int)region_egraphs.size(); ++i) {
		cout << "Region root : " << region_roots[i] << endl;
		EGraph &gr = region_egraphs[i].first;
		pair<EClassId, ENodeId> arg = findArg(gr);
		EClassId argc = arg.first;
		ENodeId argn = arg.second;
		EClassId root = region_egraphs[i].second.inv[region_roots[i]];
		StateWalk sw = UnguidedFindStateWalk(gr, argc, argn, root);
		region_extractions.push_back(regionExtractionWithStateWalk(gr, root, sw).second);
		print_extraction(gr, region_extractions.back());
	}
	*/
/*
	Extraction e1 = NormalGreedyExtraction(g, root).second;
	print_extraction(g, e1);
	if (!linearExtraction(g, root, e1)) {
		printf("This extraction is not linear!\n");
		printf("Try two-pass extraction!\n");
		StateWalk sp = getStateWalk(g, root, e1);
		pair<bool, Extraction> result = ExtractionWithStateWalk(g, root, sp);
		if (result.first) {
			printf("Two-pass extraction succeeded!\n");
			print_extraction(g, result.second);
		} else {
			printf("Two-pass extraction failed!\n");
			printf("Reading the state walk of the original program.\n");
			int l;
			fscanf(ppin, "%d", &l);
			StateWalk ori_sw(l);
			for (int i = 0; i < l; ++i) {
				fscanf(ppin, "%d%d", &ori_sw[i].first, &ori_sw[i].second);
			}
			pair<bool, Extraction> result = ExtractionWithStateWalk(g, root, ori_sw);
			if (result.first) {
				printf("Successfully extracted with the original program's state walk [as an optional test]!\n");
				Extraction e_ori = result.second;
				print_extraction(g, e_ori);
				printf("Analyzing the original program's state walk.\n");
				vector<EClassId> fullv = analyzeStateWalkOrdering(g, ori_sw);
				__gfullv = fullv;
				printf("Found the following variable ordering:\n");
				for (int i = 0; i < (int)fullv.size(); ++i) {
					printf("%d%c", fullv[i], i == (int)fullv.size() - 1 ? '\n' : ' ');
				}
				vector<int> fullvrnk(g.eclasses.size(), -1);
				for (int i = 0; i < (int)fullv.size(); ++i) {
					fullvrnk[fullv[i]] = i;
				}
				vector<EClassId> v;
				StateWalk sw;
				pair<EClassId, ENodeId> arg = findArg(g);
				EClassId argc = arg.first;
				ENodeId argn = arg.second;	
				assert(arg == ori_sw.back());
				if (FindStateWalk(g, argc, argn, root, fullv).first) {
					printf("Successfully found a state walk with the original program variable ordering [as an optional test]!\n");
				} else {
					printf("Failed to find a statewalk with the original program's variable ordering!\n");
					printf("Something is very wrong!\n");
					assert(false);
				}
				bool succ = false;
				while (!succ) {
					pair<bool, StateWalk> result = FindStateWalk(g, argc, argn, root, v);
					if (result.first) {
						printf("Found statewalk:\n");
						sw = result.second;
						for (int i = 0; i < (int)sw.size(); ++i) {
							printf("%d %d\n", sw[i].first, sw[i].second);
						}
						break;
					} else {
						printf("Failed to find statewalk with the given eclass set (Size = %zu)!\n", v.size());
						v.push_back(pick_next_variable_heuristics(v));
						vector<int> vrnks;
						for (int i = 0; i < (int)v.size(); ++i) {
							vrnks.push_back(fullvrnk[v[i]]);
						}
						sort(vrnks.begin(), vrnks.end());
						v.clear();
						for (int i = 0; i < (int)vrnks.size(); ++i) {
							if (vrnks[i] != -1) {
								v.push_back(fullv[vrnks[i]]);
							}
						}
						printf("New eclass set v: ");
						for (int i = 0; i < (int)v.size(); ++i) {
							printf("%d%c", v[i], i == (int)v.size() - 1 ? '\n' : ' ');
						}
					}
				}
				pair<bool, Extraction> result = ExtractionWithStateWalk(g, root, sw);
				if (result.first) {
					print_extraction(g, result.second);
				} else {
					printf("Failed to extract along the statewalk!\n");
					assert(false);
				}

			} else {
				printf("Failed to extract with the original program's state walk!\n");
				printf("Something is very wrong!\n");
				assert(false);
			}
		}
	}
*/
	return 0;
}