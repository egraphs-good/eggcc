#include "ilp.h"

#include <cassert>
#include <chrono>
#include <cstdlib>
#include <errno.h>
#include <fstream>
#include <functional>
#include <iostream>
#include <map>
#include <set>
#include <signal.h>
#include <sstream>
#include <string>
#include <sys/types.h>
#include <sys/wait.h>
#include <thread>
#include <unistd.h>
#include <unordered_map>
#include <unordered_set>
#include <vector>

using namespace std;

bool g_use_gurobi = true;
// 10 sec timeout on nightly with cbc
int g_ilp_timeout_seconds = 10;
// 5 min timeout with gurobi
int g_ilp_timeout_gurobi = 5 * 60;
bool g_ilp_minimize_objective = true;
extern bool g_time_ilp;

static void kill_process_group(pid_t pid) {
	if (pid <= 0) {
		return;
	}
	pid_t pgid = getpgid(pid);
	if (pgid == pid) {
		if (kill(-pgid, SIGKILL) == -1 && errno != ESRCH) {
			kill(pid, SIGKILL);
		}
	} else {
		kill(pid, SIGKILL);
	}
}

static int run_command_with_timeout(const string &command, int timeout_seconds, bool &timed_out) {
	timed_out = false;
#if defined(_WIN32)
	(void)timeout_seconds;
	int result = system(command.c_str());
	return result;
#else
	pid_t pid = fork();
	if (pid < 0) {
		return -1;
	}
	if (pid == 0) {
		if (setsid() == -1) {
			setpgid(0, 0);
		}
		execl("/bin/sh", "sh", "-c", command.c_str(), (char *)nullptr);
		_exit(127);
	}
	setpgid(pid, pid);
		

	int status = 0;
	if (timeout_seconds <= 0) {
		while (waitpid(pid, &status, 0) < 0) {
			if (errno != EINTR) {
				int err = errno;
				kill_process_group(pid);
				waitpid(pid, &status, 0);
				errno = err;
				return -1;
			}
		}
	} else {
			auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(timeout_seconds);
		while (true) {
			pid_t result = waitpid(pid, &status, WNOHANG);
			if (result == pid) {
				break;
			}
			if (result == 0) {
					if (std::chrono::steady_clock::now() >= deadline) {
					timed_out = true;
					kill_process_group(pid);
					waitpid(pid, &status, 0);
					break;
				}
					std::this_thread::sleep_for(std::chrono::milliseconds(50));
				continue;
			}
			if (result == -1) {
				if (errno == EINTR) {
					continue;
				}
				kill_process_group(pid);
				waitpid(pid, &status, 0);
				return -1;
			}
		}
	}

	if (timed_out) {
		return -1;
	}
	if (WIFEXITED(status)) {
		return WEXITSTATUS(status);
	}
	if (WIFSIGNALED(status)) {
		return 128 + WTERMSIG(status);
	}
	return -1;
#endif
}

struct SolverSolution {
	unordered_map<string, double> values;
	bool infeasible = false;
};

static inline bool is_space_char(char ch) {
	return ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == '\f' || ch == '\v';
}

static string trim_copy(const string &s) {
	size_t start = 0;
	while (start < s.size() && is_space_char(s[start])) {
		++start;
	}
	size_t end = s.size();
	while (end > start && is_space_char(s[end - 1])) {
		--end;
	}
	return s.substr(start, end - start);
}

static string lowercase_ascii(string s) {
	for (char &ch : s) {
		if (ch >= 'A' && ch <= 'Z') {
			ch = ch - 'A' + 'a';
		}
	}
	return s;
}

static bool contains_case_insensitive(const string &haystack, const string &needle) {
	string hay_lower = lowercase_ascii(haystack);
	string needle_lower = lowercase_ascii(needle);
	return hay_lower.find(needle_lower) != string::npos;
}

