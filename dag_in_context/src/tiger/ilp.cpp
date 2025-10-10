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

void print_eclass(ostream &out, const EGraph &g, EClassId c) {
	out << "EClass " << c << (g.eclasses[c].isEffectful ? " (effectful)" : "") << ":\n";
	for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
		out << "  ";
		print_enode(out, g.eclasses[c].enodes[n]);
		out << "\n";
	}
}

Extraction extractRegionILP(const EGraph &g, const EClassId initc, const ENodeId initn, const EClassId root, const vector<vector<int> > &nsubregion)  {
	auto fail = [&](const string &msg) -> void {
		cerr << "ILP extraction error: " << msg << endl;
		exit(1);
	};

	if (root == initc) {
		StateWalk sw;
		sw.push_back(make_pair(root, initn));
		return regionExtractionWithStateWalk(g, root, sw).second;
	}

	struct ChoiceVar {
		string name;
		EClassId parent_class;
		ENodeId parent_node;
		int child_idx;
		EClassId child_class;
		ENodeId child_node;
	};

	// Picking an enode in an eclass
	vector<vector<string> > pickVar(g.eclasses.size());
	// Cost of picking an enode in an eclass
	vector<vector<long long> > pickCost(g.eclasses.size());
	// Choosing an eclass, enode, child index, and child enode index
	vector<vector<vector<vector<int> > > > choiceIndex(g.eclasses.size());
	// For each child enode, which choice variables point to it
	vector<vector<vector<int> > > childParents(g.eclasses.size());
	// Order variables for acyclicity
	vector<vector<string> > orderVar(g.eclasses.size());
	// Effectful child flow tracking
	vector<vector<int> > effectOutgoing(g.eclasses.size());
	vector<vector<int> > effectIncoming(g.eclasses.size());

	int total_enodes = 0;
	for (const EClass &ec : g.eclasses) {
		total_enodes += ec.enodes.size();
	}
	int maxOrder = max(1, total_enodes);
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		pickVar[c].resize(g.eclasses[c].enodes.size());
		pickCost[c].resize(g.eclasses[c].enodes.size());
		choiceIndex[c].resize(g.eclasses[c].enodes.size());
		childParents[c].resize(g.eclasses[c].enodes.size());
		orderVar[c].resize(g.eclasses[c].enodes.size());
	}

	// All choice variables (a partiacular edge between an enode at a child index and another enode)
	vector<ChoiceVar> choices;
	// initialize choices, pickVar, pickCost, choiceIndex, childParents
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			pickVar[c][n] = string("p_") + to_string(c) + "_" + to_string(n);
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
	// minimize sum pickCost[c][n] * pickVar[c][n]
	lp << "Minimize\n obj:";
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			if (!firstTerm) {
				lp << " +";
			}
			firstTerm = false;
			lp << " " << pickCost[c][n] << " " << pickVar[c][n];
		}
	}
	if (firstTerm) {
		lp << " 0";
	}
	lp << "\nSubject To\n";

	// Require at least one root enode to be picked
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		if (g.eclasses[c].enodes.empty()) {
			fail("encountered eclass with no enodes");
		}
		if (c != root) {
			continue;
		}
		lp << " pick_sum_" << c << ":";
		bool first = true;
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			lp << (first ? " " : " + ") << pickVar[c][n];
			first = false;
		}
		lp << " >= 1\n";
	}

	// Pick at least one child per child index of a picked enode
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
				}
				lp << " >= 1\n";
			}
		}
	}
	
	// If you choose a child edge, you must pick that enode.
	for (int idx = 0; idx < (int)choices.size(); ++idx) {
		const ChoiceVar &cv = choices[idx];
		lp << " child_link_" << idx << ": " << cv.name << " - " << pickVar[cv.child_class][cv.child_node] << " <= 0\n";
	} 

	// Linearity: effectful enodes may not be targeted by multiple effectful parents.
	/*for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		if (!g.eclasses[c].isEffectful) {
			continue;
		}
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			const vector<int> &parents = childParents[c][n];
			if (parents.empty()) {
				continue;
			}
			lp << " child_unique_" << c << '_' << n << ":";
			bool first = true;
			for (int idx : parents) {
        if (g.eclasses[choices[idx].parent_class].isEffectful) {
				  lp << (first ? " " : " + ") << choices[idx].name;
				  first = false;
        }
			}
      if (first) {
        lp << " 0";
      }
			lp << " <= 1\n";
		}
	}*/

	// Order variables must decrease along chosen edges to prevent cycles.
	// When parent and child are the same enode, instead forbid it (preventing duplicate entries)
	for (int idx = 0; idx < (int)choices.size(); ++idx) {
		const ChoiceVar &cv = choices[idx];
		if (cv.parent_class == cv.child_class && cv.parent_node == cv.child_node) {
			lp << " order_edge_" << idx << ": " << cv.name
			   << " <= " << -1 << "\n";
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
			lp << " 0 <= " << orderVar[c][n] << " <= " << maxOrder << "\n";
		}
	}

	lp << "Binary\n";
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			lp << " " << pickVar[c][n] << "\n";
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

	string cmd = string("cbc \"") + lp_path + "\" solve branch solu \"" + sol_path + "\" > \"" + log_path + "\" 2>&1";
	int ret = system(cmd.c_str());
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
		cerr << "cbc log output:\n" << solver_log << endl;
		fail("cbc invocation failed");
	}
	if (solver_log.find("ERROR") != string::npos || solver_log.find("Error") != string::npos) {
		cerr << "cbc log output:\n" << solver_log << endl;
		fail("cbc reported an error while solving");
	}

	ifstream sol(sol_path.c_str());
	if (!sol.good()) {
		fail("failed to open solution file");
	}
	if (sol.peek() == ifstream::traits_type::eof()) {
		cerr << "cbc log output:\n" << solver_log << endl;
		fail("cbc produced an empty solution file");
	}
	{
		ifstream in_debug_sol(sol_path.c_str(), ios::binary);
		ofstream out_debug_sol("/tmp/tiger_last_extract.sol", ios::binary);
		out_debug_sol << in_debug_sol.rdbuf();
	}
	string line;
	unordered_map<string, double> values;
	bool infeasible = false;
	while (getline(sol, line)) {
		if (line.find("Infeasible") != string::npos || line.find("infeasible") != string::npos) {
			infeasible = true;
			break;
		}
		stringstream ss(line);
		vector<string> tokens;
		string tok;
		while (ss >> tok) {
			tokens.push_back(tok);
		}
		if (tokens.size() >= 2 && isalpha(tokens[0][0])) {
			try {
				double value = stod(tokens[1]);
				values[tokens[0]] = value;
			} catch (...) {
				continue;
			}
		} else if (tokens.size() >= 3) {
			try {
				double value = stod(tokens[2]);
				values[tokens[1]] = value;
			} catch (...) {
				continue;
			}
		}
	}
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
			cerr << "visiting node " << p.second << " in eclass " << p.first << endl;
			print_enode(cerr, g.eclasses[p.first].enodes[p.second]);
			cerr << endl;

			if (used_nodes.count(encode_node(p.first, p.second))) {
				cerr << "state walk reuses node " << p.second << endl;

				// print out the eclass of the reused node
				cerr << "in eclass " << enode_to_eclass(g, p.second) << " which has enodes:" << endl;
				print_eclass(cerr, g, enode_to_eclass(g, p.second));

				// print out effectful child enodes of the reused node
				const ENode &en = g.eclasses[enode_to_eclass(g, p.second)].enodes[0];
				cerr << "which has effectful children:" << endl;
				print_eclass(cerr, g, en.ch[0]);

				exit(1);
			}
			used_nodes.insert(encode_node(p.first, p.second));
		}

		regionExtractionWithStateWalk(g, root, sw);
		fail("cbc reported infeasibility");
	}

	vector<vector<int> > pickSelected(g.eclasses.size());
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		pickSelected[c].assign(g.eclasses[c].enodes.size(), 0);
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			double v = values.count(pickVar[c][n]) ? values[pickVar[c][n]] : 0.0;
			if (v > 0.5) {
				pickSelected[c][n] = 1;
			}
		}
	}
	bool saw_root_assignment = false;
	if (!g.eclasses[root].enodes.empty()) {
		cerr << "ILP root diagnostics (class " << root << "):\n";
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[root].enodes.size(); ++n) {
			double root_value = values.count(pickVar[root][n]) ? values[pickVar[root][n]] : 0.0;
			if (values.count(pickVar[root][n])) {
				saw_root_assignment = true;
			}
			cerr << "  " << pickVar[root][n] << " = " << root_value
			     << " (" << (pickSelected[root][n] ? "selected" : "not selected") << ")\n";
		}
	}
	if (!saw_root_assignment) {
		cerr << "cbc log output:\n" << solver_log << endl;
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

	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			const vector<vector<int> > &idx_lists = choiceIndex[c][n];
			for (int child_idx = 0; child_idx < (int)idx_lists.size(); ++child_idx) {
				const vector<int> &list = idx_lists[child_idx];
				for (int idx : list) {
					double v = values.count(choices[idx].name) ? values[choices[idx].name] : 0.0;
					if (v > 0.5) {
						if (childSelection[c][n][child_idx] != -1) {
							fail("multiple child selections detected for a single child");
						}
						childSelection[c][n][child_idx] = choices[idx].child_node;
					}
				}
			}
		}
	}

	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			if (!pickSelected[c][n]) {
				continue;
			}
			for (int child_idx = 0; child_idx < (int)childSelection[c][n].size(); ++child_idx) {
				int child_enode = childSelection[c][n][child_idx];
				if (child_enode == -1) {
					cerr << "Missing child selection for eclass " << c << " node " << n
					     << " child index " << child_idx << " options:";
					for (int idx : choiceIndex[c][n][child_idx]) {
						cerr << ' ' << choices[idx].name << "="
						     << (values.count(choices[idx].name) ? values[choices[idx].name] : 0.0);
					}
					cerr << " (pickVar=" << (values.count(pickVar[c][n]) ? values[pickVar[c][n]] : 0.0)
					     << ")" << endl;
					fail("missing child selection for picked enode");
				}
				EClassId child_class = g.eclasses[c].enodes[n].ch[child_idx];
				if (child_enode < 0 || child_enode >= (ENodeId)g.eclasses[child_class].enodes.size()) {
					fail("child selection index out of bounds");
				}
				if (!pickSelected[child_class][child_enode]) {
					fail("child enode not marked as picked");
				}
			}
		}
	}
	cerr << "Selected parent/child edges:\n";
	for (EClassId c = 0; c < (EClassId)g.eclasses.size(); ++c) {
		for (ENodeId n = 0; n < (ENodeId)g.eclasses[c].enodes.size(); ++n) {
			if (!pickSelected[c][n]) {
				continue;
			}
			const ENode &en = g.eclasses[c].enodes[n];
			cerr << "  eclass " << c << " node " << n << " (" << en.head << ") ->";
			for (int child_i = 0; child_i < (int)en.ch.size(); ++child_i) {
				EClassId child_class = en.ch[child_i];
				ENodeId child_node = childSelection[c][n][child_i];
				cerr << " (" << child_class << "," << child_node << ")";
			}
			cerr << "\n";
		}
	}

	vector<ExtractionENode> extraction;
	unordered_map<long long, ExtractionENodeId> nodeIndex;
	unordered_set<long long> usedEffectful;
	unordered_set<long long> visiting;
	function<ExtractionENodeId(EClassId, ENodeId)> build = [&](EClassId c, ENodeId n) -> ExtractionENodeId {
		long long key = (static_cast<long long>(c) << 32) | static_cast<unsigned long long>(static_cast<unsigned int>(n));
		
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
			ExtractionENodeId child_ex = build(child_class, child_node);
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
	};

	for (ENodeId root_node : root_enodes) {
		build(root, root_node);
	}
	if (extraction.empty()) {
		fail("extraction is empty");
	}
	if (!validExtraction(g, root, extraction)) {
		fail("constructed extraction is invalid");
	}
	return extraction;

}
