//! Build the tiger e-graph straight from an egglog `EGraph`.
//!
//! We walk the function tables through egglog's public read API in the same
//! order egglog's serializer visits them, so e-class and e-node numbering (and
//! therefore greedy tie-breaking) matches the original C++ extractor, which
//! read the serialized JSON.
//!
//! Everything tiger needs to know about eggcc's language is gathered in
//! [`SCHEMA`]: which sorts are expressions and types, how effectfulness is
//! recorded, which constructors are roots, and which are extractable.

use std::collections::VecDeque;
use std::hash::BuildHasherDefault;

use egglog::ast::Literal;
use egglog::sort::{F, S};
use egglog::{ArcSort, Value};
use indexmap::IndexMap;
use rustc_hash::FxHasher;

use crate::egraph::{EClass, EClassId, EGraph, ENode};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

/// The conventions of the language in the e-graph.
pub struct Schema {
    /// Sort-name prefixes of expression sorts: the e-classes tiger extracts over.
    pub expr_sorts: &'static [&'static str],
    /// Sort-name prefixes of type sorts.
    pub type_sorts: &'static [&'static str],
    /// The type constructor that marks the program state. Every type containing
    /// it is effectful, and so is every expression of such a type.
    pub state_type: &'static str,
    /// Relation `(HasType expr type)` giving expressions their types.
    pub has_type: &'static str,
    /// Type functions that are not part of a type's structure (skipped when
    /// propagating effectfulness).
    pub non_structural_type_ops: &'static [&'static str],
    /// Type constructors in normal form; only these are kept for the types tiger outputs.
    pub type_normal_form: &'static [&'static str],
    /// The constructor of extraction roots. Its e-classes are always effectful.
    pub root: &'static str,
    /// Constructors that keep (some of) their type children in the output, with the child positions.
    pub typed_ops: &'static [(&'static str, &'static [usize])],
    /// Constructors tiger may extract. Anything else is treated as unextractable.
    pub extractable_ops: &'static [&'static str],
}

impl Schema {
    fn is_expr_sort(&self, sort: &str) -> bool {
        self.expr_sorts.iter().any(|p| sort.starts_with(p))
    }

    fn is_type_sort(&self, sort: &str) -> bool {
        self.type_sorts.iter().any(|p| sort.starts_with(p))
    }

    fn is_type_normal_form(&self, op: &str) -> bool {
        self.type_normal_form.contains(&op)
    }

    /// Positions of the type children `op` keeps.
    fn kept_type_children(&self, op: &str) -> &'static [usize] {
        self.typed_ops
            .iter()
            .find(|(o, _)| *o == op)
            .map_or(&[], |(_, positions)| positions)
    }
}

/// eggcc's language.
pub const SCHEMA: Schema = Schema {
    expr_sorts: &["Expr", "Constant", "TernaryOp", "BinaryOp", "UnaryOp"],
    type_sorts: &["Type", "BaseType", "TypeList"],
    state_type: "StateT",
    has_type: "HasType",
    non_structural_type_ops: &["TypeList-ith", "TypeListRemoveAt"],
    type_normal_form: &[
        "IntT", "BoolT", "FloatT", "PointerT", "StateT", "Base", "TupleT", "TNil", "TCons",
    ],
    root: "Function",
    typed_ops: &[("Function", &[1, 2]), ("Alloc", &[3])],
    extractable_ops: &[
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
        // Schema
        "Bop",
        "Uop",
        "Top",
    ],
};

/// One table row, or one primitive / container value, of the egglog e-graph.
struct RawENode {
    /// Constructor name, or the literal's text for primitives.
    op: String,
    /// `Some` for base-value leaves (ints, strings, ...).
    lit: Option<Literal>,
    /// True for base values and containers (egglog serializes both as `primitive-*` nodes).
    is_primitive: bool,
    /// Raw e-class ids of the children.
    children: Vec<EClassId>,
}

impl RawENode {
    /// Literals are extractable, containers never are.
    fn is_extractable(&self) -> bool {
        if self.is_primitive {
            self.lit.is_some()
        } else {
            SCHEMA.extractable_ops.contains(&self.op.as_str())
        }
    }
}

/// A raw e-class: its sort name and its e-nodes.
struct RawEClass {
    sort: String,
    enodes: Vec<RawENode>,
}

impl RawEClass {
    fn is_expr(&self) -> bool {
        SCHEMA.is_expr_sort(&self.sort)
    }

    fn is_type(&self) -> bool {
        SCHEMA.is_type_sort(&self.sort)
    }

    fn is_primitive(&self) -> bool {
        self.enodes.iter().any(|n| n.is_primitive)
    }

    fn has_op(&self, op: &str) -> bool {
        self.enodes.iter().any(|n| n.op == op)
    }
}

/// The whole egglog e-graph, with eq-sort values canonicalized and numbered.
struct RawEGraph {
    classes: Vec<RawEClass>,
}

