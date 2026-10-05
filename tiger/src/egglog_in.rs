//! Build the tiger e-graph straight from an egglog `EGraph`.
//!
//! Replaces the C++ `json2egraphin.cpp`, which parsed egglog's serialized JSON.
//! We walk the function tables through egglog's public read API instead, in the
//! same order egglog's serializer would visit them, so e-class and e-node
//! numbering (and therefore greedy tie-breaking) matches the C++ extractor.
//!
//! The steps mirror the C++: collect every table row as a raw e-node, find
//! `Function` roots, mark effectful e-classes through `HasType` and the
//! `StateT` type, keep only what is reachable from the roots, drop type
//! children (except under `Function` and `Alloc`), and prune unextractable
//! e-nodes.

use std::collections::VecDeque;
use std::hash::BuildHasherDefault;

use egglog::ast::Literal;
use egglog::sort::{F, S};
use egglog::{ArcSort, Value};
use indexmap::IndexMap;
use rustc_hash::FxHasher;

use crate::egraph::{EClass, EClassId, EGraph, ENode, ENodeId};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

/// One table row (or one primitive / container value) of the egglog e-graph.
struct RawENode {
    /// Constructor name, or the literal's text for primitives.
    op: String,
    /// `Some` for base-value leaves (ints, strings, ...).
    lit: Option<Literal>,
    /// True for base values and containers (egglog serializes both as `primitive-*` nodes).
    is_primitive: bool,
    /// Raw e-class ids of the children.
    ch: Vec<EClassId>,
}

/// A raw e-class: its egglog sort name and its e-nodes, indexed by raw id.
struct RawEGraph {
    sort_names: Vec<String>,
    classes: Vec<Vec<RawENode>>,
}

impl RawEGraph {
    fn len(&self) -> usize {
        self.classes.len()
    }

    fn node(&self, i: EClassId, j: ENodeId) -> &RawENode {
        &self.classes[i][j]
    }

    fn nenodes(&self, i: EClassId) -> ENodeId {
        self.classes[i].len()
    }

    fn sort_name(&self, i: EClassId) -> &str {
        &self.sort_names[i]
    }

    fn is_expr(&self, i: EClassId) -> bool {
        let s = self.sort_name(i);
        s.starts_with("Expr")
            || s.starts_with("Constant")
            || s.starts_with("TernaryOp")
            || s.starts_with("BinaryOp")
            || s.starts_with("UnaryOp")
    }

    fn is_type(&self, i: EClassId) -> bool {
        let s = self.sort_name(i);
        s.starts_with("Type") || s.starts_with("BaseType") || s.starts_with("TypeList")
    }

    fn is_primitive_eclass(&self, i: EClassId) -> bool {
        self.classes[i].iter().any(|n| n.is_primitive)
    }

    fn has_op(&self, i: EClassId, op: &str) -> bool {
        self.classes[i].iter().any(|n| n.op == op)
    }
}

/// An e-class key before numbering: (sort index, canonical value).
type ClassKey = (usize, Value);

/// A node in egglog serialization order, with children still as keys.
struct PendingNode {
    key: ClassKey,
    ch: Vec<ClassKey>,
    op: String,
    lit: Option<Literal>,
    is_primitive: bool,
}

/// The leaf node for a non-eq-sort value. Base values become literals;
/// containers (e.g. `(Set Expr)`) are opaque and never extractable.
fn primitive_node(egraph: &egglog::EGraph, sort: &ArcSort, v: Value, key: ClassKey) -> PendingNode {
    let lit = match sort.name() {
        "i64" => Some(Literal::Int(egraph.value_to_base::<i64>(v))),
        "f64" => Some(Literal::Float(egraph.value_to_base::<F>(v).0)),
        "bool" => Some(Literal::Bool(egraph.value_to_base::<bool>(v))),
        "String" => Some(Literal::String(egraph.value_to_base::<S>(v).0.clone())),
        "Unit" => Some(Literal::Unit),
        _ => None,
    };
    let op = match &lit {
        Some(lit) => lit.to_string(),
        None => sort.name().to_string(),
    };
    PendingNode {
        key,
        ch: Vec::new(),
        op,
        lit,
        is_primitive: true,
    }
}

