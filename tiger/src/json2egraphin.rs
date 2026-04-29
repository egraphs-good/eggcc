// Port of json2egraphin.h / json2egraphin.cpp — egglog JSON parser.
// Direct line-by-line translation of the hand-rolled tokenizer.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::Read;

use indexmap::IndexMap;

use crate::egraphin::{
    prune_unextractable_enodes, EClass, EClassId, EGraph, ENode, ENodeId,
};

// is_wellformed_egraph lives in debug.rs in this Rust port.
#[cfg(feature = "debug")]
use crate::debug::is_wellformed_egraph;

// queue<string> tokenbuf;
thread_local! {
    static TOKENBUF: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
}

// scanf("%s", buf) — read whitespace-delimited token from stdin.
// Mirrors the static char buf[505]; scanf("%s", buf); return buf;
thread_local! {
    static STDIN_READER: RefCell<StdinReader> = RefCell::new(StdinReader::new());
}

struct StdinReader {
    buf: Vec<u8>,
    pos: usize,
}

impl StdinReader {
    fn new() -> Self {
        // Slurp all of stdin once. Mirrors C++ scanf's internal stdio buffering,
        // but avoids the pthread_mutex_lock/unlock that std::io::Stdin::lock()
        // performs on every read — that lock dominated the JSON parse profile.
        let mut buf = Vec::new();
        let _ = std::io::stdin().lock().read_to_end(&mut buf);
        StdinReader { buf, pos: 0 }
    }

