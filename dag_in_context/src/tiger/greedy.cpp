#include "greedy.h"

#include<queue>
#include<iostream>
#include<unordered_map>

#include "debug.h"

using namespace std;

// bag-based greedy cost
// it is unstable because it does not guarantee the lowest cost
// but that is ok for getting an estimate

struct SCost {
    Cost sum;
    unordered_map<EClassId, Cost> bag;

    SCost(Cost v) {
        sum = v;
    }
};

SCost& operator += (SCost &a, const pair<EClassId, SCost> &cb) {
    Cost overhead = cb.second.sum;
    for (unordered_map<EClassId, Cost>::const_iterator it = cb.second.bag.begin(); it != cb.second.bag.end(); ++it) {
        EClassId cid = it->first;
        Cost c = it->second;
        overhead -= c;
        if (a.bag.count(cid)) {
            if (a.bag[cid] > c) {
                a.sum -= (a.bag[cid] - c);
                a.bag[cid] = c;
            }
        } else {
            a.bag.insert(*it);
            a.sum += c;
        }
    }
    if (a.bag.count(cb.first)) {
        if (a.bag[cb.first] > overhead) {
            a.sum -= (a.bag[cb.first] - overhead);
            a.bag[cb.first] = overhead;
        }
    } else {
        a.bag[cb.first] = overhead;
        a.sum += overhead;
    }
    return a;
}

bool operator < (const SCost &a, const SCost &b) {
    return a.sum < b.sum;
}

bool isPrimitive(const string name) {
    return name.length() > 9 && name.substr(0, 9) == "primitive";
}

bool isType(const string op) {
    return op == "IntT" || op == "BoolT" || op == "FloatT" ||
		   op == "PointerT" || op == "StateT" || op == "Base" ||
		   op == "TupleT" || op == "TNil" || op == "TCons";
}

Cost get_enode_cost(const ENode &n) {
    string name = n.get_name();
    string op = n.get_op();
    if (op == "Const") {
        return 10;
    } else if (op == "Arg" || isPrimitive(name) || isType(op) || op == "Int" || op == "Bool" || op == "Float") {
        return 0;
    } else if (op == "Empty" || op == "Single" || op == "Concat" || op == "Nil" || op == "Cons") {
        return 0;
    } else if (op == "Get") {
        return 1;
    } else if (op == "Abs" || op == "Bitand" || op == "Neg" || op == "Add" || op == "PtrAdd" || op == "Sub" || op == "And" || op == "Or" || op == "Not" || op == "Shl" || op == "Shr") {
        return 100;
    } else if (op == "FAdd" || op == "FSub" || op == "Fmax" || op == "Fmin") {
        return 500;
    } else if (op == "Mul") {
        return 300;
    } else if (op == "FMul") {
        return 1500;
    } else if (op == "Div") {
        return 500;
    } else if (op == "FDiv") {
        return 2500;
    } else if (op == "Eq" || op ==  "LessThan" || op == "GreaterThan" || op == "LessEq" || op == "GreaterEq") {
        return 100;
    } else if (op == "Select") {
        return 1500;
    } else if (op == "Smax" || op == "Smin" || op == "FEq") {
        return 100;
    } else if (op == "FLessThan" || op == "FGreaterThan" || op == "FLessEq" || op == "FGreaterEq") {
        return 1000;
    } else if (op == "Print" || op == "Write" || op == "Load") {
        return 500;
    } else if (op == "Alloc" || op == "Free") {
        return 1000;
    } else if (op == "Call") {
        return 500000;
    } else if (op == "Program" || op == "Function") {
        return 0;
    // special treatment for loops and ifs somewhere else
    } else if (op == "DoWhile") {
        return 1;
    } else if (op == "If" || op == "Switch") {
        return 250;
    } else if (op == "Uop" || op == "Bop" || op == "Top") {
        return 0;
    } else {
        DEBUG_CERR("Encountered op of unknown cost: " << name << ' ' << op);
        DEBUG_ASSERT(false);
        return 0;
    }
}