/// Walk every function table and build the raw e-graph.
fn collect_raw_egraph(egraph: &egglog::EGraph) -> RawEGraph {
    // Sorts are interned so class keys stay small.
    let mut sort_index: FxIndexMap<String, usize> = FxIndexMap::default();
    let mut sorts: Vec<ArcSort> = Vec::new();
    let mut intern_sort = |sort: &ArcSort| -> usize {
        if let Some(&i) = sort_index.get(sort.name()) {
            return i;
        }
        let i = sorts.len();
        sort_index.insert(sort.name().to_string(), i);
        sorts.push(sort.clone());
        i
    };

    let key_of = |sort_idx: usize, sort: &ArcSort, v: Value| -> ClassKey {
        (sort_idx, egraph.get_canonical_value(v, sort))
    };

    let mut pending: Vec<PendingNode> = Vec::new();
    let mut seen_primitive: FxIndexMap<ClassKey, ()> = FxIndexMap::default();

    for name in egraph.get_function_names() {
        let func = egraph
            .get_function(&name)
            .expect("function listed but not found");
        if func.is_let_binding() {
            continue;
        }
        let schema = func.schema();
        let out_sort = &schema.output;
        let out_idx = intern_sort(out_sort);
        let in_idxs: Vec<usize> = schema.input.iter().map(&mut intern_sort).collect();

        egraph
            .function_for_each(&name, |row| {
                let (out, inps) = row.vals.split_last().expect("row has an output");
                // egglog serializes the output value first (adding a node only when it
                // is a primitive), then the inputs, then the row itself.
                let out_key = key_of(out_idx, out_sort, *out);
                if !out_sort.is_eq_sort() && seen_primitive.insert(out_key, ()).is_none() {
                    pending.push(primitive_node(egraph, out_sort, *out, out_key));
                }
                let mut ch: Vec<ClassKey> = Vec::with_capacity(inps.len());
                for ((v, sort), &sort_idx) in inps.iter().zip(&schema.input).zip(&in_idxs) {
                    let key = key_of(sort_idx, sort, *v);
                    if !sort.is_eq_sort() && seen_primitive.insert(key, ()).is_none() {
                        pending.push(primitive_node(egraph, sort, *v, key));
                    }
                    ch.push(key);
                }
                pending.push(PendingNode {
                    key: out_key,
                    ch,
                    op: name.clone(),
                    lit: None,
                    is_primitive: false,
                });
            })
            .expect("function listed but not found");
    }

    // Number e-classes in order of first appearance as a node's own class.
    let mut class_ids: FxIndexMap<ClassKey, EClassId> = FxIndexMap::default();
    let mut raw = RawEGraph {
        sort_names: Vec::new(),
        classes: Vec::new(),
    };
    for node in &pending {
        if !class_ids.contains_key(&node.key) {
            class_ids.insert(node.key, raw.len());
            raw.sort_names.push(sorts[node.key.0].name().to_string());
            raw.classes.push(Vec::new());
        }
    }
    for node in pending {
        let ch = node
            .ch
            .iter()
            .map(|k| {
                *class_ids
                    .get(k)
                    .expect("child e-class has no e-nodes; incomplete e-graph")
            })
            .collect();
        let class = class_ids[&node.key];
        raw.classes[class].push(RawENode {
            op: node.op,
            lit: node.lit,
            is_primitive: node.is_primitive,
            ch,
        });
    }
    raw
}

/// E-classes containing a `Function` node, in e-class order. An e-class with
/// several `Function` nodes (e.g. a recursive function after inlining) is one root.
fn find_function_roots(raw: &RawEGraph) -> Vec<EClassId> {
    (0..raw.len())
        .filter(|&i| raw.has_op(i, "Function"))
        .collect()
}

