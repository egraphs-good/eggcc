#include<cassert>
#include<queue>
#include<map>
#include<vector>
#include<cstdio>
#include<string>
#include<cstring>
#include<iostream>
#include<algorithm>

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
			//cout << s << endl;
			//assert(false); //format error
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
			assert(s.size() == 1);
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
	//cout << ret << endl;
	return ret;
}

typedef int EClassId;
typedef int ENodeId;

struct RawENode {
	string name;
	string op;
	string eclass;
	vector<string> ch;
	bool subsumed;
};

vector<vector<RawENode> > egraph;

map<string, EClassId> eclassidmp;
map<string, pair<EClassId, ENodeId> > enodeidmp;

void read_node() {
	RawENode e;
	e.name = get_next_token();
	assert(get_next_token() == string("{"));
	assert(get_next_token() == string("op"));	
	e.op = get_next_token();
	assert(get_next_token() == string("children"));
	assert(get_next_token() == string("["));
	string t = get_next_token();
	while (t[0] != ']') {
		e.ch.push_back(t);
		t = get_next_token();
	}
	assert(get_next_token() == string("eclass"));
	e.eclass = get_next_token();
	if (!eclassidmp.count(e.eclass)) {
		eclassidmp[e.eclass] = egraph.size();
		egraph.push_back(vector<RawENode>());
	}
	assert(get_next_token() == string("cost"));
	//ignore cost for now
	get_next_token();
	assert(get_next_token() == string("subsumed"));
	if (get_next_token() == string("false")) {
		e.subsumed = false;
	} else {
		e.subsumed = true;
	}
	assert(get_next_token()[0] == '}');
	enodeidmp[e.name] = make_pair(eclassidmp[e.eclass], egraph[eclassidmp[e.eclass]].size());
	egraph[eclassidmp[e.eclass]].push_back(e);
}

void read_nodes() {
	assert(get_next_token() == string("{"));
	assert(get_next_token() == string("nodes"));
	assert(get_next_token() == string("{"));
	while (peek_next_token()[0] != '}') {
		read_node();
	}
}

bool isExpr(const EClassId i) {
	return (egraph[i][0].eclass.size() >= 4 
		&& egraph[i][0].eclass.substr(0, 4) == string("Expr"))
		|| (egraph[i][0].eclass.size() >= 8
		&& egraph[i][0].eclass.substr(0, 8) == string("Constant"))
		|| (egraph[i][0].eclass.size() >= 9
		&& egraph[i][0].eclass.substr(0, 9) == string("TernaryOp"))
		|| (egraph[i][0].eclass.size() >= 8
		&& egraph[i][0].eclass.substr(0, 8) == string("BinaryOp"))
		|| (egraph[i][0].eclass.size() >= 7
		&& egraph[i][0].eclass.substr(0, 7) == string("UnaryOp"));
}

bool isPrimitiveENode(const EClassId i, const ENodeId j) {
	return (egraph[i][j].name.size() >= 9
		&& egraph[i][j].name.substr(0, 9) == string("primitive"));
}

bool isPrimitiveEClass(const EClassId i) {
	for (int j = 0; j < (int)egraph[i].size(); ++j) {
		if (isPrimitiveENode(i, j)) {
			return true;
		}
	}
	return false;
}

vector<EClassId> findFunctionRoots() {
	vector<EClassId> ret;
	for (int i = 0; i < (int)egraph.size(); ++i) {
		for (int j = 0; j < (int)egraph[i].size(); ++j) {
			if (egraph[i][j].op == string("Function")) {
				//ret.push_back(enodeidmp[egraph[i][j].ch.back()].first);
				ret.push_back(i);
			}
		}
	}
	return ret;
}

bool isType(const EClassId i) {
	return (egraph[i][0].eclass.size() >= 4 
		&& egraph[i][0].eclass.substr(0, 4) == string("Type")) 
		|| (egraph[i][0].eclass.size() >= 8
		&& egraph[i][0].eclass.substr(0, 8) == string("BaseType"))
		|| (egraph[i][0].eclass.size() >= 8
		&& egraph[i][0].eclass.substr(0, 8) == string("TypeList"));
}

vector<bool> isEffectfulType;

