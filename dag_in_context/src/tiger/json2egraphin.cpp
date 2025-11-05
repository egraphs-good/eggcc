#include<queue>
#include<unordered_map>
#include<vector>
#include<cstdio>
#include<string>
#include<cstring>

#include "debug.h"

#include "json2egraphin.h"

using namespace std;

queue<string> tokenbuf;

inline string read_string() {
	static char buf[505];
	scanf("%s", buf);
	return buf;
}

void read_next_token() {
	string s;
	s = read_string();
	if (s[0] == '{' || s[0] == '}') {
		tokenbuf.push(s);
	} else if (s[0] == '\"') {
		if (s[s.size() - 1] == '\"' && s[s.size() - 2] != '\\') {
			tokenbuf.push(s.substr(1, s.size() - 2));
		} else if (s[s.size() - 2] == '\"' && s[s.size() - 3] != '\\') {
			tokenbuf.push(s.substr(1, s.size() - 3));
		} else {
			string sb;
			sb = read_string();
			s = s + string(" ") + sb;
			while (s[s.size() - 1] != '\"' && s[s.size() - 2] != '\"') {
				sb = read_string();
				s = s + string(" ") + sb;
			}
			if (s[s.size() - 1] == '\"') {
				tokenbuf.push(s.substr(1, s.size() - 2));
			} else if (s[s.size() - 2] == '\"') {
				tokenbuf.push(s.substr(1, s.size() - 3));
			}
		}
	} else if (s[0] == '[') {
		if (s.size() > 1 && s[1] == ']') {
			tokenbuf.push(string("["));
			tokenbuf.push(string("]"));
		} else { 
			DEBUG_ASSERT(s.size() == 1);
			tokenbuf.push(string("["));
		}
	} else if (s[0] == ']') {
		tokenbuf.push(s);
	} else {
		tokenbuf.push(s);
	}
}

string peek_next_token() {
	if (tokenbuf.size() == 0) {
		read_next_token();
	}
	return tokenbuf.front();
}

string get_next_token() {
	if (tokenbuf.size() == 0) {
		read_next_token();
	}
	string ret = tokenbuf.front();
	tokenbuf.pop();
	return ret;
}

struct RawENode {
	string name;
	string op;
	string eclass;
	vector<string> ch;
};

vector<vector<RawENode> > raw_egraph;

unordered_map<string, EClassId> raw_eclassidmp;
unordered_map<string, pair<EClassId, ENodeId> > raw_enodeidmp;

inline void consume_next_token(const string expected) {
	string tmp = get_next_token();
	DEBUG_ASSERT(tmp == expected);
}

inline void consume_next_token_check_first_char(const char expected) {
	string tmp = get_next_token();
	DEBUG_ASSERT(tmp[0] == expected);
}

void read_node() {
	RawENode e;
	e.name = get_next_token();
	consume_next_token("{");
	consume_next_token("op");
	e.op = get_next_token();
	consume_next_token("children");
	consume_next_token("[");
	string t = get_next_token();
	while (t[0] != ']') {
		e.ch.push_back(t);
		t = get_next_token();
	}
	consume_next_token("eclass");
	e.eclass = get_next_token();
	if (!raw_eclassidmp.count(e.eclass)) {
		raw_eclassidmp[e.eclass] = raw_egraph.size();
		raw_egraph.push_back(vector<RawENode>());
	}
	consume_next_token("cost");
	//ignore cost
	get_next_token();
	consume_next_token("subsumed");
	get_next_token();
	consume_next_token_check_first_char('}');
	raw_enodeidmp[e.name] = make_pair(raw_eclassidmp[e.eclass], raw_egraph[raw_eclassidmp[e.eclass]].size());
	raw_egraph[raw_eclassidmp[e.eclass]].push_back(e);
}

void read_nodes() {
	consume_next_token("{");
	consume_next_token("nodes");
	consume_next_token("{");
	while (peek_next_token()[0] != '}') {
		read_node();
	}
}

bool isExpr(const EClassId i) {
	return (raw_egraph[i][0].eclass.size() >= 4 
		&& raw_egraph[i][0].eclass.substr(0, 4) == string("Expr"))
		|| (raw_egraph[i][0].eclass.size() >= 8
		&& raw_egraph[i][0].eclass.substr(0, 8) == string("Constant"))
		|| (raw_egraph[i][0].eclass.size() >= 9
		&& raw_egraph[i][0].eclass.substr(0, 9) == string("TernaryOp"))
		|| (raw_egraph[i][0].eclass.size() >= 8
		&& raw_egraph[i][0].eclass.substr(0, 8) == string("BinaryOp"))
		|| (raw_egraph[i][0].eclass.size() >= 7
		&& raw_egraph[i][0].eclass.substr(0, 7) == string("UnaryOp"));
}