/// Types that (transitively) contain `StateT`.
fn propagate_effectful_types(raw: &RawEGraph) -> Vec<bool> {
    let n = raw.len();
    let mut edges: Vec<Vec<EClassId>> = vec![Vec::new(); n];
    let mut is_effectful_type = vec![false; n];
    for i in 0..n {
        if !raw.is_type(i) {
            continue;
        }
        for j in 0..raw.nenodes(i) {
            let node = raw.node(i, j);
            // assuming it will be merged with some grounded type
            if node.op == "TypeList-ith" || node.op == "TypeListRemoveAt" {
                continue;
            }
            for &v in &node.ch {
                debug_assert!(raw.is_type(v));
                edges[v].push(i);
            }
        }
    }
    let mut state_t: EClassId = 0;
    for i in 0..n {
        if raw.has_op(i, "StateT") {
            state_t = i;
        }
    }
    let mut q: VecDeque<EClassId> = VecDeque::new();
    is_effectful_type[state_t] = true;
    q.push_back(state_t);
    while let Some(u) = q.pop_front() {
        for &v in &edges[u] {
            if !is_effectful_type[v] {
                is_effectful_type[v] = true;
                q.push_back(v);
            }
        }
    }
    is_effectful_type
}

/// Exprs with an effectful type (via `HasType`), plus every `Function` e-class.
fn mark_effectful_exprs(raw: &RawEGraph, is_effectful_type: &[bool]) -> Vec<bool> {
    let n = raw.len();
    let mut has_effectful_type = vec![false; n];
    for i in 0..n {
        for j in 0..raw.nenodes(i) {
            let node = raw.node(i, j);
            if node.op == "HasType" {
                debug_assert!(node.ch.len() == 2);
                let ec = node.ch[0];
                let tc = node.ch[1];
                debug_assert!(raw.is_expr(ec));
                debug_assert!(raw.is_type(tc));
                if is_effectful_type[tc] {
                    has_effectful_type[ec] = true;
                }
            }
            if node.op == "Function" {
                has_effectful_type[i] = true;
            }
        }
    }
    has_effectful_type
}

fn is_type_normal_form(op: &str) -> bool {
    matches!(
        op,
        "IntT" | "BoolT" | "FloatT" | "PointerT" | "StateT" | "Base" | "TupleT" | "TNil" | "TCons"
    )
}

/// Reachability from a root over Expr and primitive e-classes. Also records the
/// types `Function` and `Alloc` nodes depend on, since those are kept in the output.
fn mark_reachable(
    raw: &RawEGraph,
    root: EClassId,
    reachable: &mut [bool],
    necessary_types: &mut [bool],
) {
    if reachable[root] {
        return;
    }
    let mut q: VecDeque<EClassId> = VecDeque::new();
    let mut tq: VecDeque<EClassId> = VecDeque::new();
    reachable[root] = true;
    q.push_back(root);
    while let Some(u) = q.pop_front() {
        if raw.is_primitive_eclass(u) {
            continue;
        }
        for i in 0..raw.nenodes(u) {
            let node = raw.node(u, i);
            for &v in &node.ch {
                if !reachable[v] && (raw.is_expr(v) || raw.is_primitive_eclass(v)) {
                    reachable[v] = true;
                    q.push_back(v);
                }
            }
            // Special cases for Function and Alloc to preserve the types they depend on
            let mut need_type = |t: EClassId| {
                debug_assert!(raw.is_type(t));
                if !necessary_types[t] {
                    necessary_types[t] = true;
                    tq.push_back(t);
                }
            };
            if node.op == "Function" {
                debug_assert!(node.ch.len() == 4);
                need_type(node.ch[1]);
                need_type(node.ch[2]);
            }
            if node.op == "Alloc" {
                debug_assert!(node.ch.len() == 4);
                need_type(node.ch[3]);
            }
        }
    }
    while let Some(u) = tq.pop_front() {
        debug_assert!(raw.is_type(u));
        for i in 0..raw.nenodes(u) {
            let node = raw.node(u, i);
            if is_type_normal_form(&node.op) {
                for &v in &node.ch {
                    if !necessary_types[v] {
                        necessary_types[v] = true;
                        tq.push_back(v);
                    }
                }
            }
        }
    }
}

const EXTRACTABLE_OPS: &[&str] = &[
    "Int",
    "Bool",
    "Float",
    // Leaves
    "Const",
    "Arg",
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
    "DoWhile",
    "If",
    "Switch",
    // Schema
    "Bop",
    "Uop",
    "Top",
];

