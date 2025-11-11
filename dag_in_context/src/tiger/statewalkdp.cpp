#include<random>
#include<queue>
#include<unordered_map>

#include "statewalkdp.h"
#include "persistent_btree.h"

#include "debug.h"

PersistentDecArray enode_cnt_pool;

PersistentBitSet true_extractable_pool;

using HashType = unsigned long long;

using DPId = int;

struct DPValue {
    Cost c;
    PBId root;
    DPId prev;
    EClassId ec;
    ENodeId pick;

    DPValue(Cost c, PBId root, DPId prev, EClassId ec, ENodeId pick) : c(c), root(root), prev(prev), ec(ec), pick(pick) {}
};

struct BitsetExtraInfo {
    HashType true_hash;
    HashType masked_hash;
    PBId array;

    BitsetExtraInfo(HashType true_hash, HashType masked_hash, PBId array) : true_hash(true_hash), masked_hash(masked_hash), array(array) {}
};

Statewalk statewalkDP(const EGraph &g, const EClassId root, const vector<vector<Cost> > &statewalk_cost, const bool use_liveness, const bool use_satellite_opt, StatewalkWidthStat *const stat) {
    // find arg
    DEBUG_ASSERT(arg_check_regionalized_egraph(g));
    EClassId argc = UNEXTRACTABLE_ECLASS;
    ENodeId argn = UNEXTRACTABLE_ECLASS;
    for (EClassId i = 0; i < (EClassId)g.neclasses() && argc == UNEXTRACTABLE_ECLASS; ++i) {
        const EClass &c = g.eclasses[i];
        if (c.isEffectful) {
            for (ENodeId j = 0; j < (ENodeId)c.nenodes() && argc == UNEXTRACTABLE_ECLASS; ++j) {
                const ENode &n = c.enodes[j];
                if (n.ch.size() == 0) {
                    argc = i;
                    argn = j;
                }
            }
        }
    }

    Cost init_cost = statewalk_cost[argc][argn];

    // prepare for heavy duty data structures
    vector<vector<pair<EClassId, ENodeId> > > parent_edge_to_pure(g.neclasses()), parent_edge_to_effectful(g.neclasses());
    vector<vector<unsigned> > enode_cnt(g.neclasses());

    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        if (c.isEffectful) {
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                const ENode &n = c.enodes[j];
                for (size_t k = 0; k < n.ch.size(); ++k) {
                    if (g.eclasses[n.ch[k]].isEffectful) {
                        parent_edge_to_effectful[n.ch[k]].push_back(make_pair(i, j));
                    }
                }
            }
        } else {
            enode_cnt[i].resize(c.nenodes());
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                const ENode &n = c.enodes[j];
                enode_cnt[i][j] = n.ch.size();
                for (size_t k = 0; k < n.ch.size(); ++k) {
                    parent_edge_to_pure[n.ch[k]].push_back(make_pair(i, j));
                }
            }
        }
    }

    queue<EClassId> q;
    vector<bool> init_extractable(g.neclasses(), false);

    init_extractable[argc] = true;
    q.push(argc);
    
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        const EClass &c = g.eclasses[i];
        if (!c.isEffectful) {
            bool extractable = false;
            for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                const ENode &n = c.enodes[j];
                if (n.ch.size() == 0) {
                    extractable = true;
                    break;
                }
            }
            if (extractable) {
                init_extractable[i] = true;
                q.push(i);
            }
        }
    }

    while(q.size()) {
        EClassId u = q.front();
        q.pop();
        for (size_t i = 0; i < parent_edge_to_pure[u].size(); ++i) {
            EClassId vc = parent_edge_to_pure[u][i].first;
            ENodeId vn = parent_edge_to_pure[u][i].second;
            if ((--enode_cnt[vc][vn]) == 0) {
                if (!init_extractable[vc]) {
                    init_extractable[vc] = true;
                    q.push(vc);
                }
            }
        }
    }

    // flatten egraphin format
    vector<int> rnk(g.neclasses(), 0);
    vector<unsigned> init_cnt;
    vector<int> compressed_eclass_id(g.neclasses(), -1);
    // be very careful that the compressed eclasses do not include the arg eclass
    vector<EClassId> inv_compressed_eclass_id;
    for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
        if (init_extractable[i]) {
            enode_cnt[i].clear();
            continue;
        }
        compressed_eclass_id[i] = inv_compressed_eclass_id.size();
        inv_compressed_eclass_id.push_back(i);
        if (enode_cnt[i].size() > 0) {
            rnk[i] = init_cnt.size();
            init_cnt.insert(init_cnt.end(), enode_cnt[i].begin(), enode_cnt[i].end());
        }
    }

    // crazy optimization to use only 2 bits for all in-degree counters
    for (size_t i = 0; i < init_cnt.size(); ++i) {
        DEBUG_ASSERT(init_cnt[i] > 0);
        --init_cnt[i];
        DEBUG_ASSERT(init_cnt[i] <= 3);
    }
    size_t ncompressed_eclass = inv_compressed_eclass_id.size();

    PBId init_cnt_id = enode_cnt_pool.init(init_cnt);
    PBId init_extractable_id = true_extractable_pool.init(vector<unsigned>(ncompressed_eclass, 0));
 
    // crazy hash scheme
    vector<HashType> base_vectors(ncompressed_eclass);
    mt19937_64 rand_gen;
    for (size_t i = 0; i < base_vectors.size(); ++i) {
        base_vectors[i] = rand_gen();
    }
    HashType init_true_hash = 0, init_masked_hash = 0;

    // liveness

    vector<vector<unsigned long long> > liveness(g.neclasses());
    vector<unordered_map<EClassId, vector<int> > > liveness_delta(g.neclasses());

    if (use_liveness) {
        for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
            const EClass &c = g.eclasses[i];
            if (c.isEffectful) {
                liveness[i].resize((g.neclasses() + 63) >> 6, 0);
                queue<EClassId> q;
                q.push(i);
                while (q.size()) {
                    EClassId u = q.front();
                    q.pop();
                    if (g.eclasses[u].isEffectful && u != root) {
                        for (size_t j = 0; j < parent_edge_to_effectful[u].size(); ++j) {
                            EClassId v = parent_edge_to_effectful[u][j].first;
                            if (!((liveness[i][v >> 6] >> (v & 63)) & 1)) {
                                liveness[i][v >> 6] |= 1ull << (v & 63);
                                q.push(v);
                            }
                        }
                    }
                    if ((liveness[i][u >> 6] >> (u & 63)) & 1) {
                        const EClass &c = g.eclasses[u];
                        for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                            const ENode &n = c.enodes[j];
                            for (size_t k = 0; k < n.ch.size(); ++k) {
                                EClassId v = n.ch[k];
                                if (!init_extractable[v] && !g.eclasses[v].isEffectful && !((liveness[i][v >> 6] >> (v & 63)) & 1)) {
                                    liveness[i][v >> 6] |= 1ull << (v & 63);
                                    q.push(v);
                                }
                            }
                        }
                    }
                }
            }
        }

        for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
            const EClass &c = g.eclasses[i];
            if (c.isEffectful && i != root) {
                for (size_t j = 0; j < parent_edge_to_effectful[i].size(); ++j) {
                    EClassId v = parent_edge_to_effectful[i][j].first;
                    if (!liveness_delta[i].count(v)) {
                        vector<EClassId> &l = liveness_delta[i][v];
                        for (size_t k = 0; k < liveness[i].size(); ++k) {
                            DEBUG_ASSERT((liveness[i][k] & liveness[v][k]) == liveness[v][k]);
                            unsigned long long delta = liveness[i][k] ^ liveness[v][k];
                            while (delta != 0) {
                                unsigned long long lb = delta & (-delta);
                                delta ^= lb;
                                EClassId w = __builtin_ctzll(lb) + (k << 6);
                                if (!g.eclasses[w].isEffectful && !init_extractable[w]) {
                                    l.push_back(compressed_eclass_id[w]);
                                }
                            } 
                        }
                    }
                }
            }
        }
    }

    // AC - satellite eclasses
    vector<EClassId> satellite_pa(g.neclasses(), UNEXTRACTABLE_ECLASS);
    vector<int> satellite_chcnt(g.neclasses(), 0);
    const int SATELLITE_BAR = 6;
    if (use_satellite_opt) {
        for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
            const EClass &c = g.eclasses[i];
            if (c.isEffectful) {
                EClassId &candidate = satellite_pa[i];
                for (ENodeId j = 0; j < (ENodeId)c.nenodes(); ++j) {
                    const ENode &n = c.enodes[j];
                    EClassId cp = UNEXTRACTABLE_ECLASS;
                    for (size_t k = 0; k < n.ch.size(); ++k) {
                        EClassId ch = n.ch[k];
                        if (g.eclasses[ch].isEffectful) {
                            cp = ch;
                            break;
                        }
                    }
                    if (cp == UNEXTRACTABLE_ECLASS) {
                        candidate = UNEXTRACTABLE_ECLASS;
                        break;
                    } else if (candidate == UNEXTRACTABLE_ECLASS) {
                        candidate = cp;
                    } else if (candidate != cp) {
                        candidate = UNEXTRACTABLE_ECLASS;
                        break;
                    }
                }
                if (candidate != UNEXTRACTABLE_ECLASS) {
                    if (parent_edge_to_effectful[i].size() == 0) {
                        candidate = UNEXTRACTABLE_ECLASS;
                    } else {
                        for (size_t j = 0; j < parent_edge_to_effectful[i].size(); ++j) {
                            if (parent_edge_to_effectful[i][j].first != candidate) {
                                candidate = UNEXTRACTABLE_ECLASS;
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
            if (satellite_pa[i] != UNEXTRACTABLE_ECLASS) {
                ++satellite_chcnt[satellite_pa[i]];
            }
        }
    }
    

    // main DP data structures

    vector<unordered_map<HashType, DPId> > dpmap(g.neclasses());
    vector<DPValue> dp;
    unordered_map<PBId, BitsetExtraInfo> bitset_extra;
    unordered_map<HashType, PBId> unifier;
    unordered_map<HashType, PBId> pure_saturation_cache;

    dpmap[argc][init_true_hash] = dp.size();
    dp.push_back(DPValue(init_cost, init_extractable_id, -1, argc, argn));
    bitset_extra.insert_or_assign(init_extractable_id, BitsetExtraInfo(init_true_hash, init_masked_hash, init_cnt_id));
    unifier[init_true_hash] = init_extractable_id;
    priority_queue<pair<Cost, DPId> > maxheap;
    maxheap.push(make_pair(~init_cost, 0));
    DPId best_statewalk = root == argc ? 0 : -1;
    while (maxheap.size() > 0) {
        Cost c = ~maxheap.top().first;
        DPId uid = maxheap.top().second;
        maxheap.pop();
        // let the dp saturate if in stat mode
        if (stat != nullptr && (best_statewalk != -1 && dp[uid].c == dp[best_statewalk].c)) {
            break;
        }
        if (dp[uid].c == c && dp[uid].ec != root) {
            EClassId u = dp[uid].ec;
            bool enable_satellite_opt = satellite_chcnt[dp[uid].ec] > SATELLITE_BAR;
            bool satellite_updated = false;
            for (size_t i = 0; i < parent_edge_to_effectful[u].size(); ++i) {
                EClassId v = parent_edge_to_effectful[u][i].first;
                ENodeId vn = parent_edge_to_effectful[u][i].second;
                bool is_satellite_update = (satellite_pa[v] == dp[uid].ec);
                if (enable_satellite_opt && is_satellite_update && satellite_updated) {
                    continue;
                }
                // test for validity
                bool can_extend = true;
                const ENode &n = g.eclasses[v].enodes[vn];
                for (size_t j = 0; j < n.ch.size(); ++j) {
                    EClassId chc = n.ch[j];
                    if (!init_extractable[chc] && (true_extractable_pool.getpos(dp[uid].root, compressed_eclass_id[chc]) == 0)) {
                        can_extend = false;
                        break;
                    }
                }
                if (can_extend) {
                    // find the new dp state
                    const BitsetExtraInfo &info = bitset_extra.find(dp[uid].root)->second;                    
                    HashType nhash;
                    PBId nroot;
                    Cost nc = c + statewalk_cost[v][vn];
                    if (best_statewalk != -1 && dp[best_statewalk].c <= nc) {
                        continue;
                    }
                    if (init_extractable[v] || true_extractable_pool.getpos(dp[uid].root, compressed_eclass_id[v]) == 1) {
                        nhash = info.masked_hash;
                        nroot = dp[uid].root;
                    } else {
                        HashType pschash = ((unsigned long long)dp[uid].root << 32) | (unsigned)v;
                        if (pure_saturation_cache.count(pschash)) {
                            nroot = pure_saturation_cache[pschash];
                            nhash = bitset_extra.find(nroot)->second.masked_hash;
                        } else {
                            enode_cnt_pool.new_version();
                            true_extractable_pool.new_version();
                            nroot = dp[uid].root;
                            BitsetExtraInfo ninfo(info);
                            if (use_liveness) {
                                //liveness-1
                                const vector<int> &delta = liveness_delta[u][v];
                                for (size_t j = 0; j < delta.size(); ++j) {
                                    if (true_extractable_pool.getpos(nroot, delta[j])) {
                                        ninfo.masked_hash ^= base_vectors[delta[j]];
                                    }
                                }
                            }
                            const vector<unsigned long long> &nliveness = liveness[v];
                            queue<EClassId> q;

                            // saturate pure
                            q.push(v);
                            nroot = true_extractable_pool.setpos(nroot, compressed_eclass_id[v]).first;
                            DEBUG_ASSERT(true_extractable_pool.getpos(nroot, compressed_eclass_id[v]))
                            ninfo.true_hash ^= base_vectors[compressed_eclass_id[v]];
                            while (q.size()) {
                                EClassId u = q.front();
                                q.pop();
                                for (size_t j = 0; j < parent_edge_to_pure[u].size(); ++j) {
                                    EClassId v = parent_edge_to_pure[u][j].first;
                                    if (!init_extractable[v] && true_extractable_pool.getpos(nroot, compressed_eclass_id[v]) == 0) {
                                        ENodeId vn = parent_edge_to_pure[u][j].second;
                                        int eid = rnk[v] + vn;
                                        pair<PBId, DataType> res = enode_cnt_pool.dec(ninfo.array, eid);
                                        ninfo.array = res.first;
                                        if (res.second == 0) {
                                            pair<PBId, bool> res = true_extractable_pool.setpos(nroot, compressed_eclass_id[v]);
                                            if (!res.second) {
                                                nroot = res.first;
                                                ninfo.true_hash ^= base_vectors[compressed_eclass_id[v]];
                                                //liveness-2
                                                if (!use_liveness || (nliveness[v >> 6] >> (v & 63)) & 1) {
                                                    ninfo.masked_hash ^= base_vectors[compressed_eclass_id[v]];
                                                }
                                                q.push(v);
                                            }
                                        }
                                    }
                                }
                            }
                            if (unifier.count(ninfo.true_hash)) {
                                nroot = unifier[ninfo.true_hash];
                                DEBUG_ASSERT(bitset_extra.count(nroot))
                                BitsetExtraInfo ninfop = bitset_extra.find(nroot)->second;
                                DEBUG_ASSERT(ninfo.true_hash == ninfop.true_hash)
                                DEBUG_ASSERT(ninfo.masked_hash == ninfop.masked_hash)
                            } else {
                                unifier[ninfo.true_hash] = nroot;
                                bitset_extra.insert_or_assign(nroot, ninfo);
                            }
                            pure_saturation_cache[pschash] = nroot;
                            nhash = ninfo.masked_hash;
                        }
                    }
                    if (enable_satellite_opt && is_satellite_update) {
                        if (nhash == info.masked_hash) {
                            continue;
                        } else {
                            satellite_updated = true;
                        }
                    }
                    // update
                    if (!dpmap[v].count(nhash)) {
                        DPId vid = dpmap[v][nhash] = dp.size();
                        dp.push_back(DPValue(nc, nroot, uid, v, vn));
                        maxheap.push(make_pair(~nc, vid));
                        if (v == root) {
                            best_statewalk = vid;
                        }
                    } else {
                        DPId vid = dpmap[v][nhash];
                        if (dp[vid].c > nc) {
                            dp[vid].c = nc;
                            dp[vid].root = nroot;
                            dp[vid].prev = uid;
                            dp[vid].pick = vn;
                            maxheap.push(make_pair(~nc, vid));
                            if (v == root) {
                                best_statewalk = vid;
                            }
                        }
                    }
                }
            }
        }
    }
    DEBUG_ASSERT(best_statewalk != -1);
    // reconstruct statewalk
    Statewalk sw;
    DPId cur = best_statewalk;
    while (cur != -1) {
        sw.push_back(make_pair(dp[cur].ec, dp[cur].pick));
        cur = dp[cur].prev;
    }
    DEBUG_ASSERT(is_valid_statewalk(g, root, sw));
    // stats
    if (stat != nullptr) {
        for (EClassId i = 0; i < (EClassId)g.neclasses(); ++i) {
            if (g.eclasses[i].isEffectful) {
                (*stat).push_back(dpmap[i].size());
            }
        }
    }
    return sw;
}