    #[inline]
    fn read_byte(&mut self) -> Option<u8> {
        if self.pos < self.buf.len() {
            let b = self.buf[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }
}

#[inline]
pub fn read_string() -> String {
    // 1. Skip ASCII whitespace bytes
    // 2. Read non-whitespace bytes into a String
    // 3. Return it
    STDIN_READER.with(|r| {
        let mut r = r.borrow_mut();
        // Most tokens are short identifiers / small numbers; pre-allocating avoids
        // the realloc churn (0→4→8→16) that showed up under `RawVec::grow_one`.
        let mut s = String::with_capacity(32);
        // skip whitespace
        let mut b;
        loop {
            match r.read_byte() {
                Some(c) if (c as char).is_ascii_whitespace() => continue,
                Some(c) => { b = c; break; }
                None => return s, // EOF -> empty string
            }
        }
        // accumulate non-whitespace
        loop {
            s.push(b as char);
            match r.read_byte() {
                Some(c) if (c as char).is_ascii_whitespace() => break,
                Some(c) => { b = c; }
                None => break,
            }
        }
        s
    })
}

pub fn read_next_token() {
    let mut s: String;
    s = read_string();
    if s.as_bytes()[0] == b'{' || s.as_bytes()[0] == b'}' {
        TOKENBUF.with(|b| b.borrow_mut().push_back(s));
    } else if s.as_bytes()[0] == b'\"' {
        if s.as_bytes()[s.len() - 1] == b'\"' && s.as_bytes()[s.len() - 2] != b'\\' {
            let len = s.len();
            TOKENBUF.with(|b| b.borrow_mut().push_back(s[1..1 + (len - 2)].to_string()));
        } else if s.as_bytes()[s.len() - 2] == b'\"' && s.as_bytes()[s.len() - 3] != b'\\' {
            let len = s.len();
            TOKENBUF.with(|b| b.borrow_mut().push_back(s[1..1 + (len - 3)].to_string()));
        } else {
            let mut sb: String;
            sb = read_string();
            s = format!("{}{}{}", s, " ", sb);
            while s.as_bytes()[s.len() - 1] != b'\"' && s.as_bytes()[s.len() - 2] != b'\"' {
                sb = read_string();
                s = format!("{}{}{}", s, " ", sb);
            }
            if s.as_bytes()[s.len() - 1] == b'\"' {
                let len = s.len();
                TOKENBUF.with(|b| b.borrow_mut().push_back(s[1..1 + (len - 2)].to_string()));
            } else if s.as_bytes()[s.len() - 2] == b'\"' {
                let len = s.len();
                TOKENBUF.with(|b| b.borrow_mut().push_back(s[1..1 + (len - 3)].to_string()));
            }
        }
    } else if s.as_bytes()[0] == b'[' {
        if s.len() > 1 && s.as_bytes()[1] == b']' {
            TOKENBUF.with(|b| b.borrow_mut().push_back("[".to_string()));
            TOKENBUF.with(|b| b.borrow_mut().push_back("]".to_string()));
        } else {
            crate::debug_assert_tiger!(s.len() == 1);
            TOKENBUF.with(|b| b.borrow_mut().push_back("[".to_string()));
        }
    } else if s.as_bytes()[0] == b']' {
        TOKENBUF.with(|b| b.borrow_mut().push_back(s));
    } else {
        TOKENBUF.with(|b| b.borrow_mut().push_back(s));
    }
}

pub fn peek_next_token() -> String {
    if TOKENBUF.with(|b| b.borrow().len()) == 0 {
        read_next_token();
    }
    TOKENBUF.with(|b| b.borrow().front().unwrap().clone())
}

pub fn get_next_token() -> String {
    if TOKENBUF.with(|b| b.borrow().len()) == 0 {
        read_next_token();
    }
    // Move out of the queue rather than clone-then-pop. The original C++ uses
    // `tokenbuf.front(); tokenbuf.pop()` which moves; cloning + dropping was
    // an unnecessary malloc/free per token.
    TOKENBUF.with(|b| b.borrow_mut().pop_front().unwrap())
}

#[derive(Clone, Default)]
struct RawENode {
    name: String,
    op: String,
    eclass: String,
    ch: Vec<String>,
}

// vector<vector<RawENode> > raw_egraph;
thread_local! {
    static RAW_EGRAPH: RefCell<Vec<Vec<RawENode>>> = RefCell::new(Vec::new());
}

// unordered_map<string, EClassId> raw_eclassidmp;
thread_local! {
    static RAW_ECLASSIDMP: RefCell<IndexMap<String, EClassId>> = RefCell::new(IndexMap::new());
}

// unordered_map<string, pair<EClassId, ENodeId> > raw_enodeidmp;
thread_local! {
    static RAW_ENODEIDMP: RefCell<IndexMap<String, (EClassId, ENodeId)>> = RefCell::new(IndexMap::new());
}

#[inline]
pub fn consume_next_token(expected: &str) {
    let tmp = get_next_token();
    crate::debug_assert_tiger!(tmp == expected);
    let _ = tmp;
}

#[inline]
pub fn consume_next_token_check_first_char(expected: char) {
    let tmp = get_next_token();
    crate::debug_assert_tiger!(tmp.as_bytes()[0] == expected as u8);
    let _ = tmp;
}

pub fn read_node() {
    let mut e = RawENode::default();
    e.name = get_next_token();
    consume_next_token("{");
    consume_next_token("op");
    e.op = get_next_token();
    consume_next_token("children");
    consume_next_token("[");
    let mut t = get_next_token();
    while t.as_bytes()[0] != b']' {
        e.ch.push(t);
        t = get_next_token();
    }
    consume_next_token("eclass");
    e.eclass = get_next_token();
    let has = RAW_ECLASSIDMP.with(|m| m.borrow().contains_key(&e.eclass));
    if !has {
        let new_id: EClassId = RAW_EGRAPH.with(|g| g.borrow().len()) as EClassId;
        RAW_ECLASSIDMP.with(|m| { m.borrow_mut().insert(e.eclass.clone(), new_id); });
        RAW_EGRAPH.with(|g| { g.borrow_mut().push(Vec::new()); });
    }
    consume_next_token("cost");
    //ignore cost
    get_next_token();
    consume_next_token("subsumed");
    get_next_token();
    consume_next_token_check_first_char('}');
    let ec_id: EClassId = RAW_ECLASSIDMP.with(|m| *m.borrow().get(&e.eclass).unwrap());
    let en_id: ENodeId = RAW_EGRAPH.with(|g| g.borrow()[ec_id as usize].len()) as ENodeId;
    RAW_ENODEIDMP.with(|m| { m.borrow_mut().insert(e.name.clone(), (ec_id, en_id)); });
    RAW_EGRAPH.with(|g| { g.borrow_mut()[ec_id as usize].push(e); });
}

pub fn read_nodes() {
    consume_next_token("{");
    consume_next_token("nodes");
    consume_next_token("{");
    while peek_next_token().as_bytes()[0] != b'}' {
        read_node();
    }
}

pub fn isExpr(i: EClassId) -> bool {
    RAW_EGRAPH.with(|g| {
        let g = g.borrow();
        let ec = &g[i as usize][0].eclass;
        (ec.len() >= 4 && &ec[0..4] == "Expr")
            || (ec.len() >= 8 && &ec[0..8] == "Constant")
            || (ec.len() >= 9 && &ec[0..9] == "TernaryOp")
            || (ec.len() >= 8 && &ec[0..8] == "BinaryOp")
            || (ec.len() >= 7 && &ec[0..7] == "UnaryOp")
    })
}

pub fn isPrimitiveENode(i: EClassId, j: ENodeId) -> bool {
    RAW_EGRAPH.with(|g| {
        let g = g.borrow();
        let name = &g[i as usize][j as usize].name;
        name.len() >= 9 && &name[0..9] == "primitive"
    })
}

pub fn isPrimitiveEClass(i: EClassId) -> bool {
    let n = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
    let mut j: ENodeId = 0;
    while j < n {
        if isPrimitiveENode(i, j) {
            return true;
        }
        j += 1;
    }
    false
}

pub fn find_function_roots() -> Vec<EClassId> {
    let mut ret: Vec<EClassId> = Vec::new();
    let n = RAW_EGRAPH.with(|g| g.borrow().len()) as EClassId;
    for i in 0..n {
        let m = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
        for j in 0..m {
            let op_eq = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].op == "Function");
            if op_eq {
                ret.push(i);
            }
        }
    }
    ret
}