impl RawEGraph {
    fn len(&self) -> usize {
        self.classes.len()
    }

    fn class_ids(&self) -> std::ops::Range<EClassId> {
        0..self.len()
    }

    fn is_expr(&self, c: EClassId) -> bool {
        self.classes[c].is_expr()
    }

    fn is_type(&self, c: EClassId) -> bool {
        self.classes[c].is_type()
    }

    fn is_primitive(&self, c: EClassId) -> bool {
        self.classes[c].is_primitive()
    }
}

/// An e-class before numbering: (sort index, canonical value).
type ClassKey = (usize, Value);

/// A node in egglog serialization order, with children still as keys.
struct PendingNode {
    key: ClassKey,
    children: Vec<ClassKey>,
    op: String,
    lit: Option<Literal>,
    is_primitive: bool,
}

/// The leaf node for a non-eq-sort value. Base values become literals;
/// containers (e.g. `(Set Expr)`) stay opaque.
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
        children: Vec::new(),
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
        *sort_index
            .entry(sort.name().to_string())
            .or_insert_with(|| {
                sorts.push(sort.clone());
                sorts.len() - 1
            })
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
                let (out, inputs) = row.vals.split_last().expect("row has an output");
                // egglog serializes the output value first (adding a node only when it
                // is a primitive), then the inputs, then the row itself.
                let out_key = key_of(out_idx, out_sort, *out);
                if !out_sort.is_eq_sort() && seen_primitive.insert(out_key, ()).is_none() {
                    pending.push(primitive_node(egraph, out_sort, *out, out_key));
                }
                let mut children: Vec<ClassKey> = Vec::with_capacity(inputs.len());
                for ((v, sort), &sort_idx) in inputs.iter().zip(&schema.input).zip(&in_idxs) {
                    let key = key_of(sort_idx, sort, *v);
                    if !sort.is_eq_sort() && seen_primitive.insert(key, ()).is_none() {
                        pending.push(primitive_node(egraph, sort, *v, key));
                    }
                    children.push(key);
                }
                pending.push(PendingNode {
                    key: out_key,
                    children,
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
        classes: Vec::new(),
    };
    for node in &pending {
        class_ids.entry(node.key).or_insert_with(|| {
            raw.classes.push(RawEClass {
                sort: sorts[node.key.0].name().to_string(),
                enodes: Vec::new(),
            });
            raw.len() - 1
        });
    }
    for node in pending {
        let children = node
            .children
            .iter()
            .map(|k| *class_ids.get(k).expect("child e-class has no e-nodes"))
            .collect();
        raw.classes[class_ids[&node.key]].enodes.push(RawENode {
            op: node.op,
            lit: node.lit,
            is_primitive: node.is_primitive,
            children,
        });
    }
    raw
}

/// Types that (transitively) contain the state type.
fn effectful_types(raw: &RawEGraph) -> Vec<bool> {
    let mut users: Vec<Vec<EClassId>> = vec![Vec::new(); raw.len()];
    for (c, class) in raw.classes.iter().enumerate().filter(|(_, c)| c.is_type()) {
        for node in &class.enodes {
            // Assumed to be merged with some structural type.
            if SCHEMA.non_structural_type_ops.contains(&node.op.as_str()) {
                continue;
            }
            for &child in &node.children {
                debug_assert!(raw.is_type(child));
                users[child].push(c);
            }
        }
    }
    let mut effectful = vec![false; raw.len()];
    let mut queue: VecDeque<EClassId> = raw
        .class_ids()
        .filter(|&c| raw.classes[c].has_op(SCHEMA.state_type))
        .last()
        .into_iter()
        .collect();
    for &c in &queue {
        effectful[c] = true;
    }
    while let Some(u) = queue.pop_front() {
        for &v in &users[u] {
            if !effectful[v] {
                effectful[v] = true;
                queue.push_back(v);
            }
        }
    }
    effectful
}

/// Expressions with an effectful type, plus every root e-class.
fn effectful_exprs(raw: &RawEGraph, effectful_type: &[bool]) -> Vec<bool> {
    let mut effectful = vec![false; raw.len()];
    for (c, class) in raw.classes.iter().enumerate() {
        for node in &class.enodes {
            if node.op == SCHEMA.has_type {
                let [expr, ty] = node.children[..] else {
                    panic!("{} should relate an expression and a type", SCHEMA.has_type)
                };
                debug_assert!(raw.is_expr(expr) && raw.is_type(ty));
                if effectful_type[ty] {
                    effectful[expr] = true;
                }
            }
            if node.op == SCHEMA.root {
                effectful[c] = true;
            }
        }
    }
    effectful
}