// extract all eclasses if root is -1
pair<vector<ENodeId>, vector<Cost> > greedy_extract_compute_eclasses_pick(const EGraph &g, const EClassId root = -1) {
    vector<ENodeId> pick(g.neclasses(), -1);
    vector<SCost> dis(g.neclasses(), SCost(INF));
    priority_queue<pair<Cost, EClassId> > maxheap;
    // Optimization
    vector<vector<pair<EClassId, ENodeId> > > parents = compute_reverse_index(g);
    vector<vector<int> > cnt(g.neclasses());
    // Initialize nodes
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        cnt[i].resize(c.nenodes());
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            cnt[i][j] = n.ch.size();
            if (cnt[i][j] == 0) {
                SCost ndis = SCost(get_enode_cost(n));
                if (ndis < dis[i]) {
                    dis[i] = ndis;
                    pick[i] = j;
                    maxheap.push(make_pair(~dis[i].sum, i));
                }
            }
        }
    }
    while (maxheap.size() > 0) {
        Cost d = ~maxheap.top().first;
        EClassId i = maxheap.top().second;
        if (i == root) {
            break;
        }
        maxheap.pop();
        if (d == dis[i].sum) {
            for (size_t j = 0; j < parents[i].size(); ++j) {
                EClassId pc = parents[i][j].first;
                ENodeId pn = parents[i][j].second;
                if ((--cnt[pc][pn]) == 0) {
                    const ENode &n = g.eclasses[pc].enodes[pn];
                    SCost ndis(0);
                    if (n.get_op() == "If") {
                        DEBUG_ASSERT(n.ch.size() == 4);
                        ndis = SCost(get_enode_cost(n));
                        ndis += make_pair(n.ch[0], dis[n.ch[0]]);
                        ndis += make_pair(n.ch[1], dis[n.ch[1]]);
                        Cost then_cost = dis[n.ch[2]].sum, else_cost = dis[n.ch[3]].sum;
                        // Heuristics for computing cost of an If
                        ndis.sum += max(then_cost, else_cost) + (min(then_cost, else_cost) >> 2);
                    } else if (n.get_op() == "DoWhile") {
                        DEBUG_ASSERT(n.ch.size() == 2);
                        ndis = dis[n.ch[0]];
                        Cost body_cost = dis[n.ch[1]].sum;
                        // Heuristics for computing cost of a loop
                        // Can be improved further by plumbing the information from the iter analysis
                        // This can potentially overflow due to deeply nested loops
                        ndis.sum += body_cost * 500;
                    } else {
                        // Otherwise, just the bag cost
                        ndis = SCost(get_enode_cost(n));
                        for (size_t k = 0; k < n.ch.size(); ++k) {
                            ndis += make_pair(n.ch[k], dis[n.ch[k]]);
                        }
                    }
                    if (ndis < dis[pc]) {
                        dis[pc] = ndis;
                        pick[pc] = pn;
                        maxheap.push(make_pair(~dis[pc].sum, pc));
                    }
                }
            }
        }
    }
    vector<Cost> dis_sum(g.neclasses());
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        dis_sum[i] = dis[i].sum;
    }
    return make_pair(pick, dis_sum);
}

vector<Cost> greedy_extract_estimate_all_eclasses_cost(const EGraph &g) {
    return greedy_extract_compute_eclasses_pick(g).second;
}


Cost get_statewalk_enode_cost(const EGraph &g, const vector<Cost> &eclass_cost, const ENode &n) {
    Cost ret = 0;
    if (n.get_op() == "If") {
        DEBUG_ASSERT(n.ch.size() == 4);
        ret = get_enode_cost(n);
        if (!g.eclasses[n.ch[0]].isEffectful) {
            ret += eclass_cost[n.ch[0]];
        }
        if (!g.eclasses[n.ch[1]].isEffectful) {
            ret += eclass_cost[n.ch[1]];
        }
        Cost then_cost = eclass_cost[n.ch[2]], else_cost = eclass_cost[n.ch[3]];
        ret += max(then_cost, else_cost) + (min(then_cost, else_cost) >> 2);
    } else if (n.get_op() == "DoWhile") {
        DEBUG_ASSERT(n.ch.size() == 2);
        Cost body_cost = eclass_cost[n.ch[1]];
        ret = body_cost * 500;
    } else {
        ret = get_enode_cost(n);
        for (size_t k = 0; k < n.ch.size(); ++k) {
            EClassId cid = n.ch[k];
            if (!g.eclasses[cid].isEffectful) {
                ret += eclass_cost[cid];
            }
        }
    }
    return ret;
}

vector<vector<Cost> > compute_statewalk_cost(const EGraph &g) {
    vector<Cost> eclass_cost = greedy_extract_estimate_all_eclasses_cost(g);

    vector<vector<Cost> > statewalk_cost(g.neclasses());
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        if (c.isEffectful) {
            statewalk_cost[i].resize(c.nenodes());
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                const ENode &n = c.enodes[j];
                statewalk_cost[i][j] = get_statewalk_enode_cost(g, eclass_cost, n);
            }
        }
    }
    return statewalk_cost;
}


vector<vector<Cost> > project_statewalk_cost(const EGraphMapping &gr2g, const vector<vector<Cost> > &statewalk_cost) {
    vector<vector<Cost> > rstatewalk_cost(gr2g.eclassidmp.size());
    for (EClassId i = 0; i < (EClassId)gr2g.eclassidmp.size(); ++i) {
        const EClassId cid = gr2g.eclassidmp[i];
        if (statewalk_cost[cid].size() > 0) {
            rstatewalk_cost[i].resize(gr2g.enodeidmp[i].size());
            for (ENodeId j = 0; j < (ENodeId)rstatewalk_cost[i].size(); ++j) {
                rstatewalk_cost[i][j] = statewalk_cost[gr2g.eclassidmp[i]][gr2g.enodeidmp[i][j]];
            }
        }
    }
    return rstatewalk_cost;
}