pub fn isType(i: EClassId) -> bool {
    RAW_EGRAPH.with(|g| {
        let g = g.borrow();
        let ec = &g[i as usize][0].eclass;
        (ec.len() >= 4 && &ec[0..4] == "Type")
            || (ec.len() >= 8 && &ec[0..8] == "BaseType")
            || (ec.len() >= 8 && &ec[0..8] == "TypeList")
    })
}

// vector<bool> isEffectfulType;
thread_local! {
    static IS_EFFECTFUL_TYPE: RefCell<Vec<bool>> = RefCell::new(Vec::new());
}

pub fn propagate_effectful_types() {
    let n = RAW_EGRAPH.with(|g| g.borrow().len());
    let mut edges: Vec<Vec<EClassId>> = vec![Vec::new(); n];
    IS_EFFECTFUL_TYPE.with(|v| { *v.borrow_mut() = vec![false; n]; });
    for i in 0..(n as EClassId) {
        if isType(i) {
            let m = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
            for j in 0..m {
                // assuming it will be merged with some grounded type
                let op_is_typelist_ith = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].op == "TypeList-ith");
                if op_is_typelist_ith {
                    continue;
                }
                let op_is_typelistremoveat = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].op == "TypeListRemoveAt");
                if op_is_typelistremoveat {
                    continue;
                }
                let ch_len = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].ch.len());
                for k in 0..ch_len {
                    let ch_k = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].ch[k].clone());
                    let v: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch_k).unwrap().0);
                    crate::debug_assert_tiger!(isType(v));
                    edges[v as usize].push(i);
                }
            }
        }
    }
    let mut stateT: EClassId = 0;
    for i in 0..(n as EClassId) {
        let m = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
        for j in 0..m {
            let op_is_statet = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].op == "StateT");
            if op_is_statet {
                stateT = i;
            }
        }
    }
    let mut q: VecDeque<EClassId> = VecDeque::new();
    IS_EFFECTFUL_TYPE.with(|v| { v.borrow_mut()[stateT as usize] = true; });
    q.push_back(stateT);
    while q.len() > 0 {
        let u = *q.front().unwrap();
        q.pop_front();
        for i in 0..edges[u as usize].len() {
            let v: EClassId = edges[u as usize][i];
            let is_eff = IS_EFFECTFUL_TYPE.with(|vec| vec.borrow()[v as usize]);
            if !is_eff {
                IS_EFFECTFUL_TYPE.with(|vec| { vec.borrow_mut()[v as usize] = true; });
                q.push_back(v);
            }
        }
    }
}

// vector<bool> hasEffectfulType;
thread_local! {
    static HAS_EFFECTFUL_TYPE: RefCell<Vec<bool>> = RefCell::new(Vec::new());
}