bool isPrimitiveENode(const EClassId i, const ENodeId j) {
	return (raw_egraph[i][j].name.size() >= 9
		&& raw_egraph[i][j].name.substr(0, 9) == string("primitive"));
}

bool isPrimitiveEClass(const EClassId i) {
	for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
		if (isPrimitiveENode(i, j)) {
			return true;
		}
	}
	return false;
}

vector<EClassId> find_function_roots() {
	vector<EClassId> ret;
	for (EClassId i = 0; i < (EClassId)raw_egraph.size(); ++i) {
		for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
			if (raw_egraph[i][j].op == string("Function")) {
				ret.push_back(i);
			}
		}
	}
	return ret;
}

bool isType(const EClassId i) {
	return (raw_egraph[i][0].eclass.size() >= 4 
		&& raw_egraph[i][0].eclass.substr(0, 4) == string("Type")) 
		|| (raw_egraph[i][0].eclass.size() >= 8
		&& raw_egraph[i][0].eclass.substr(0, 8) == string("BaseType"))
		|| (raw_egraph[i][0].eclass.size() >= 8
		&& raw_egraph[i][0].eclass.substr(0, 8) == string("TypeList"));
}

vector<bool> isEffectfulType;

void propagate_effectful_types() {
	vector<vector<EClassId> > edges(raw_egraph.size());
	isEffectfulType = vector<bool>(raw_egraph.size(), false);
	for (EClassId i = 0; i < (EClassId)raw_egraph.size(); ++i) {
		if (isType(i)) {
			for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
				// assuming it will be merged with some grounded type
				if (raw_egraph[i][j].op == "TypeList-ith") {
					continue;
				}
				if (raw_egraph[i][j].op == "TypeListRemoveAt") {
					continue;
				}
				for (size_t k = 0; k < raw_egraph[i][j].ch.size(); ++k) {
					EClassId v = raw_enodeidmp[raw_egraph[i][j].ch[k]].first;
					DEBUG_ASSERT(isType(v));
					edges[v].push_back(i);
				}
			}
		}
	}
	EClassId stateT;
	for (EClassId i = 0; i < (EClassId)raw_egraph.size(); ++i) {
		for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
			if (raw_egraph[i][j].op == string("StateT")) {
				stateT = i;
			}
		}
	}
	queue<EClassId> q;
	isEffectfulType[stateT] = true;
	q.push(stateT);
	while (q.size()) {
		EClassId u = q.front();
		q.pop();
		for (int i = 0; i < (int)edges[u].size(); ++i) {
			EClassId v = edges[u][i];
			if (!isEffectfulType[v]) {
				isEffectfulType[v] = true;
				q.push(v);
			}
		}
	}
}

vector<bool> hasEffectfulType;

void mark_effectful_exprs() {
	hasEffectfulType = vector<bool>(raw_egraph.size(), false);
	for (EClassId i = 0; i < (EClassId)raw_egraph.size(); ++i) {
		for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
			if (raw_egraph[i][j].op == string("HasType")) {
				DEBUG_ASSERT(raw_egraph[i][j].ch.size() == 2);
				EClassId ec = raw_enodeidmp[raw_egraph[i][j].ch[0]].first,
						 tc = raw_enodeidmp[raw_egraph[i][j].ch[1]].first;
				DEBUG_ASSERT(isExpr(ec));
				DEBUG_ASSERT(isType(tc));
				if (isEffectfulType[tc]) {
					hasEffectfulType[ec] = true;
				}
			}
			// Additionally, mark function roots as effectful
			if (raw_egraph[i][j].op == string("Function")) {
				hasEffectfulType[i] = true;
			}
		}
	}
}

vector<bool> reachable;

vector<bool> necessary_types;

bool isTypeNormalForm(const string &op) {
	return op == "IntT" || op == "BoolT" || op == "FloatT" ||
		   op == "PointerT" || op == "StateT" || op == "Base" ||
		   op == "TupleT" || op == "TNil" || op == "TCons";
}

