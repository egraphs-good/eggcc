#include "toegglog.h"

#include <cassert>

void print_egg_prologue() {
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
	for (ExtractionENodeId i = 0; i < (ExtractionENodeId)e.size(); ++i) {
        const ENode &n = g.eclasses[e[i].c].enodes[e[i].n];
		string name = n.get_name(), op = n.get_op();
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

void print_egg_epilogue() {
	printf("(run reconstruction 1)\n");
}

void output_egglog(const EGraph &g, const vector<Extraction> &es) {
    print_egg_prologue();
    for (size_t i = 0; i < es.size(); ++i) {
        print_egg_extraction(g, es[i]);
    }
    print_egg_epilogue();
}