pub fn mark_effectful_exprs() {
    let n = RAW_EGRAPH.with(|g| g.borrow().len());
    HAS_EFFECTFUL_TYPE.with(|v| { *v.borrow_mut() = vec![false; n]; });
    for i in 0..(n as EClassId) {
        let m = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
        for j in 0..m {
            let op_is_hastype = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].op == "HasType");
            if op_is_hastype {
                let ch_len = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].ch.len());
                crate::debug_assert_tiger!(ch_len == 2);
                let ch0 = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].ch[0].clone());
                let ch1 = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].ch[1].clone());
                let ec: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch0).unwrap().0);
                let tc: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch1).unwrap().0);
                crate::debug_assert_tiger!(isExpr(ec));
                crate::debug_assert_tiger!(isType(tc));
                let is_eff = IS_EFFECTFUL_TYPE.with(|v| v.borrow()[tc as usize]);
                if is_eff {
                    HAS_EFFECTFUL_TYPE.with(|v| { v.borrow_mut()[ec as usize] = true; });
                }
            }
            // Additionally, mark function roots as effectful
            let op_is_function = RAW_EGRAPH.with(|g| g.borrow()[i as usize][j as usize].op == "Function");
            if op_is_function {
                HAS_EFFECTFUL_TYPE.with(|v| { v.borrow_mut()[i as usize] = true; });
            }
        }
    }
}

// vector<bool> reachable;
thread_local! {
    static REACHABLE: RefCell<Vec<bool>> = RefCell::new(Vec::new());
}

// vector<bool> necessary_types;
thread_local! {
    static NECESSARY_TYPES: RefCell<Vec<bool>> = RefCell::new(Vec::new());
}

pub fn isTypeNormalForm(op: &str) -> bool {
    op == "IntT" || op == "BoolT" || op == "FloatT"
        || op == "PointerT" || op == "StateT" || op == "Base"
        || op == "TupleT" || op == "TNil" || op == "TCons"
}

pub fn mark_reachable(root: EClassId) {
    if REACHABLE.with(|r| r.borrow()[root as usize]) {
        return;
    }
    let mut q: VecDeque<EClassId> = VecDeque::new();
    let mut tq: VecDeque<EClassId> = VecDeque::new();
    REACHABLE.with(|r| { r.borrow_mut()[root as usize] = true; });
    q.push_back(root);
    while q.len() > 0 {
        let u = *q.front().unwrap();
        q.pop_front();
        if isPrimitiveEClass(u) {
            continue;
        }
        let m = RAW_EGRAPH.with(|g| g.borrow()[u as usize].len()) as ENodeId;
        for i in 0..m {
            let ch_len = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch.len());
            for j in 0..ch_len {
                let ch_j = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch[j].clone());
                let v: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch_j).unwrap().0);
                let r_v = REACHABLE.with(|r| r.borrow()[v as usize]);
                if !r_v && (isExpr(v) || isPrimitiveEClass(v)) {
                    REACHABLE.with(|r| { r.borrow_mut()[v as usize] = true; });
                    q.push_back(v);
                }
            }
            // Special cases for Function and Alloc to preserve the types they depend on
            let op_is_function = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].op == "Function");
            if op_is_function {
                let ch_len2 = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch.len());
                crate::debug_assert_tiger!(ch_len2 == 4);
                let ch1 = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch[1].clone());
                let ch2 = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch[2].clone());
                let inputt: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch1).unwrap().0);
                let outputt: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch2).unwrap().0);
                crate::debug_assert_tiger!(isType(inputt));
                crate::debug_assert_tiger!(isType(outputt));
                let n_in = NECESSARY_TYPES.with(|n| n.borrow()[inputt as usize]);
                if !n_in {
                    NECESSARY_TYPES.with(|n| { n.borrow_mut()[inputt as usize] = true; });
                    tq.push_back(inputt);
                }
                let n_out = NECESSARY_TYPES.with(|n| n.borrow()[outputt as usize]);
                if !n_out {
                    NECESSARY_TYPES.with(|n| { n.borrow_mut()[outputt as usize] = true; });
                    tq.push_back(outputt);
                }
            }
            let op_is_alloc = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].op == "Alloc");
            if op_is_alloc {
                let ch_len2 = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch.len());
                crate::debug_assert_tiger!(ch_len2 == 4);
                let ch3 = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch[3].clone());
                let ty: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch3).unwrap().0);
                crate::debug_assert_tiger!(isType(ty));
                let n_ty = NECESSARY_TYPES.with(|n| n.borrow()[ty as usize]);
                if !n_ty {
                    NECESSARY_TYPES.with(|n| { n.borrow_mut()[ty as usize] = true; });
                    tq.push_back(ty);
                }
            }
        }
    }
    while tq.len() > 0 {
        let u = *tq.front().unwrap();
        tq.pop_front();
        #[cfg(feature = "debug")]
        {
            if !isType(u) {
                eprintln!("Found non-type children of a type: ");
                let (name0, op0) = RAW_EGRAPH.with(|g| {
                    let g = g.borrow();
                    (g[u as usize][0].name.clone(), g[u as usize][0].op.clone())
                });
                eprintln!("{} {}", name0, op0);
                assert!(false);
            }
        }
        let m = RAW_EGRAPH.with(|g| g.borrow()[u as usize].len()) as ENodeId;
        for i in 0..m {
            let op_i = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].op.clone());
            if isTypeNormalForm(&op_i) {
                let ch_len = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch.len());
                for j in 0..ch_len {
                    let ch_j = RAW_EGRAPH.with(|g| g.borrow()[u as usize][i as usize].ch[j].clone());
                    let v: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&ch_j).unwrap().0);
                    let n_v = NECESSARY_TYPES.with(|n| n.borrow()[v as usize]);
                    if !n_v {
                        NECESSARY_TYPES.with(|n| { n.borrow_mut()[v as usize] = true; });
                        tq.push_back(v);
                    }
                }
            }
        }
    }
}

