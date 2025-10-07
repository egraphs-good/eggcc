#include<vector>
#include<cstdio>
#include<climits>
#include<cassert>
#include<cstring>
#include<string>
#include<iostream>
#include<algorithm>

using namespace std;

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

void json_print_egraph(const EGraph &g) {
    vector<string> name_for_eclass;
    for (int i = 0; i < (int)g.eclasses.size(); ++i) {
        const ENode &n = g.eclasses[i].enodes[0];
        string name;
        int pos = n.head.find("###");
        name = n.head.substr(0, pos);
        name_for_eclass.push_back(name);
    }

    printf("{\n");
    printf("\t\"nodes\": {\n");
    bool isfirst = true;
    for (int i = 0; i < (int)g.eclasses.size(); ++i) {
        for (int j = 0; j < (int)g.eclasses[i].enodes.size(); ++j) {
            const ENode &n = g.eclasses[i].enodes[j];
            string name, op;
            int pos = n.head.find("###");
            name = n.head.substr(0, pos);
            op = n.head.substr(pos + 3, n.head.length() - pos - 3);
            if (!isfirst) {
                printf("\t\t},\n");
            }
            isfirst = false;
            printf("\t\t\"%s\": {\n", name.c_str());
            printf("\t\t\t\"op\": \"%s\",\n", op.c_str());
            printf("\t\t\t\"children\": [\n");
            for (int k = 0; k < (int)n.ch.size(); ++k) {
                printf("\t\t\t\t\"%s\"%s\n", name_for_eclass[n.ch[k]].c_str(), k == (int)n.ch.size() - 1 ? "" : ",");
            }
            printf("\t\t\t],\n");
            printf("\t\t\t\"eclass\": \"%d\"\n", i);
        }
    }
    isfirst = true;
    printf("\t\t}\n");
    printf("\t},\n");
    printf("\t\"class_data\": {\n");
    for (int i = 0; i < (int)g.eclasses.size(); ++i) {
        if (!isfirst) {
            printf("\t\t},\n");
        }
        isfirst = false;
        printf("\t\t\"%d\": {\n", i);
        printf("\t\t\t\"type\": \"%s\"\n", g.eclasses[i].isEffectful ? "Effectful" : "Pure");
    }
    printf("\t\t}\n");
    printf("\t}\n");
    printf("}\n");
}

int main() {
	FILE* ppin = preprocessing();
	EGraph g = read_egraph(ppin);
    json_print_egraph(g);    
    return 0;
}