/// E-classes reachable from the roots: expressions and primitives, plus the
/// (normal-form) types that typed ops keep. Returns `(reachable, kept_types)`.
fn reachable_from(raw: &RawEGraph, roots: &[EClassId]) -> (Vec<bool>, Vec<bool>) {
    let mut reachable = vec![false; raw.len()];
    let mut kept_type = vec![false; raw.len()];
    let mut queue = VecDeque::new();
    let mut type_queue = VecDeque::new();
    for &root in roots {
        if !reachable[root] {
            reachable[root] = true;
            queue.push_back(root);
        }
        while let Some(u) = queue.pop_front() {
            if raw.is_primitive(u) {
                continue;
            }
            for node in &raw.classes[u].enodes {
                for &v in &node.children {
                    if !reachable[v] && (raw.is_expr(v) || raw.is_primitive(v)) {
                        reachable[v] = true;
                        queue.push_back(v);
                    }
                }
                for &pos in SCHEMA.kept_type_children(&node.op) {
                    let ty = node.children[pos];
                    debug_assert!(raw.is_type(ty));
                    if !kept_type[ty] {
                        kept_type[ty] = true;
                        type_queue.push_back(ty);
                    }
                }
            }
        }
        while let Some(u) = type_queue.pop_front() {
            debug_assert!(raw.is_type(u));
            for node in &raw.classes[u].enodes {
                if SCHEMA.is_type_normal_form(&node.op) {
                    for &v in &node.children {
                        if !kept_type[v] {
                            kept_type[v] = true;
                            type_queue.push_back(v);
                        }
                    }
                }
            }
        }
    }
    (reachable, kept_type)
}

/// The e-graph tiger extracts from: reachable expression and primitive
/// e-classes with their extractable e-nodes, plus the kept types. Type children
/// are dropped except where a typed op keeps them. Returns the e-graph and the
/// raw -> new id map.
fn build_extraction_egraph(
    raw: &RawEGraph,
    reachable: &[bool],
    kept_type: &[bool],
    effectful: &[bool],
) -> (EGraph, FxIndexMap<EClassId, EClassId>) {
    let mut g = EGraph::default();
    let mut new_id: FxIndexMap<EClassId, EClassId> = FxIndexMap::default();
    for c in raw.class_ids() {
        debug_assert!(!(kept_type[c] && reachable[c]));
        let keep = (reachable[c] && (raw.is_expr(c) || raw.is_primitive(c))) || kept_type[c];
        if keep {
            new_id.insert(c, g.len());
            g.classes.push(EClass {
                enodes: Vec::new(),
                is_effectful: reachable[c] && effectful[c],
            });
        }
    }
    let make_enode = |node: &RawENode, keep_child: &dyn Fn(EClassId) -> bool| ENode {
        op: node.op.clone(),
        lit: node.lit.clone(),
        children: node
            .children
            .iter()
            .filter(|&&v| keep_child(v))
            .filter_map(|v| new_id.get(v).copied())
            .collect(),
    };
    for (c, class) in raw.classes.iter().enumerate() {
        let Some(&nid) = new_id.get(&c) else {
            continue;
        };
        let enodes = &mut g.classes[nid].enodes;
        if kept_type[c] {
            for node in class
                .enodes
                .iter()
                .filter(|n| SCHEMA.is_type_normal_form(&n.op))
            {
                enodes.push(make_enode(node, &|_| true));
            }
            debug_assert!(enodes.len() == 1, "kept type has one normal form");
        } else if class.is_expr() {
            for node in class.enodes.iter().filter(|n| n.is_extractable()) {
                let kept = SCHEMA.kept_type_children(&node.op);
                let keeps_types = !kept.is_empty();
                enodes.push(make_enode(node, &|v| keeps_types || !raw.is_type(v)));
            }
        } else {
            for node in class
                .enodes
                .iter()
                .filter(|n| n.is_primitive && n.is_extractable())
            {
                enodes.push(make_enode(node, &|v| !raw.is_type(v)));
            }
        }
    }
    (g, new_id)
}

/// Build the tiger e-graph for `egraph` and return it with its root e-classes.
pub fn build_egraph(egraph: &egglog::EGraph) -> (EGraph, Vec<EClassId>) {
    let raw = collect_raw_egraph(egraph);
    let effectful_type = effectful_types(&raw);
    let effectful = effectful_exprs(&raw, &effectful_type);
    // An e-class with several root nodes (e.g. a recursive function after inlining) is one root.
    let roots: Vec<EClassId> = raw
        .class_ids()
        .filter(|&c| raw.classes[c].has_op(SCHEMA.root))
        .collect();
    let (reachable, kept_type) = reachable_from(&raw, &roots);
    let (g, new_id) = build_extraction_egraph(&raw, &reachable, &kept_type, &effectful);
    debug_assert!(crate::checks::is_wellformed(&g, true, true));
    let (pruned, mapping) = g.prune_unextractable(None);
    let roots = roots.iter().map(|r| mapping.class(new_id[r])).collect();
    (pruned, roots)
}