fn is_extractable(node: &RawENode) -> bool {
    if node.is_primitive {
        // Literals are extractable; containers are not.
        return node.lit.is_some();
    }
    EXTRACTABLE_OPS.contains(&node.op.as_str())
}

/// The simplified e-graph tiger extracts from: reachable Expr and primitive
/// e-classes, plus the type e-classes `Function` and `Alloc` need. Type children
/// are dropped everywhere else. Returns the e-graph and the raw -> new id map.
fn build_simple_egraph(
    raw: &RawEGraph,
    reachable: &[bool],
    necessary_types: &[bool],
    has_effectful_type: &[bool],
) -> (EGraph, FxIndexMap<EClassId, EClassId>) {
    let mut g = EGraph::default();
    let mut new_id: FxIndexMap<EClassId, EClassId> = FxIndexMap::default();
    let n = raw.len();
    for i in 0..n {
        if reachable[i] && (raw.is_expr(i) || raw.is_primitive_eclass(i)) {
            new_id.insert(i, g.len());
            g.classes.push(EClass {
                enodes: Vec::new(),
                is_effectful: has_effectful_type[i],
            });
        }
        if necessary_types[i] {
            new_id.insert(i, g.len());
            g.classes.push(EClass {
                enodes: Vec::new(),
                is_effectful: false,
            });
        }
        debug_assert!(!(necessary_types[i] && reachable[i]));
    }
    let make_enode = |node: &RawENode, keep_child: &dyn Fn(EClassId) -> bool| ENode {
        op: node.op.clone(),
        lit: node.lit.clone(),
        children: node
            .ch
            .iter()
            .filter(|&&v| keep_child(v))
            .filter_map(|v| new_id.get(v).copied())
            .collect(),
    };
    for i in 0..n {
        if reachable[i] {
            if raw.is_expr(i) {
                let nid = new_id[&i];
                for j in 0..raw.nenodes(i) {
                    let node = raw.node(i, j);
                    if is_extractable(node) {
                        let keeps_types = node.op == "Function" || node.op == "Alloc";
                        let en = make_enode(node, &|v| !raw.is_type(v) || keeps_types);
                        g.classes[nid].enodes.push(en);
                    }
                }
            } else {
                for j in 0..raw.nenodes(i) {
                    let node = raw.node(i, j);
                    if node.is_primitive && is_extractable(node) {
                        let nid = new_id[&i];
                        let en = make_enode(node, &|v| !raw.is_type(v));
                        g.classes[nid].enodes.push(en);
                    }
                }
            }
        }
        // preserve necessary types
        if necessary_types[i] {
            let nid = new_id[&i];
            for j in 0..raw.nenodes(i) {
                let node = raw.node(i, j);
                if is_type_normal_form(&node.op) {
                    let en = make_enode(node, &|_| true);
                    debug_assert!(en.children.len() == node.ch.len());
                    g.classes[nid].enodes.push(en);
                }
            }
            debug_assert!(g.classes[nid].enodes.len() == 1);
        }
    }
    (g, new_id)
}

/// Build the tiger e-graph for `egraph` and return it with its `Function` roots.
pub fn build_egraph(egraph: &egglog::EGraph) -> (EGraph, Vec<EClassId>) {
    let raw = collect_raw_egraph(egraph);
    let is_effectful_type = propagate_effectful_types(&raw);
    let has_effectful_type = mark_effectful_exprs(&raw, &is_effectful_type);
    let roots = find_function_roots(&raw);
    let n = raw.len();
    let mut reachable = vec![false; n];
    let mut necessary_types = vec![false; n];
    for &root in &roots {
        mark_reachable(&raw, root, &mut reachable, &mut necessary_types);
    }
    let (g, new_id) = build_simple_egraph(&raw, &reachable, &necessary_types, &has_effectful_type);
    debug_assert!(crate::checks::is_wellformed(&g, true, true));
    let (pruned, mapping) = g.prune_unextractable(None);
    let new_roots = roots.iter().map(|r| mapping.class(new_id[r])).collect();
    (pruned, new_roots)
}