Extraction statewalk_greedy_extraction(const EGraph &g, const EClassId root) {
    vector<ENodeId> pick(g.neclasses(), -1);
    vector<SCost> dis(g.neclasses(), SCost(INF));
    priority_queue<pair<Cost, EClassId> > maxheap;
    // Optimization
    vector<vector<pair<EClassId, ENodeId> > > parents = compute_reverse_index(g);
    vector<vector<int> > cnt(g.neclasses());
    // Initialize nodes
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        cnt[i].resize(c.nenodes());
        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
            const ENode &n = c.enodes[j];
            cnt[i][j] = n.ch.size();
            if (cnt[i][j] == 0) {
                SCost ndis = SCost(get_enode_cost(n));
                if (ndis < dis[i]) {
                    dis[i] = ndis;
                    pick[i] = j;
                    maxheap.push(make_pair(~dis[i].sum, i));
                }
            }
        }
    }
    Extraction e;
    vector<ExtractionENodeId> extracted(g.neclasses(), UNEXTRACTABLE_ECLASS);
    vector<int> buf_id(g.neclasses(), -1);
    vector<bool> processed(g.neclasses(), false);
    while (maxheap.size() > 0) {
        Cost d = ~maxheap.top().first;
        EClassId i = maxheap.top().second;
        maxheap.pop();
        if (d != dis[i].sum) {
            continue;
        }
        if (g.eclasses[i].isEffectful) {
            // grow the extraction
            vector<EClassId> buf;
            vector<vector<int> > edges;
            vector<int> bcnt;
            buf_id[i] = 0;
            buf.push_back(i);
            edges.push_back(vector<int>());
            bcnt.push_back(0);
            for (size_t i = 0; i < buf.size(); ++i) {
                EClassId u = buf[i];
                const EClass &c = g.eclasses[u];
                const ENode &n = c.enodes[pick[u]];
                for (size_t k = 0; k < n.ch.size(); ++k) {
                    EClassId v = n.ch[k];
                    if (extracted[v] == UNEXTRACTABLE_ECLASS) {
                        if (buf_id[v] == -1) {
                            buf_id[v] = buf.size();
                            buf.push_back(v);
                            edges.push_back(vector<int>());
                            bcnt.push_back(0);
                        }
                        edges[buf_id[v]].push_back(i);
                        bcnt[i]++;
                    }
                }
            }
            vector<int> q;
            for (size_t i = 0; i < buf.size(); ++i) {
                if (bcnt[i] == 0) {
                    q.push_back(i);
                }
            }
            for (size_t i = 0; i < q.size(); ++i) {
                int u = q[i];
                for (size_t j = 0; j < edges[u].size(); ++j) {
                    int v = edges[u][j];
                    if ((--bcnt[v]) == 0) {
                        q.push_back(v);
                    }
                }
            }
            DEBUG_ASSERT(q.size() == buf.size());
            int base = e.size();
            e.resize(e.size() + q.size());
            for (size_t i = 0; i < q.size(); ++i) {
                int u = buf[q[i]];
                extracted[u] = base + i;
                ExtractionENode &en = e[extracted[u]];
                en.c = u;
                en.n = pick[u];
            }
            for (size_t i = 0; i < q.size(); ++i) {
                int u = buf[q[i]];
                const ENode &n = g.eclasses[u].enodes[pick[u]];
                ExtractionENode &en = e[extracted[u]];
                en.ch.resize(n.ch.size());
                for (size_t j = 0; j < n.ch.size(); ++j) {
                    en.ch[j] = extracted[n.ch[j]];
                }
            }
            // processed optimization
            for (size_t j = 0; j < q.size(); ++j) {
                int u = buf[q[j]];
                if (dis[u].sum > 0) {
                    dis[u].sum = 0;
                    dis[u].bag.clear();
                    if (u != i) {
                        maxheap.push(make_pair(~0, u));
                    }
                }
            }
        }
        if (i == root) {
            break;
        }
        for (size_t j = 0; j < parents[i].size(); ++j) {
            EClassId pc = parents[i][j].first;
            ENodeId pn = parents[i][j].second;
            if ((processed[i] && cnt[pc][pn] == 0) || (!processed[i] && (--cnt[pc][pn] == 0))) {
                const ENode &n = g.eclasses[pc].enodes[pn];
                SCost ndis(0);
                if (!g.eclasses[pc].isEffectful) {
                    ndis = SCost(get_enode_cost(n));
                }
                for (size_t k = 0; k < n.ch.size(); ++k) {
                    ndis += make_pair(n.ch[k], dis[n.ch[k]]);
                }
                if (ndis < dis[pc]) {
                    dis[pc] = ndis;
                    pick[pc] = pn;
                    maxheap.push(make_pair(~dis[pc].sum, pc));
                }
            }
        }
        processed[i] = true;
    }
    DEBUG_ASSERT(extracted[root] != UNEXTRACTABLE_ECLASS);
    DEBUG_ASSERT(is_effect_safe_extraction(g, root, e));
    return e;
}