// unordered_map<EClassId, EClassId> new_eclassidmp;
// Integer-keyed: use FxHash to mirror the trivial integer hash used by
// std::unordered_map<EClassId, EClassId> on the C++ side.
thread_local! {
    static NEW_ECLASSIDMP: RefCell<IndexMap<EClassId, EClassId, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>> =
        RefCell::new(IndexMap::with_hasher(std::hash::BuildHasherDefault::<rustc_hash::FxHasher>::default()));
}

const EXTRACTABLEOP: &[&str] = &[
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
    "Function",
];

pub fn isExtractableOP(op: &str) -> bool {
    let b0 = op.as_bytes()[0];
    if b0 == b'\\' || b0 == b'.' || b0 == b'-' || (b'0' <= b0 && b0 <= b'9') {
        return true;
    }
    for i in 0..EXTRACTABLEOP.len() {
        if op == EXTRACTABLEOP[i] {
            return true;
        }
    }
    false
}

pub fn build_simple_egraph() -> EGraph {
    let mut g: EGraph = EGraph::default();
    NEW_ECLASSIDMP.with(|m| { m.borrow_mut().clear(); });
    let n = RAW_EGRAPH.with(|g| g.borrow().len()) as EClassId;
    for i in 0..n {
        let r_i = REACHABLE.with(|r| r.borrow()[i as usize]);
        if r_i && (isExpr(i) || isPrimitiveEClass(i)) {
            let new_id = g.eclasses.len() as EClassId;
            NEW_ECLASSIDMP.with(|m| { m.borrow_mut().insert(i, new_id); });
            g.eclasses.push(EClass::default());
            let h_eff = HAS_EFFECTFUL_TYPE.with(|v| v.borrow()[i as usize]);
            g.eclasses.last_mut().unwrap().isEffectful = h_eff;
        }
        let nt_i = NECESSARY_TYPES.with(|n| n.borrow()[i as usize]);
        if nt_i {
            let new_id = g.eclasses.len() as EClassId;
            NEW_ECLASSIDMP.with(|m| { m.borrow_mut().insert(i, new_id); });
            g.eclasses.push(EClass::default());
            g.eclasses.last_mut().unwrap().isEffectful = false;
        }
        let nt_i2 = NECESSARY_TYPES.with(|n| n.borrow()[i as usize]);
        let r_i2 = REACHABLE.with(|r| r.borrow()[i as usize]);
        crate::debug_assert_tiger!(!(nt_i2 && r_i2));
    }
    for i in 0..n {
        let r_i = REACHABLE.with(|r| r.borrow()[i as usize]);
        if r_i {
            if isExpr(i) {
                let nid: EClassId = NEW_ECLASSIDMP.with(|m| *m.borrow().get(&i).unwrap());
                let m = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
                for j in 0..m {
                    // RawENode &rn = raw_egraph[i][j];
                    let (rn_op, rn_name, rn_ch) = RAW_EGRAPH.with(|g| {
                        let g = g.borrow();
                        let rn = &g[i as usize][j as usize];
                        (rn.op.clone(), rn.name.clone(), rn.ch.clone())
                    });
                    if isExtractableOP(&rn_op) {
                        let mut en = ENode::default();
                        en.head = format!("{}{}{}", rn_name, "###", rn_op);
                        en.eclass = nid;
                        for k in 0..rn_ch.len() {
                            let v: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&rn_ch[k]).unwrap().0);
                            let v_in_new = NEW_ECLASSIDMP.with(|m| m.borrow().contains_key(&v));
                            if v_in_new && (!isType(v) || rn_op == "Function" || rn_op == "Alloc") {
                                let nv = NEW_ECLASSIDMP.with(|m| *m.borrow().get(&v).unwrap());
                                en.ch.push(nv);
                            }
                        }
                        g.eclasses[nid as usize].enodes.push(en);
                    }
                }
            } else {
                let m = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
                for j in 0..m {
                    let (rn_op, rn_name, rn_ch) = RAW_EGRAPH.with(|g| {
                        let g = g.borrow();
                        let rn = &g[i as usize][j as usize];
                        (rn.op.clone(), rn.name.clone(), rn.ch.clone())
                    });
                    if isPrimitiveENode(i, j) && isExtractableOP(&rn_op) {
                        let nid: EClassId = NEW_ECLASSIDMP.with(|m| *m.borrow().get(&i).unwrap());
                        let mut en = ENode::default();
                        en.head = format!("{}{}{}", rn_name, "###", rn_op);
                        en.eclass = nid;
                        for k in 0..rn_ch.len() {
                            let v: EClassId = RAW_ENODEIDMP.with(|m| m.borrow().get(&rn_ch[k]).unwrap().0);
                            let v_in_new = NEW_ECLASSIDMP.with(|m| m.borrow().contains_key(&v));
                            if v_in_new && !isType(v) {
                                let nv = NEW_ECLASSIDMP.with(|m| *m.borrow().get(&v).unwrap());
                                en.ch.push(nv);
                            }
                        }
                        g.eclasses[nid as usize].enodes.push(en);
                    }
                }
            }
        }
        // preserve necessary types
        let nt_i = NECESSARY_TYPES.with(|n| n.borrow()[i as usize]);
        if nt_i {
            let nid: EClassId = NEW_ECLASSIDMP.with(|m| *m.borrow().get(&i).unwrap());
            let m = RAW_EGRAPH.with(|g| g.borrow()[i as usize].len()) as ENodeId;
            for j in 0..m {
                let (rn_op, rn_name, rn_ch) = RAW_EGRAPH.with(|g| {
                    let g = g.borrow();
                    let rn = &g[i as usize][j as usize];
                    (rn.op.clone(), rn.name.clone(), rn.ch.clone())
                });
                if isTypeNormalForm(&rn_op) {
                    let mut en = ENode::default();
                    en.head = format!("{}{}{}", rn_name, "###", rn_op);
                    en.eclass = nid;
                    for k in 0..rn_ch.len() {
                        let v: EClassId = RAW_ENODEIDMP.with(|mm| mm.borrow().get(&rn_ch[k]).unwrap().0);
                        let in_new = NEW_ECLASSIDMP.with(|mm| mm.borrow().contains_key(&v));
                        crate::debug_assert_tiger!(in_new);
                        let nv = NEW_ECLASSIDMP.with(|mm| *mm.borrow().get(&v).unwrap());
                        en.ch.push(nv);
                    }
                    g.eclasses[nid as usize].enodes.push(en);
                }
            }
            crate::debug_assert_tiger!(g.eclasses[nid as usize].enodes.len() == 1);
        }
    }
    g
}