void propagate_effectful_types() {
	vector<vector<EClassId> > edges(egraph.size());
	isEffectfulType = vector<bool>(egraph.size(), false);
	for (int i = 0; i < (int)egraph.size(); ++i) {
		if (isType(i)) {
			for (int j = 0; j < (int)egraph[i].size(); ++j) {
				// assuming it will be merged with some grounded type
				if (egraph[i][j].op == "TypeList-ith") {
					continue;
				}
				if (egraph[i][j].op == "TypeListRemoveAt") {
					continue;
				}
				//cout << egraph[i][j].name << endl;
				//cout << egraph[i][j].op << endl;
				for (int k = 0; k < (int)egraph[i][j].ch.size(); ++k) {
					//cout << ":" << egraph[i][j].ch[k] << endl;
					EClassId v = enodeidmp[egraph[i][j].ch[k]].first;
					assert(isType(v));
					edges[v].push_back(i);
				}
			}
		}
	}
	EClassId stateT;
	for (int i = 0; i < (int)egraph.size(); ++i) {
		for (int j = 0; j < (int)egraph[i].size(); ++j) {
			if (egraph[i][j].op == string("StateT")) {
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
	hasEffectfulType = vector<bool>(egraph.size(), false);
	for (int i = 0; i < (int)egraph.size(); ++i) {
		for (int j = 0; j < (int)egraph[i].size(); ++j) {
			if (egraph[i][j].op == string("HasType")) {
				assert(egraph[i][j].ch.size() == 2);
				EClassId ec = enodeidmp[egraph[i][j].ch[0]].first,
						 tc = enodeidmp[egraph[i][j].ch[1]].first;
				//cout << egraph[i][j].ch[0] << ' ' << egraph[i][j].ch[1] << endl;
				assert(isExpr(ec));
				assert(isType(tc));
				if (isEffectfulType[tc]) {
					hasEffectfulType[ec] = true;
				}
			}
			// Additionally, mark function roots as effectful
			if (egraph[i][j].op == string("Function")) {
				hasEffectfulType[i] = true;
			}
		}
	}
}

vector<bool> reachable;

vector<bool> necessary_types;

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
		for (int i = 0; i < (int)egraph[u].size(); ++i) {
			if (!egraph[u][i].subsumed) {
				for (int j = 0; j < (int)egraph[u][i].ch.size(); ++j) {
					EClassId v = enodeidmp[egraph[u][i].ch[j]].first;
					if (!reachable[v] && (isExpr(v) || isPrimitiveEClass(v))) {
						reachable[v] = true;
						q.push(v);
					}
				}
				// Special cases for Function and Alloc to preserve the types they depend on
				if (egraph[u][i].op == "Function") {
					assert(egraph[u][i].ch.size() == 4);
					EClassId inputt = enodeidmp[egraph[u][i].ch[1]].first,
							 outputt = enodeidmp[egraph[u][i].ch[2]].first;
					assert(isType(inputt));
					assert(isType(outputt));
					if (!necessary_types[inputt]) {
						necessary_types[inputt] = true;
						tq.push(inputt);
					}
					if (!necessary_types[outputt]) {
						necessary_types[outputt] = true;
						tq.push(outputt);
					}
				}
				if (egraph[u][i].op == "Alloc") {
					assert(egraph[u][i].ch.size() == 4);
					EClassId ty = enodeidmp[egraph[u][i].ch[3]].first;
					assert(isType(ty));
					if (!necessary_types[ty]) {
						necessary_types[ty] = true;
						tq.push(ty);
					}
				}
			}
		}
	}
	while (tq.size()) {
		EClassId u = tq.front();
		tq.pop();
		assert(isType(u));
		for (int i = 0; i < (int)egraph[u].size(); ++i) {
			if (!egraph[u][i].subsumed && egraph[u][i].op != "TypeList-ith" && egraph[u][i].op != "TLConcat") {
				//cout << egraph[u][i].name << ' ' << egraph[u][i].op << endl;
				for (int j = 0; j < (int)egraph[u][i].ch.size(); ++j) {
					//cout << " " << egraph[u][i].ch[j] << endl;
					EClassId v = enodeidmp[egraph[u][i].ch[j]].first;
					if (!necessary_types[v]) {
						necessary_types[v] = true;
						tq.push(v);
					}
				}
			}
		}
	}
}

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

EGraph g;
map<EClassId, EClassId> neweclassidmp;

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

void build_simple_egraph() {
	g.eclasses.clear();
	neweclassidmp.clear();
	for (int i = 0; i < (int)egraph.size(); ++i) {
		if (reachable[i] && (isExpr(i) || isPrimitiveEClass(i))) {
			neweclassidmp[i] = g.eclasses.size();
			g.eclasses.push_back(EClass());
			g.eclasses.back().isEffectful = hasEffectfulType[i];
		}
		if (necessary_types[i]) {
			neweclassidmp[i] = g.eclasses.size();
			g.eclasses.push_back(EClass());
			g.eclasses.back().isEffectful = false;
		}
	}
	for (int i = 0; i < (int)egraph.size(); ++i) {
		if (reachable[i]) {
			if (isExpr(i)) {
				EClassId nid = neweclassidmp[i];
				for (int j = 0; j < (int)egraph[i].size(); ++j) {
					if (!egraph[i][j].subsumed && isExtractableOP(egraph[i][j].op)) {
						ENode en;
						en.head = egraph[i][j].name + "###" + egraph[i][j].op;
						en.eclass = nid;
						for (int k = 0; k < (int)egraph[i][j].ch.size(); ++k) {
							EClassId v = enodeidmp[egraph[i][j].ch[k]].first;
							if (neweclassidmp.count(v) && (!isType(v) || egraph[i][j].op == "Function" || egraph[i][j].op == "Alloc")) {
								en.ch.push_back(neweclassidmp[v]);
							}
						}
						g.eclasses[nid].enodes.push_back(en);
					}
				}
			} else {
				for (int j = 0; j < (int)egraph[i].size(); ++j) {
					if (!egraph[i][j].subsumed && isPrimitiveENode(i, j) && isExtractableOP(egraph[i][j].op)) {
						EClassId nid = neweclassidmp[i];
						ENode en;
						en.head = egraph[i][j].name + "###" + egraph[i][j].op;
						en.eclass = nid;
						for (int k = 0; k < (int)egraph[i][j].ch.size(); ++k) {
							EClassId v = enodeidmp[egraph[i][j].ch[k]].first;
							if (neweclassidmp.count(v) && !isType(v)) {
								en.ch.push_back(neweclassidmp[v]);
							}
						}
						g.eclasses[nid].enodes.push_back(en);
					}
				}
			}
		}
		// preserve necessary types
		if (necessary_types[i]) {
			EClassId nid = neweclassidmp[i];
			for (int j = 0; j < (int)egraph[i].size(); ++j) {
				if (egraph[i][j].op != "TypeList-ith" && egraph[i][j].op != "TLConcat") {
					ENode en;
					en.head = egraph[i][j].name + "###" + egraph[i][j].op;
					en.eclass = nid;
					for (int k = 0; k < (int)egraph[i][j].ch.size(); ++k) {
						assert(neweclassidmp.count(enodeidmp[egraph[i][j].ch[k]].first));
						en.ch.push_back(neweclassidmp[enodeidmp[egraph[i][j].ch[k]].first]);
					}
					g.eclasses[nid].enodes.push_back(en);
				}
			}
			assert(g.eclasses[nid].enodes.size() == 1);
		}
	}
}

void print_egraph(const EGraph &g) {
	int n = g.eclasses.size();
	printf("%d\n", n);
	for (int i = 0; i < n; ++i) {
		const EClass &c = g.eclasses[i];
		int f = c.isEffectful ? 1 : 0,
			m = c.enodes.size();
		printf("# %d\n", i);
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

int main() {
	read_nodes();
	propagate_effectful_types();
	mark_effectful_exprs();
	vector<EClassId> roots = findFunctionRoots();
	reachable = vector<bool>(egraph.size(), false);
	necessary_types = vector<bool>(egraph.size(), false);	
	for (int i = 0; i < (int)roots.size(); ++i) {
		//cout << roots[i] << ' ' << egraph[roots[i]][0].name << endl;
		mark_reachable(roots[i]);
	}
	build_simple_egraph();
	print_egraph(g);
	for (int i = 0; i < (int)roots.size(); ++i) {
		assert(neweclassidmp.count(roots[i]));
		printf("%d\n", neweclassidmp[roots[i]]);
	}
	return 0;
}