void mark_reachable(EClassId root) {
	if (reachable[root]) {
		return;
	}
	queue<EClassId> q, tq;
	reachable[root] = true;
	q.push(root);
	while (q.size()) {
		EClassId u = q.front();
		q.pop();
		if (isPrimitiveEClass(u)) {
			continue;
		}
		for (ENodeId i = 0; i < (ENodeId)raw_egraph[u].size(); ++i) {
			for (size_t j = 0; j < raw_egraph[u][i].ch.size(); ++j) {
				EClassId v = raw_enodeidmp[raw_egraph[u][i].ch[j]].first;
				if (!reachable[v] && (isExpr(v) || isPrimitiveEClass(v))) {
					reachable[v] = true;
					q.push(v);
				}
			}
			// Special cases for Function and Alloc to preserve the types they depend on
			if (raw_egraph[u][i].op == "Function") {
				DEBUG_ASSERT(raw_egraph[u][i].ch.size() == 4);
				EClassId inputt = raw_enodeidmp[raw_egraph[u][i].ch[1]].first,
							outputt = raw_enodeidmp[raw_egraph[u][i].ch[2]].first;
				DEBUG_ASSERT(isType(inputt));
				DEBUG_ASSERT(isType(outputt));
				if (!necessary_types[inputt]) {
					necessary_types[inputt] = true;
					tq.push(inputt);
				}
				if (!necessary_types[outputt]) {
					necessary_types[outputt] = true;
					tq.push(outputt);
				}
			}
			if (raw_egraph[u][i].op == "Alloc") {
				DEBUG_ASSERT(raw_egraph[u][i].ch.size() == 4);
				EClassId ty = raw_enodeidmp[raw_egraph[u][i].ch[3]].first;
				DEBUG_ASSERT(isType(ty));
				if (!necessary_types[ty]) {
					necessary_types[ty] = true;
					tq.push(ty);
				}
			}
		}
	}
	while (tq.size()) {
		EClassId u = tq.front();
		tq.pop();
#ifdef DEBUG
		if (!isType(u)) {
			cerr << "Found non-type children of a type: " << endl;
			cerr << raw_egraph[u][0].name << ' ' << raw_egraph[u][0].op << endl;
			assert(false);
		}
#endif
		for (ENodeId i = 0; i < (ENodeId)raw_egraph[u].size(); ++i) {
			if (isTypeNormalForm(raw_egraph[u][i].op)) {
				for (size_t j = 0; j < raw_egraph[u][i].ch.size(); ++j) {
					EClassId v = raw_enodeidmp[raw_egraph[u][i].ch[j]].first;
					if (!necessary_types[v]) {
						necessary_types[v] = true;
						tq.push(v);
					}
				}
			}
		}
	}
}

unordered_map<EClassId, EClassId> new_eclassidmp;

const char* EXTRACTABLEOP[] = {
	"Int",
	"Bool",
	"Float",
    // Leaves
    "Const",
    "Arg",
	// int, float, string
    "true",
	"false",
	"()",
    // Lists
    "Empty",
	"Single",
	"Concat",
	"Nil",
	"Cons",
    "Get",
    // Algebra
    "Abs",
	"Bitand",
	"Neg",
	"Add",
	"PtrAdd",
	"Sub",
	"And",
	"Or",
	"Not",
	"Shl",
    "Shr",
    "FAdd",
	"FSub",
	"Fmax",
	"Fmin",
    "Mul",
    "FMul",
    "Div",
	"FDiv",
    // Comparisons
    "Eq",
	"LessThan",
	"GreaterThan",
	"LessEq",
	"GreaterEq",
    "Select",
	"Smax",
	"Smin",
    "FEq",
    "FLessThan",
	"FGreaterThan",
	"FLessEq",
	"FGreaterEq",
    // Effects
    "Print",
	"Write",
	"Load",
    "Alloc",
	"Free",
    "Call",
	// Control
    "Program",
	"Function",
    // custom logic for DoWhile will multiply the body by the LoopNumItersGuess
    "DoWhile",
    "If",
	"Switch",
    // Schema
    "Bop",
	"Uop",
	"Top",
	// Function
	"Function"
};

bool isExtractableOP (const string &op) {
	if (op[0] == '\\' || op[0] == '.' || op[0] == '-' || ('0' <= op[0] && op[0] <= '9')) {
		return true;
	}
	for (int i = 0; i < sizeof(EXTRACTABLEOP) / sizeof(char*); ++i) {
		if (op == EXTRACTABLEOP[i]) {
			return true;
		}
	}
	return false;
}

