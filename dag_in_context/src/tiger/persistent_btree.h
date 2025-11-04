#ifndef PERSISTENTBTREE_H
#define PERSISTENTBTREE_H

#include<vector>

#include "debug.h"

using namespace std;

using PBId = unsigned;
using DataType = unsigned;

const int INIT_SIZE = 4000000;
const int GROWTH_FACTOR = 4;

// BP and S represent actual sizes as power of 2
template<size_t BP, size_t S> class PersistentBTree {

    protected:

        int top = 0;

        int len = 0;

        int height = 0;

        vector<PBId> mem;

        int timestamp = 0;

        inline size_t B() {
            return 1 << BP;
        }

        inline unsigned& getts(const PBId &i) {
            return mem[i];
        }

        inline unsigned& getc(const PBId &i, const int &j) {
            return mem[i + 1 + j];
        }

        PBId new_node() {
            if (top + 1 + B() > mem.size()) {
                mem.resize(mem.size() * GROWTH_FACTOR);
            }
            PBId ret = top;
            top += 1 + B();
            getts(ret) = timestamp;
            for (int i = 0; i < B(); ++i) {
                getc(ret, i) = 0;
            }
            return ret;
        }

        PBId new_node(const PBId ori) {
            if (top + 1 + B() > mem.size()) {
                mem.resize(mem.size() * GROWTH_FACTOR);
            }
            PBId ret = top;
            top += 1 + B();
            getts(ret) = timestamp;
            memcpy(&mem[ret + 1], &mem[ori + 1], sizeof(PBId) * B());
            return ret;
        }

        inline size_t single_cell_capacity_P() {
            return 5 - S;
        }

        inline size_t single_cell_capacity() {
            return (sizeof(PBId) * 8) >> S;
        }

        inline size_t single_node_capacity_P() {
            return single_cell_capacity_P() + BP;
        }

        inline size_t single_node_capacity() {
            return single_cell_capacity() << BP;
        }

        inline int chid(const int h, const int i) {
            return (i >> ((h * BP) + single_cell_capacity_P())) & (B() - 1);
        }

        inline int pos(const int i) {
            return i & (single_cell_capacity() - 1);
        }

        PBId init_recurse(const int l, const int h, const vector<DataType> &data) {
            PBId cur = new_node();
            if (h == 0) {
                for (int i = 0; i < single_node_capacity() && l + i < data.size(); ++i) {
                    unsigned &c = getc(cur, chid(0, i));
                    c |= (data[l + i] << (pos(i) << S));
                }
            } else {
                for (int i = 0; i < B(); ++i) {
                    int cl = l + (i << (h * BP + single_cell_capacity_P()));
                    if (cl >= data.size()) {
                        getc(cur, i) = -1;
                    } else {
                        getc(cur, i) = init_recurse(cl, h - 1, data);
                    }
                }
            }
            return cur;
        }

        public:

        void new_version() {
            ++timestamp;
        }

        PBId init(const vector<DataType> &data) {
            if (mem.size() < INIT_SIZE) {
                mem.resize(INIT_SIZE);
            }
            top = 0;
            timestamp = 0;
            len = data.size();
            height = 0;
            size_t sum = single_node_capacity();
            while (sum < data.size()) {
                sum <<= BP;
                ++height;
            }
            return init_recurse(0, height, data);
        }
};

class PersistentDecArray : public PersistentBTree<2, 1> {

    public:

        pair<PBId, DataType> dec(const PBId root, const int i) {
            DEBUG_ASSERT(0 <= i && i < len);
            vector<PBId> stack(height + 1);
            stack[0] = root;
            for (int h = 0; h < height; ++h) {
                stack[h + 1] = getc(stack[h], chid(height - h, i));
            }
            unsigned &c = getc(stack.back(), chid(0, i));
            unsigned val = (c >> (pos(i) << 1)) & ((1 << 2) - 1);
            if (val == 0) {
                return make_pair(root, 0);
            }
            unsigned nc = c ^ ((val ^ (val - 1)) << (pos(i) << 1));
            if (getts(stack.back()) == timestamp) {
                c = nc;
                return make_pair(root, val);
            } else {
                PBId n = new_node(stack.back());
                getc(n, chid(0, i)) = nc;
                stack.back() = n;
                for (int h = height - 1; h >= 0; --h) {
                    if (getc(stack[h], chid(height - h, i)) != stack[h + 1]) {
                        if (getts(stack[h]) == timestamp) {
                            getc(stack[h], chid(height - h, i)) = stack[h + 1];
                        } else {
                            stack[h] = new_node(stack[h]);
                            getc(stack[h], chid(height - h, i)) = stack[h + 1];
                        }
                    } else {
                        break;
                    }
                }
                return make_pair(stack[0], val);
            }
        }
};


class PersistentBitSet : public PersistentBTree<2, 0> {

    public:

        bool getpos(const PBId root, const int i) {
            DEBUG_ASSERT(0 <= i && i < len);
            PBId cur = root;
            for (int h = 0; h < height; ++h) {
                cur = chid(height - h, i);
            }
            unsigned &c = getc(cur, chid(0, i));
            unsigned val = (c >> pos(i)) & 1;
            return val;
        }

        pair<PBId, bool> setpos(const PBId root, const int i) {
            DEBUG_ASSERT(0 <= i && i < len);
            vector<PBId> stack(height + 1);
            stack[0] = root;
            for (int h = 0; h < height; ++h) {
                stack[h + 1] = getc(stack[h], chid(height - h, i));
            }
            unsigned &c = getc(stack.back(), chid(0, i));
            unsigned nc = c | (1u << (pos(i)));
            if (c == nc) {
                return make_pair(root, true);
            } else {
                if (getts(stack.back()) == timestamp) {
                    c = nc;
                    return make_pair(root, false);
                } else {
                    PBId n = new_node(stack.back());
                    getc(n, chid(0, i)) = nc;
                    stack.back() = n;
                    for (int h = height - 1; h >= 0; --h) {
                        if (getc(stack[h], chid(height - h, i)) != stack[h + 1]) {
                            if (getts(stack[h]) == timestamp) {
                                getc(stack[h], chid(height - h, i)) = stack[h + 1];
                            } else {
                                stack[h] = new_node(stack[h]);
                                getc(stack[h], chid(height - h, i)) = stack[h + 1];
                            }
                        } else {
                            break;
                        }
                    }
                    return make_pair(stack[0], false);
                }
            }
        }
};

#endif