// Port of toegglog.h / toegglog.cpp — emit extraction as egglog program.
// Direct line-by-line translation.

use crate::egraphin::{EGraph, ENode, Extraction, ExtractionENodeId};

fn print_egg_prologue() {
    let schema: &str =
"
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

(datatype Assumption    \n  (DumC)
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
  (FGreaterThan) \n  (FLessEq)
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
";
    print!("{}", schema);
}

fn print_egg_extraction(g: &EGraph, e: &Extraction) {
    static mut FUNID: i32 = 0;
    static mut CNT: i32 = 0;
    // SAFETY: mirrors C++ `static int funid = 0, cnt = 0;` inside the function body.
    let funid_ptr: *mut i32 = &raw mut FUNID;
    let cnt_ptr: *mut i32 = &raw mut CNT;
    unsafe { *funid_ptr += 1; }
    print!("; Function #{}\n", unsafe { *funid_ptr });
    print!("(rule () (\n");
    let mut var: Vec<String> = vec![String::new(); e.len()];
    for i in 0..(e.len() as ExtractionENodeId) {
        let n: &ENode = &g.eclasses[e[i as usize].c as usize].enodes[e[i as usize].n as usize];
        let name: String = n.get_name();
        let op: String = n.get_op();
        if name.len() > 9 && &name[0..9] == "primitive" {
            if op.as_bytes()[0] == b'\\' {
                var[i as usize] = op[1..op.len() - 2].to_string() + "\"";
            } else {
                var[i as usize] = op.clone();
            }
        } else {
            let curvar: String = String::from("__tmp") + &unsafe { *cnt_ptr }.to_string();
            unsafe { *cnt_ptr += 1; }
            var[i as usize] = curvar.clone();
            print!("\t(let {} (", curvar);
            if op == "Arg" {
                assert!(e[i as usize].ch.len() == 0);
                print!("Arg DumT (DumC)");
            } else if op == "Const" {
                assert!(e[i as usize].ch.len() == 1);
                print!("Const {} DumT (DumC)", var[e[i as usize].ch[0] as usize]);
            } else if op == "Empty" {
                assert!(e[i as usize].ch.len() == 0);
                print!("Empty DumT (DumC)");
            } else {
                print!("{}", op);
                for j in 0..(e[i as usize].ch.len() as i32) {
                    print!(" {}", var[e[i as usize].ch[j as usize] as usize]);
                }
            }
            print!("))\n");
        }
    }
    print!(") :ruleset reconstruction)\n");
}

fn print_egg_epilogue() {
    print!("(run reconstruction 1)\n");
}

pub fn output_egglog(g: &EGraph, es: &Vec<Extraction>) {
    print_egg_prologue();
    for i in 0..es.len() {
        print_egg_extraction(g, &es[i]);
    }
    print_egg_epilogue();
}
