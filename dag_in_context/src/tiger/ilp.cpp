#include "ilp.h"
#include "main.h"
#include "greedy.h"

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

#include<algorithm>
#include<climits>
#include<cstring>
#include<cstdio>
#include<fstream>
#include<iostream>
#include<queue>
#include<utility>

using namespace std;

pair<EClassId, ENodeId> findArg(const EGraph &g);
bool validExtraction(const EGraph &g, const EClassId root, const Extraction &e);

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
	bool log_mentions_infeasible = contains_case_insensitive(solver_log, "infeasible");
	if (!has_content) {
		if (log_mentions_infeasible) {
			result.infeasible = true;
			return result;
		}
		if (!result.infeasible) {
			fail_with_log(string("") + solver_name + " produced an empty solution file");
		}
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
	if (result.values.empty() && log_mentions_infeasible) {
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

Extraction extractRegionILPInner(const EGraph &g, const EClassId root, const vector<vector<Cost> > &rstatewalk_cost, bool &timed_out, bool &infeasible, size_t *edge_variable_count)  {
	auto fail = [&](const string &msg) -> void {
		cerr << "ILP extraction error: " << msg << endl;
		exit(1);
	};
	timed_out = false;
	infeasible = false;

	pair<EClassId, ENodeId> arg = findArg(g);
	EClassId initc = arg.first;
	ENodeId initn = arg.second;

	/*
	if (root == initc) {
		StateWalk sw;
		sw.push_back(make_pair(root, initn));
		return regionExtractionWithStateWalk(g, root, sw).second;
	}
	*/

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
			const ENode &en = g.eclasses[c].enodes[n];
			Cost node_cost = get_enode_cost(en);
			if (g.eclasses[c].isEffectful) {
				if (static_cast<size_t>(c) >= rstatewalk_cost.size()) {
					fail(string("statewalk cost missing for effectful eclass ") + to_string(c));
				}
				if (static_cast<size_t>(n) >= rstatewalk_cost[c].size()) {
					fail(string("statewalk cost missing for effectful enode ") + to_string(c) + ":" + to_string(n));
				}
				node_cost = rstatewalk_cost[c][n];
			}
			const Cost bounded_cost = min(node_cost, static_cast<Cost>(LLONG_MAX));
			pickCost[c][n] = static_cast<long long>(bounded_cost);
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
	if (g_config.ilp_minimize_objective) {
		lp << " obj:";
		int term_count = 0;
		for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
			for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
				if (!firstTerm) {
					lp << " + ";
				} else {
					lp << " ";
					firstTerm = false;
				}
				lp << pickCost[c][n] << " " << pickNode[c][n];
				++term_count;
				if (term_count % 50 == 0) {
					lp << "\n";
				}
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
	if (edge_variable_count != nullptr) {
		*edge_variable_count = choices.size();
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

	string solver_name = g_config.use_gurobi ? "gurobi" : "cbc";
	string cmd = "";
	if (g_config.use_gurobi) {
		cmd = string("gurobi_cl Threads=1 ResultFile=\"") + sol_path + "\" LogFile=\"" + log_path + "\" " + lp_path + " > /dev/null 2>&1";
	} else {
		cmd = string("cbc \"") + lp_path + "\" solve branch solu \"" + sol_path + "\" > \"" + log_path + "\" 2>&1";
	}
	bool solver_timed_out = false;
	int timeout = g_config.ilp_timeout_seconds;
	int ret = run_command_with_timeout(cmd, timeout, solver_timed_out);
	if (solver_timed_out) {
		timed_out = true;
		if (!g_config.time_ilp) {
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
	SolverSolution solver_solution = parse_solver_solution(sol_path, solver_log, solver_name, g_config.use_gurobi, fail_with_log);
 	const unordered_map<string, double> &values = solver_solution.values;
	if (solver_solution.infeasible) {
		infeasible = true;
		return Extraction();
	}
	unordered_map<string, bool> value_map = build_binary_value_map(pickNode, choices, values);

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
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[root].enodes.size(); ++n) {
			bool root_selected = require_binary_value(value_map, pickNode[root][n], fail);
			double root_value = root_selected ? 1.0 : 0.0;
			if (values.count(pickNode[root][n])) {
				saw_root_assignment = true;
			}
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

struct SubEGraphMap {
	vector<EClassId> eclassmp;
	map<EClassId, EClassId> inv;
	vector<vector<int> > nsubregion;
	vector<vector<ENodeId> > enode_map;
};

// Prune any nodes that refer to empty eclasses
// Then can happen after we prune nodes that have this same subregion as a child.
static void prune_region_egraph(EGraph &g,
					     vector<vector<int> > *nsubregion,
					     vector<vector<ENodeId> > *enode_map) {
	assert(nsubregion != nullptr);
	assert(enode_map != nullptr);
	assert(nsubregion->size() == g.eclasses.size());
	assert(enode_map->size() == g.eclasses.size());

	vector<bool> empty(g.eclasses.size(), false);
	for (size_t i = 0; i < g.eclasses.size(); ++i) {
		empty[i] = g.eclasses[i].enodes.empty();
	}

	bool changed = true;
	while (changed) {
		changed = false;
		for (size_t i = 0; i < g.eclasses.size(); ++i) {
			vector<ENode> &enodes = g.eclasses[i].enodes;
			vector<int> &subregion_counts = (*nsubregion)[i];
			vector<ENodeId> &orig_enode_ids = (*enode_map)[i];
			assert(subregion_counts.size() == enodes.size());
			assert(orig_enode_ids.size() == enodes.size());

			size_t write_idx = 0;
			for (size_t j = 0; j < enodes.size(); ++j) {
				const ENode &node = enodes[j];
				bool prune = false;
				for (EClassId child : node.ch) {
					if (child < 0 || child >= static_cast<EClassId>(g.eclasses.size()) || empty[child]) {
						prune = true;
						break;
					}
				}
				if (!prune) {
					if (write_idx != j) {
						enodes[write_idx] = node;
						subregion_counts[write_idx] = subregion_counts[j];
						orig_enode_ids[write_idx] = orig_enode_ids[j];
					}
					++write_idx;
				} else {
					changed = true;
				}
			}
			if (write_idx != enodes.size()) {
				enodes.resize(write_idx);
				subregion_counts.resize(write_idx);
				orig_enode_ids.resize(write_idx);
			}
			if (!empty[i] && enodes.empty()) {
				empty[i] = true;
				changed = true;
			}
		}
	}
}

static EClassId get_inv_entry_or_fail(const map<EClassId, EClassId> &inv,
									 		 EClassId key,
									 		 const char *context) {
	auto it = inv.find(key);
	if (it == inv.end()) {
		cerr << "Error: SubEGraphMap missing mapping for eclass " << key;
		if (context != nullptr) {
			cerr << " while " << context;
		}
		cerr << endl;
		exit(1);
	}
	return it->second;
}

static bool should_skip_region_enode(const EGraph &g,
								 const EClassId parent_eclass,
								 const ENode &enode,
								 const EClassId region_root) {
	if (!g.eclasses[parent_eclass].isEffectful) {
		return false;
	}
	bool saw_effectful_child = false;
	for (EClassId child : enode.ch) {
		if (!g.eclasses[child].isEffectful) {
			continue;
		}
		if (saw_effectful_child && child == region_root) {
			return true;
		}
		saw_effectful_child = true;
	}
	return false;
}

pair<EGraph, SubEGraphMap> createRegionEGraph(const EGraph &g, const EClassId region_root) {
	SubEGraphMap mp;
	queue<EClassId> worklist;
	auto enqueue = [&](EClassId c) {
		if (mp.inv.count(c)) {
			return;
		}
		mp.inv[c] = mp.eclassmp.size();
		mp.eclassmp.push_back(c);
		mp.nsubregion.push_back(vector<int>(g.eclasses[c].enodes.size(), 0));
		mp.enode_map.push_back(vector<ENodeId>());
		worklist.push(c);
	};

	enqueue(region_root);
	while (!worklist.empty()) {
		EClassId u = worklist.front();
		worklist.pop();
		EClassId u_idx = get_inv_entry_or_fail(mp.inv, u, "accessing nsubregion");
		assert(mp.nsubregion[u_idx].size() == g.eclasses[u].enodes.size());
		bool parent_effectful = g.eclasses[u].isEffectful;
		for (int i = 0; i < (int)g.eclasses[u].enodes.size(); ++i) {
			bool saw_effectful_child = false;
			for (int j = 0; j < (int)g.eclasses[u].enodes[i].ch.size(); ++j) {
				EClassId v = g.eclasses[u].enodes[i].ch[j];
				if (g.eclasses[v].isEffectful) {
					if (saw_effectful_child) {
						if (parent_effectful) {
							mp.nsubregion[u_idx][i]++;
						}
						continue;
					}
					saw_effectful_child = true;
				}
				enqueue(v);
			}
		}
	}

	EGraph gr;
	for (int i = 0; i < (int)mp.eclassmp.size(); ++i) {
		EClass c;
		c.isEffectful = g.eclasses[mp.eclassmp[i]].isEffectful;
		const auto &orig_enodes = g.eclasses[mp.eclassmp[i]].enodes;
		vector<int> filtered_nsubregion;
		vector<ENodeId> filtered_enode_map;
		for (int j = 0; j < (int)orig_enodes.size(); ++j) {
			const ENode &orig_node = orig_enodes[j];
			if (should_skip_region_enode(g, mp.eclassmp[i], orig_node, region_root)) {
				continue;
			}
			ENode node;
			node.eclass = i;
			node.head = orig_node.head;
			bool subregionchild = false;
			for (int k = 0; k < (int)orig_node.ch.size(); ++k) {
				EClassId cp = orig_node.ch[k];
				if (g.eclasses[cp].isEffectful) {
					if (subregionchild) {
						continue;
					}
					subregionchild = true;
				}
				node.ch.push_back(get_inv_entry_or_fail(mp.inv, cp, "building region egraph"));
			}
			c.enodes.push_back(node);
			filtered_nsubregion.push_back(mp.nsubregion[i][j]);
			filtered_enode_map.push_back(j);
		}
		assert(filtered_nsubregion.size() == c.enodes.size());
		assert(filtered_enode_map.size() == c.enodes.size());
		mp.nsubregion[i] = filtered_nsubregion;
		mp.enode_map[i] = filtered_enode_map;
		gr.eclasses.push_back(c);
	}

	prune_region_egraph(gr, &mp.nsubregion, &mp.enode_map);
	EClassId root_idx = get_inv_entry_or_fail(mp.inv, region_root, "getting region root mapping after pruning region egraph");
	if (gr.eclasses[root_idx].enodes.empty()) {
		cerr << "Error: Region root eclass " << region_root
		     << " became empty after pruning invalid enodes." << endl;
		exit(1);
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

typedef int RegionId;

static pair<Extraction, std::chrono::nanoseconds> run_ilp_extractor(const EGraph &g, const EClassId root, const vector<vector<Cost> > &rstatewalk_cost, bool &timed_out, bool &infeasible, size_t *edge_variable_count) {
	auto start = std::chrono::steady_clock::now();
	timed_out = false;
	infeasible = false;
	Extraction extraction = extractRegionILPInner(g, root, rstatewalk_cost, timed_out, infeasible, edge_variable_count);
	auto elapsed = std::chrono::steady_clock::now() - start;
	return make_pair(extraction, std::chrono::duration_cast<std::chrono::nanoseconds>(elapsed));
}

long long extract_region_ilp_with_timing(const EGraph &g, EClassId root, const vector<vector<Cost> > &rstatewalk_cost, Extraction &out, bool &timed_out, bool &infeasible, size_t &edge_variable_count) {
	auto result = run_ilp_extractor(g, root, rstatewalk_cost, timed_out, infeasible, &edge_variable_count);
	out = std::move(result.first);
	return result.second.count();
}

// the main function for getting a linear extraction from a region
// this uses ILP in ilp mode or the unguided statewalk search in the normal mode
Extraction extractRegionILP(const EGraph &g, const EClassId root, const vector<vector<Cost> > &rstatewalk_cost) {
	bool ilp_timed_out = false;
	bool ilp_infeasible = false;
	size_t edge_variable_count = 0;
	auto ilp_result = run_ilp_extractor(g, root, rstatewalk_cost, ilp_timed_out, ilp_infeasible, &edge_variable_count);
	if (ilp_timed_out) {
		cout << "TIMEOUT" << endl;
		std::exit(1);
	}
	if (ilp_infeasible) {
		cerr << "ILP solver reported infeasibility" << endl;
		std::exit(1);
	}
	return ilp_result.first;
}

ExtractionENodeId reconstructExtraction(const EGraph &g,
						 const vector<EClassId> &region_roots,
						 const vector<RegionId> &region_root_id,
						 vector<ExtractionENodeId> &extracted_roots,
						 Extraction &e,
						 const RegionId &cur_region,
						 const vector<vector<Cost> > &statewalk_cost) {
	if (extracted_roots[cur_region] != -1) {
		return extracted_roots[cur_region];
	}
	EClassId region_root = region_roots[cur_region];
	//cout << cur_region << " Region root : " << region_root << endl;
	pair<EGraph, SubEGraphMap> res = createRegionEGraph(g, region_root);
	EGraph &gr = res.first;
	SubEGraphMap &rmap = res.second;
	EClassId root = get_inv_entry_or_fail(rmap.inv, region_root, "getting region root mapping");
	EGraphMapping region_to_global;
	region_to_global.eclassidmp = rmap.eclassmp;
	region_to_global.enodeidmp = rmap.enode_map;
	vector<vector<Cost> > rstatewalk_cost = project_statewalk_cost(region_to_global, statewalk_cost);
	Extraction er = extractRegionILP(gr, root, rstatewalk_cost);
	Extraction ner(er.size());
	for (int i = 0; i < (int)er.size(); ++i) {
		ExtractionENode &en = er[i], &nen = ner[i];
		EClassId oric = rmap.eclassmp[en.c];
		nen.c = oric;
		assert(en.c < (int)rmap.enode_map.size());
		assert(en.n < (int)rmap.enode_map[en.c].size());
		ENodeId orin = rmap.enode_map[en.c][en.n];
		nen.n = orin;
		bool subregionchild = false;
		for (int j = 0, k = 0; j < (int)g.eclasses[oric].enodes[orin].ch.size(); ++j) {
			EClassId orichc = g.eclasses[oric].enodes[orin].ch[j];
			if (g.eclasses[orichc].isEffectful) {
				if (subregionchild) {
					nen.ch.push_back(reconstructExtraction(g, region_roots, region_root_id, extracted_roots, e, region_root_id[orichc], statewalk_cost));
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

vector<Extraction> extractAllILP(EGraph g, vector<EClassId> fun_roots) {
	vector<vector<Cost> > statewalk_cost = compute_statewalk_cost(g);
	vector<Extraction> ret;
	for (int _ = 0; _ < (int)fun_roots.size(); ++_) {
		EClassId fun_root = fun_roots[_];
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
		reconstructExtraction(g, region_roots, region_root_id, extracted_roots, e, region_root_id[fun_root], statewalk_cost);
		assert(linearExtraction(g, fun_root, e));
		ret.push_back(e);
	}
	//write_extract_region_timings();
	return ret;
}