EGraph build_simple_egraph() {
	EGraph g;
	new_eclassidmp.clear();
	for (EClassId i = 0; i < (EClassId)raw_egraph.size(); ++i) {
		if (reachable[i] && (isExpr(i) || isPrimitiveEClass(i))) {
			new_eclassidmp[i] = g.eclasses.size();
			g.eclasses.push_back(EClass());
			g.eclasses.back().isEffectful = hasEffectfulType[i];
		}
		if (necessary_types[i]) {
			new_eclassidmp[i] = g.eclasses.size();
			g.eclasses.push_back(EClass());
			g.eclasses.back().isEffectful = false;
		}
		DEBUG_ASSERT(!(necessary_types[i] && reachable[i]));
	}
	for (EClassId i = 0; i < (EClassId)raw_egraph.size(); ++i) {
		if (reachable[i]) {
			if (isExpr(i)) {
				EClassId nid = new_eclassidmp[i];
				for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
					RawENode &rn = raw_egraph[i][j];
					if (isExtractableOP(rn.op)) {
						ENode en;
						en.head = rn.name + "###" + rn.op;
						en.eclass = nid;
						for (size_t k = 0; k < rn.ch.size(); ++k) {
							EClassId v = raw_enodeidmp[rn.ch[k]].first;
							if (new_eclassidmp.count(v) && (!isType(v) || rn.op == "Function" || rn.op == "Alloc")) {
								en.ch.push_back(new_eclassidmp[v]);
							}
						}
						g.eclasses[nid].enodes.push_back(en);
					}
				}
			} else {
				for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
					RawENode &rn = raw_egraph[i][j];
					if (isPrimitiveENode(i, j) && isExtractableOP(rn.op)) {
						EClassId nid = new_eclassidmp[i];
						ENode en;
						en.head = rn.name + "###" + rn.op;
						en.eclass = nid;
						for (size_t k = 0; k < rn.ch.size(); ++k) {
							EClassId v = raw_enodeidmp[rn.ch[k]].first;
							if (new_eclassidmp.count(v) && !isType(v)) {
								en.ch.push_back(new_eclassidmp[v]);
							}
						}
						g.eclasses[nid].enodes.push_back(en);
					}
				}
			}
		}
		// preserve necessary types
		if (necessary_types[i]) {
			EClassId nid = new_eclassidmp[i];
			for (ENodeId j = 0; j < (ENodeId)raw_egraph[i].size(); ++j) {
				RawENode &rn = raw_egraph[i][j];
				if (isTypeNormalForm(rn.op)) {
					ENode en;
					en.head = rn.name + "###" + rn.op;
					en.eclass = nid;
					for (size_t k = 0; k < rn.ch.size(); ++k) {
						DEBUG_ASSERT(new_eclassidmp.count(raw_enodeidmp[rn.ch[k]].first));
						en.ch.push_back(new_eclassidmp[raw_enodeidmp[rn.ch[k]].first]);
					}
					g.eclasses[nid].enodes.push_back(en);
				}
			}
			DEBUG_ASSERT(g.eclasses[nid].enodes.size() == 1);
		}
	}
	return g;
}

pair<EGraph, vector<EClassId> > parse_egglog_json() {
	read_nodes();
	propagate_effectful_types();
	mark_effectful_exprs();
	vector<EClassId> roots = find_function_roots();
	reachable = vector<bool>(raw_egraph.size(), false);
	necessary_types = vector<bool>(raw_egraph.size(), false);	
	for (int i = 0; i < (int)roots.size(); ++i) {
		mark_reachable(roots[i]);
	}
	EGraph g = build_simple_egraph();
	DEBUG_ASSERT(is_wellformed_egraph(g, true, true));
	pair<EGraph, EGraphMapping> p = prune_unextractable_enodes(g);
    vector<EClassId> new_roots(roots.size());
    for (size_t i = 0; i < roots.size(); ++i) {
		DEBUG_ASSERT(new_eclassidmp.count(roots[i]));
        new_roots[i] = p.second.eclassidmp[new_eclassidmp[roots[i]]];
		DEBUG_ASSERT(0 <= new_roots[i] && new_roots[i] < p.first.neclasses());
    }
    return make_pair(p.first, new_roots);
}