template <typename FailFn>
static SolverSolution parse_solver_solution(const string &sol_path,
															const string &solver_log,
															const string &solver_name,
															bool solver_uses_xml,
															const FailFn &fail_with_log) {
	SolverSolution result;
	ifstream sol(sol_path.c_str());
	if (!sol.good()) {
		fail_with_log(string("failed to open ") + solver_name + " solution file");
	}
	vector<string> lines;
	string line;
	bool has_content = false;
	while (getline(sol, line)) {
		lines.push_back(line);
		if (!line.empty()) {
			has_content = true;
		}
		if (line.find("Infeasible") != string::npos || line.find("infeasible") != string::npos ||
			line.find("INFEASIBLE") != string::npos) {
			result.infeasible = true;
		}
	}
	sol.close();
	if (!has_content && !result.infeasible) {
		fail_with_log(string("") + solver_name + " produced an empty solution file");
	}
	if (result.infeasible) {
		return result;
	}
	
	for (const string &raw_line : lines) {
		string trimmed = trim_copy(raw_line);
		if (trimmed.empty()) {
			continue;
		}
		if (trimmed[0] == '#') {
			continue;
		}
		string lower_trimmed = lowercase_ascii(trimmed);
		if (lower_trimmed.find("objective value") != string::npos ||
			lower_trimmed.find("solution status") != string::npos ||
			lower_trimmed.find("solution time") != string::npos) {
			continue;
		}
		stringstream ss(trimmed);
		vector<string> tokens;
		string tok;
		while (ss >> tok) {
			tokens.push_back(tok);
		}
		if (tokens.empty()) {
			continue;
		}
		if (tokens.size() >= 2 &&
				((isalpha(static_cast<unsigned char>(tokens[0][0])) || tokens[0][0] == '_' ||
					tokens[0].find('(') != string::npos))) {
			try {
				double value = stod(tokens[1]);
				result.values[tokens[0]] = value;
				continue;
			} catch (...) {
			}
		}
		if (tokens.size() >= 3) {
			try {
				double value = stod(tokens[2]);
				result.values[tokens[1]] = value;
				continue;
			} catch (...) {
			}
		}
	}
	if (result.values.empty() && contains_case_insensitive(solver_log, "infeasible")) {
		result.infeasible = true;
	}
	return result;
}

EClassId enode_to_eclass(const EGraph &g, ENodeId n) {
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId m = 0; m < (ENodeId)g.eclasses[c].enodes.size(); ++m) {
			if (n == m) {
				return c;
			}
		}
	}
	return -1;
}

void print_enode(ostream &out, const ENode &n) {
	out << n.head << "(";
	for (int i = 0; i < (int)n.ch.size(); ++i) {
		if (i > 0) {
			out << ",";
		}
		out << n.ch[i];
	}
	out << ")";
}


void print_eclass(ostream &out, const EGraph &g, EClassId c) {
	out << "EClass " << c << (g.eclasses[c].isEffectful ? " (effectful)" : "") << ":\n";
	for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
		out << "  ";
		print_enode(out, g.eclasses[c].enodes[n]);
		out << "\n";
	}
}