pub fn parse_egglog_json() -> (EGraph, Vec<EClassId>) {
    read_nodes();
    propagate_effectful_types();
    mark_effectful_exprs();
    let roots: Vec<EClassId> = find_function_roots();
    let n = RAW_EGRAPH.with(|g| g.borrow().len());
    REACHABLE.with(|r| { *r.borrow_mut() = vec![false; n]; });
    NECESSARY_TYPES.with(|nt| { *nt.borrow_mut() = vec![false; n]; });
    for i in 0..roots.len() {
        mark_reachable(roots[i]);
    }
    let g: EGraph = build_simple_egraph();
    crate::debug_assert_tiger!(is_wellformed_egraph(&g, true, true));
    let p: (EGraph, crate::egraphin::EGraphMapping) = prune_unextractable_enodes(&g, -1);
    let mut new_roots: Vec<EClassId> = vec![0; roots.len()];
    for i in 0..roots.len() {
        let in_new = NEW_ECLASSIDMP.with(|m| m.borrow().contains_key(&roots[i]));
        crate::debug_assert_tiger!(in_new);
        let mapped = NEW_ECLASSIDMP.with(|m| *m.borrow().get(&roots[i]).unwrap());
        new_roots[i] = p.1.eclassidmp[mapped as usize];
        crate::debug_assert_tiger!(0 <= new_roots[i] && new_roots[i] < p.0.neclasses() as EClassId);
    }
    (p.0, new_roots)
}