namespace {

struct ChoiceVar {
	string name;
	EClassId parent_class;
	ENodeId parent_node;
	int child_idx;
	EClassId child_class;
	ENodeId child_node;
};

static inline long long encode_child_selection_key(EClassId cls, ENodeId node) {
	return (static_cast<long long>(cls) << 32) |
	       static_cast<unsigned long long>(static_cast<unsigned int>(node));
}

static unordered_map<string, bool> build_binary_value_map(
		const vector<vector<string> > &pickNode,
		const vector<ChoiceVar> &choices,
		const unordered_map<string, double> &raw_values) {
	size_t estimate = choices.size();
	for (const auto &row : pickNode) {
		estimate += row.size();
	}
	unordered_map<string, bool> result;
	result.reserve(estimate);
	for (const auto &row : pickNode) {
		for (const string &name : row) {
			double value = 0.0;
			auto it = raw_values.find(name);
			if (it != raw_values.end()) {
				value = it->second;
			}
			result.emplace(name, value > 0.5);
		}
	}
	for (const ChoiceVar &cv : choices) {
		double value = 0.0;
		auto it = raw_values.find(cv.name);
		if (it != raw_values.end()) {
			value = it->second;
		}
		result.emplace(cv.name, value > 0.5);
	}
	return result;
}

template <typename FailFn>
static bool require_binary_value(const unordered_map<string, bool> &value_map,
									 const string &name,
									 const FailFn &fail) {
	auto it = value_map.find(name);
	if (it == value_map.end()) {
		fail(string("missing solver assignment for variable ") + name);
		return false;
	}
	return it->second;
}

template <typename FailFn>
void build_child_selection_for_roots(
		const EGraph &g,
		EClassId root_class,
		const vector<ENodeId> &root_enodes,
		const vector<vector<int> > &pickSelected,
		const vector<vector<vector<vector<int> > > > &choiceIndex,
		const vector<ChoiceVar> &choices,
		const vector<vector<string> > &pickNode,
		const unordered_map<string, bool> &value_map,
		const FailFn &fail,
		vector<vector<vector<ENodeId> > > &childSelection) {
	(void)root_class;
	(void)root_enodes;
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			if (!pickSelected[c][n]) {
				continue;
			}
			const ENode &en = g.eclasses[c].enodes[n];
			for (int child_idx = 0; child_idx < (int)en.ch.size(); ++child_idx) {
				const vector<int> &choice_list = choiceIndex[c][n][child_idx];
				if (choice_list.empty()) {
					continue;
				}

				int chosen_choice_idx = -1;
				for (int idx : choice_list) {
					bool chosen = require_binary_value(value_map, choices[idx].name, fail);
					if (chosen && (chosen_choice_idx == -1 || idx < chosen_choice_idx)) {
						chosen_choice_idx = idx;
					}
				}
				if (chosen_choice_idx == -1) {
					cerr << "Missing child selection for eclass " << c << " node " << n
				     << " child index " << child_idx << " options:";
					for (int idx : choice_list) {
						bool opt_v = require_binary_value(value_map, choices[idx].name, fail);
						cerr << ' ' << choices[idx].name << "=" << (opt_v ? 1 : 0);
					}
					bool pick_v = require_binary_value(value_map, pickNode[c][n], fail);
					cerr << " (pickNode=" << (pick_v ? 1 : 0) << ")" << endl;
					fail("missing child selection for picked enode");
				}
				const ChoiceVar &chosen = choices[chosen_choice_idx];
				EClassId child_class = chosen.child_class;
				ENodeId child_node = chosen.child_node;
				if (child_node < 0 || child_node >= (ENodeId)g.eclasses[child_class].enodes.size()) {
					fail("child selection index out of bounds");
				}
				if (!pickSelected[child_class][child_node]) {
					fail("child enode not marked as picked");
				}
				childSelection[c][n][child_idx] = child_node;
			}
		}
	}
}

template <typename FailFn>
static ExtractionENodeId build_extraction_node(
		const EGraph &g,
		const vector<vector<vector<ENodeId> > > &childSelection,
		EClassId c,
		ENodeId n,
		vector<ExtractionENode> &extraction,
		unordered_map<long long, ExtractionENodeId> &nodeIndex,
		unordered_set<long long> &usedEffectful,
		unordered_set<long long> &visiting,
		const FailFn &fail) {
	long long key = (static_cast<long long>(c) << 32) |
	               static_cast<unsigned long long>(static_cast<unsigned int>(n));
	auto it = nodeIndex.find(key);
	if (it != nodeIndex.end()) {
		return it->second;
	}
	if (visiting.count(key)) {
		fail("cycle detected when building extraction");
	}
	visiting.insert(key);
	const ENode &en = g.eclasses[c].enodes[n];
	vector<ExtractionENodeId> ch_idx;
	ch_idx.reserve(en.ch.size());
	for (int child_i = 0; child_i < (int)en.ch.size(); ++child_i) {
		EClassId child_class = en.ch[child_i];
		ENodeId child_node = childSelection[c][n][child_i];
		if (child_node == -1) {
			fail("missing child during extraction reconstruction");
		}
		ExtractionENodeId child_ex = build_extraction_node(
			g, childSelection, child_class, child_node,
			extraction, nodeIndex, usedEffectful, visiting, fail);
		ch_idx.push_back(child_ex);
	}
	visiting.erase(key);
	ExtractionENode node;
	node.c = c;
	node.n = n;
	node.ch = ch_idx;
	ExtractionENodeId idx = extraction.size();
	extraction.push_back(node);
	nodeIndex[key] = idx;
	if (g.eclasses[c].isEffectful) {
		usedEffectful.insert(key);
	}
	return idx;
}

} // namespace

Extraction extractRegionILP(const EGraph &g, const EClassId initc, const ENodeId initn, const EClassId root, const vector<vector<int> > &nsubregion, bool &timed_out)  {
	auto fail = [&](const string &msg) -> void {
		cerr << "ILP extraction error: " << msg << endl;
		exit(1);
	};
	timed_out = false;

	if (root == initc) {
		StateWalk sw;
		sw.push_back(make_pair(root, initn));
		return regionExtractionWithStateWalk(g, root, sw).second;
	}

	// VARIABLES
	// Picking an enode in an eclass
	vector<vector<string> > pickNode(g.eclasses.size());
	// Choosing an eclass, enode, child index, and child enode index
	vector<vector<vector<vector<int> > > > choiceIndex(g.eclasses.size());
	// Order variables for acyclicity
	vector<vector<string> > orderVar(g.eclasses.size());

	// COST MODEL
	// Cost of picking an enode in an eclass
	vector<vector<long long> > pickCost(g.eclasses.size());
	
	// CACHES
	// For each child enode, which choice variables point to it
	vector<vector<vector<int> > > childParents(g.eclasses.size());
	// Effectful child flow tracking
	// For a given eclass, which choice variables are outgoing/incoming effectful edges
	vector<vector<int> > effectOutgoing(g.eclasses.size());
	vector<vector<int> > effectIncoming(g.eclasses.size());

	int total_enodes = 0;
	for (const EClass &ec : g.eclasses) {
		total_enodes += ec.enodes.size();
	}
	int maxOrder = max(1, total_enodes);
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		pickNode[c].resize(g.eclasses[c].enodes.size());
		pickCost[c].resize(g.eclasses[c].enodes.size());
		choiceIndex[c].resize(g.eclasses[c].enodes.size());
		childParents[c].resize(g.eclasses[c].enodes.size());
		orderVar[c].resize(g.eclasses[c].enodes.size());
	}

	// All choice variables (a partiacular edge between an enode at a child index and another enode)
	vector<ChoiceVar> choices;
	// initialize choices, pickNode, pickCost, choiceIndex, childParents
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			pickNode[c][n] = string("p_") + to_string(c) + "_" + to_string(n);
			orderVar[c][n] = string("o_") + to_string(c) + "_" + to_string(n);
			long long add = 0;
			if (c < (EClassId)nsubregion.size() && n < (ENodeId)nsubregion[c].size()) {
				add = nsubregion[c][n];
			}
			pickCost[c][n] = 1 + 1000LL * add;
			const ENode &en = g.eclasses[c].enodes[n];
			choiceIndex[c][n].resize(en.ch.size());
			for (int child_idx = 0; child_idx < (int)en.ch.size(); ++child_idx) {
				EClassId child_class = en.ch[child_idx];
				if (child_class < 0 || child_class >= (EClassId)g.eclasses.size()) {
					fail("child eclass index out of bounds");
				}
				const EClass &child_ec = g.eclasses[child_class];
				if (child_ec.enodes.empty()) {
					fail("child eclass has no enodes to select");
				}
				vector<int> &idx_list = choiceIndex[c][n][child_idx];
				idx_list.reserve(child_ec.enodes.size());
				for (ENodeId m = 0; m < (ENodeId)child_ec.enodes.size(); ++m) {
					ChoiceVar cv;
					cv.name = string("s_") + to_string(c) + "_" + to_string(n) + "_" + to_string(child_idx) + "_" + to_string(m);
					cv.parent_class = c;
					cv.parent_node = n;
					cv.child_idx = child_idx;
					cv.child_class = child_class;
					cv.child_node = m;
					int idx = choices.size();
					choices.push_back(cv);
					idx_list.push_back(idx);
					childParents[child_class][m].push_back(idx);
					if (g.eclasses[c].isEffectful && g.eclasses[child_class].isEffectful) {
						effectOutgoing[c].push_back(idx);
						effectIncoming[child_class].push_back(idx);
					}
				}
			}
		}
	}

	char lp_template[] = "/tmp/extract_regionXXXXXX.lp";
	int lp_fd = mkstemps(lp_template, 3);
	if (lp_fd == -1) {
		fail("failed to create LP temp file");
	}
	string lp_path(lp_template);
	close(lp_fd);

	char sol_template[] = "/tmp/extract_regionXXXXXX.sol";
	int sol_fd = mkstemps(sol_template, 4);
	if (sol_fd == -1) {
		unlink(lp_path.c_str());
		fail("failed to create solution temp file");
	}
	string sol_path(sol_template);
	close(sol_fd);

	char log_template[] = "/tmp/extract_regionXXXXXX.log";
	int log_fd = mkstemps(log_template, 4);
	if (log_fd == -1) {
		unlink(lp_path.c_str());
		unlink(sol_path.c_str());
		fail("failed to create log temp file");
	}
	string log_path(log_template);
	close(log_fd);

	struct FileCleaner {
		string path;
		~FileCleaner() {
			if (!path.empty()) {
				unlink(path.c_str());
			}
		}
	};
	FileCleaner lp_cleaner{lp_path};
	FileCleaner sol_cleaner{sol_path};
	FileCleaner log_cleaner{log_path};

	ofstream lp(lp_path.c_str());
	if (!lp.good()) {
		fail("failed to open LP file for writing");
	}

	bool firstTerm = true;
	// optionally minimize sum pickCost[c][n] * pickNode[c][n]
	lp << "Minimize\n";
	if (g_ilp_minimize_objective) {
		lp << " obj:";
		for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
			for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
				if (!firstTerm) {
					lp << " +";
				}
				firstTerm = false;
				lp << " " << pickCost[c][n] << " " << pickNode[c][n];
			}
		}
		if (firstTerm) {
			lp << " 0";
		}
		lp << "\n";
	} else {
		lp << " obj: 0\n";
	}
	lp << "Subject To\n";

	// Require at least one root enode to be picked
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		if (c != root) {
			continue;
		}
		if (g.eclasses[c].enodes.empty()) {
			fail("encountered eclass with no enodes");
		}
		lp << " pick_sum_" << c << ":";
		bool first = true;
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			lp << (first ? " " : " + ") << pickNode[c][n];
			first = false;
		}
		lp << " >= 1\n";
	}

	// If you pick an enode, for every child index pick at least one child edge.
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			const vector<vector<int> > &idx_lists = choiceIndex[c][n];
			for (int child_idx = 0; child_idx < (int)idx_lists.size(); ++child_idx) {
				const vector<int> &list = idx_lists[child_idx];
				if (list.empty()) {
					continue;
				}
				lp << " child_select_" << c << '_' << n << '_' << child_idx << ":";
				bool first = true;
				for (int idx : list) {
					lp << (first ? " " : " + ") << choices[idx].name;
					first = false;
					// sanity check: assert that the parent eclass of the choice is c and parent_node is n
					assert(choices[idx].parent_class == c && choices[idx].parent_node == n);
				}
				lp << " - " << pickNode[c][n] << " >= 0\n";
			}
		}
	}
	
	// If you choose a child edge, you must pick the enode it points to.
	for (int idx = 0; idx < (int)choices.size(); ++idx) {
		const ChoiceVar &cv = choices[idx];
		lp << " child_link_" << idx << ": " << cv.name << " - " << pickNode[cv.child_class][cv.child_node] << " <= 0\n";
	} 

	// Linearity: effectful enodes may not be targeted by multiple effectful parents.
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		if (!g.eclasses[c].isEffectful) {
			continue;
		}
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			const vector<int> &parents = childParents[c][n];
			if (parents.empty()) {
				continue;
			}
			vector<int> effectful_parents;
			effectful_parents.reserve(parents.size());
			for (int idx : parents) {
				if (g.eclasses[choices[idx].parent_class].isEffectful) {
					effectful_parents.push_back(idx);
				}
			}
			if (effectful_parents.empty()) {
				continue;
			}
			lp << " child_unique_" << c << '_' << n << ":";
			bool first = true;
			for (int idx : effectful_parents) {
				lp << (first ? " " : " + ") << choices[idx].name;
				first = false;
			}
			lp << " <= 1\n";
		}
	}

	// Order variables must decrease along chosen edges to prevent cycles.
	// When parent and child are the same enode, forbid taking that edge to avoid duplicate constraints.
	for (int idx = 0; idx < (int)choices.size(); ++idx) {
		const ChoiceVar &cv = choices[idx];
		if (cv.parent_class == cv.child_class && cv.parent_node == cv.child_node) {
			lp << " order_edge_" << idx << ": " << maxOrder << " " << cv.name
			   << " <= " << (maxOrder - 1) << "\n";
		} else {
			lp << " order_edge_" << idx << ": " << orderVar[cv.child_class][cv.child_node]
			   << " - " << orderVar[cv.parent_class][cv.parent_node]
			   << " + " << maxOrder << " " << cv.name
			   << " <= " << (maxOrder - 1) << "\n";
		}
	}


	lp << "Bounds\n";
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			lp << " 0 <= " << orderVar[c][n] << " <= " << (maxOrder - 1) << "\n";
		}
	}

	lp << "Binary\n";
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			lp << " " << pickNode[c][n] << "\n";
		}
	}
	for (const ChoiceVar &cv : choices) {
		lp << " " << cv.name << "\n";
	}
	lp << "End\n";
	lp.close();
	{
		ifstream in_debug(lp_path.c_str(), ios::binary);
		ofstream out_debug("/tmp/tiger_last_extract.lp", ios::binary);
		out_debug << in_debug.rdbuf();
	}

	string solver_name = g_use_gurobi ? "gurobi" : "cbc";
	string cmd = "";
	if (g_use_gurobi) {
		cmd = string("gurobi_cl Threads=1 ResultFile=\"") + sol_path + "\" LogFile=\"" + log_path + "\" " + lp_path + " > /dev/null 2>&1";
	} else {
		cmd = string("cbc \"") + lp_path + "\" solve branch solu \"" + sol_path + "\" > \"" + log_path + "\" 2>&1";
	}
	bool solver_timed_out = false;
	int timeout = g_use_gurobi ? g_ilp_timeout_gurobi : g_ilp_timeout_seconds;
	int ret = run_command_with_timeout(cmd, timeout, solver_timed_out);
	if (solver_timed_out) {
		timed_out = true;
		if (!g_time_ilp) {
			cout << "TIMEOUT" << endl;
			fail(solver_name + " timed out after " + to_string(timeout) + " seconds");
		}
		return Extraction();
	}
	string solver_log;
	{
		ifstream log_in(log_path.c_str(), ios::binary);
		stringstream buffer;
		buffer << log_in.rdbuf();
		solver_log = buffer.str();
	}
	{
		ifstream in_debug_log(log_path.c_str(), ios::binary);
		ofstream out_debug_log("/tmp/tiger_last_extract.log", ios::binary);
		out_debug_log << in_debug_log.rdbuf();
	}
	if (ret != 0) {
		cerr << solver_name << " log output:\n" << solver_log << endl;
		fail(solver_name + " invocation failed");
	}
	if (solver_log.find("ERROR") != string::npos || solver_log.find("Error") != string::npos) {
		cerr << solver_name << " log output:\n" << solver_log << endl;
		fail(solver_name + " reported an error while solving");
	}
	{
		ifstream in_debug_sol(sol_path.c_str(), ios::binary);
		ofstream out_debug_sol("/tmp/tiger_last_extract.sol", ios::binary);
		out_debug_sol << in_debug_sol.rdbuf();
	}
	auto fail_with_log = [&](const string &msg) {
		cerr << solver_name << " log output:\n" << solver_log << endl;
		fail(msg);
	};
	SolverSolution solver_solution = parse_solver_solution(sol_path, solver_log, solver_name, g_use_gurobi, fail_with_log);
	const unordered_map<string, double> &values = solver_solution.values;
	bool infeasible = solver_solution.infeasible;
	unordered_map<string, bool> value_map = build_binary_value_map(pickNode, choices, values);
	if (infeasible) {
		cout << "infeasible" << endl;
		// try the old extraction method for debugging
		StateWalk sw = UnguidedFindStateWalk(g, initc, initn, root, nsubregion);

		// does this state walk use a node multiple times?
		auto encode_node = [](EClassId cls, ENodeId node) -> long long {
			return (static_cast<long long>(cls) << 32) |
			       static_cast<unsigned long long>(static_cast<unsigned int>(node));
		};
		unordered_set<long long> used_nodes;
		for (const auto &p : sw) {
			if (used_nodes.count(encode_node(p.first, p.second))) {
				cerr << "state walk reuses node " << p.second << endl;

				// print out the eclass of the reused node
				cerr << "in eclass " << enode_to_eclass(g, p.second) << " which has enodes:" << endl;
				print_eclass(cerr, g, enode_to_eclass(g, p.second));
				exit(1);
			}
			used_nodes.insert(encode_node(p.first, p.second));
		}

		regionExtractionWithStateWalk(g, root, sw);
		fail(solver_name + " reported infeasibility");
	}

	vector<vector<int> > pickSelected(g.eclasses.size());
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		pickSelected[c].assign(g.eclasses[c].enodes.size(), 0);
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			bool selected = require_binary_value(value_map, pickNode[c][n], fail);
			if (selected) {
				pickSelected[c][n] = 1;
			}
		}
	}
	bool saw_root_assignment = false;
	if (!g.eclasses[root].enodes.empty()) {
		cerr << "ILP root diagnostics (class " << root << "):\n";
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[root].enodes.size(); ++n) {
			bool root_selected = require_binary_value(value_map, pickNode[root][n], fail);
			double root_value = root_selected ? 1.0 : 0.0;
			if (values.count(pickNode[root][n])) {
				saw_root_assignment = true;
			}
			cerr << "  " << pickNode[root][n] << " = " << root_value
			     << " (" << (pickSelected[root][n] ? "selected" : "not selected") << ")\n";
		}
	}
	if (!saw_root_assignment) {
		cerr << solver_name << " log output:\n" << solver_log << endl;
		fail("solution file did not contain root variable assignments");
	}

	if (pickSelected[root].empty()) {
		fail("root eclass has no selected enode");
	}
	vector<ENodeId> root_enodes;
	for (ENodeId n = 0; n < (ENodeId)pickSelected[root].size(); ++n) {
		if (pickSelected[root][n]) {
			root_enodes.push_back(n);
		}
	}
	if (root_enodes.empty()) {
		fail("no root enode selected");
	}
	/*if (!pickSelected[initc].empty() && !pickSelected[initc][initn]) {
		fail("init enode not selected");
	}*/

	vector<vector<vector<ENodeId> > > childSelection(g.eclasses.size());
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		childSelection[c].resize(g.eclasses[c].enodes.size());
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			childSelection[c][n].assign(g.eclasses[c].enodes[n].ch.size(), -1);
		}
	}

	build_child_selection_for_roots(g, root, root_enodes, pickSelected, choiceIndex,
	                                choices, pickNode, value_map, fail, childSelection);

	vector<ExtractionENode> extraction;
	unordered_map<long long, ExtractionENodeId> nodeIndex;
	unordered_set<long long> usedEffectful;
	unordered_set<long long> visiting;

	for (ENodeId root_node : root_enodes) {
		build_extraction_node(g, childSelection, root, root_node,
								extraction, nodeIndex, usedEffectful, visiting, fail);
	}
	if (extraction.empty()) {
		fail("extraction is empty");
	}
	if (!validExtraction(g, root, extraction)) {
		fail("constructed extraction is invalid");
	}
	return extraction;

}
