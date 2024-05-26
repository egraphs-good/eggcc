//! A pass over a CFG returned from the RVSDG=>CFG [conversion module](crate::rvsdg::to_cfg)
//! to simplify branch structures.
//!
//! # Introduction
//! RVSDGs are more structured than arbitrary CFGs. The only control strutures
//! that RVSDGs support directly are ifs (with an else) and tail-controlled
//! loops. This means that any other control flow constructs, from `for` loops
//! all the way to `goto`s need to be simulated using auxiliary predicate
//! variables introduced during translation time.
//!
//! The resulting representation is great for declarative rewrites, but can
//! generate slower code when directly interpreted as a CFG, e.g. to break out
//! of multiple layers of nested loop, CFGs only require a single jump but
//! RVSDGs require a branch for every layer of nesting.
//!
//! The algorithm in this module aims to recover more natural, efficient
//! control-flow structure from the raw CFG generated by an RVSDG. It is
//! inspired by the PCFR algorithm described in "Perfect Reconstructability of
//! Control Flow from Demand Dependence Graphs" by Bahman, Reissmann, Jahre, and
//! Meyer, but it has a different structure:
//!
//!  * PCFR operates on the RVSDG directly, while this algorithm operates on the
//!  resulting CFG. This is pragmatically useful for eggcc, which already has
//!  fairly involved RVSDG=>CFG conversion code.
//!  * PCFR expects an RVSDG in _predicate continuation form_, where predicates
//!  are introduced immediately before they are used. eggcc almost certainly
//!  does not preserve this property, and we want to avoid duplicating or
//!  splitting RVSDG nodes to reintroduce it. The algorithm in this module is
//!  robust to some predicates being used more than once, sometiems across
//!  branches.
//!  * The algorithm in this module has not been optimized for efficiency and as
//!  a result is likely slower than a good implementation of PCFR. This doesn't
//!  seem like an inherent issue and the performance of the two should be
//!  similar after some optimization.
//!  * The paper from Bahman et. al. also sketches a "ShortCircuitCFG" algorithm
//!  that is similar to the algorithm here, but makes some simplifying
//!  assumptions, potentially based again on predicate continuation form.
//!   
//! # Algorithm Overview
//! The algorithm code is fairly heavily commented. It relies on computing the
//! fixpoint of a monotone dataflow analysis tracking the value of boolean
//! identifiers at each CFG node. The analysis takes branches into account:
//! successors along the "true" edge of a branch on 'x' know that 'x' is true.
//! With that information in place (along with a few technical details explained
//! in code comments), we apply two kinds of rewrites on the CFG:
//!
//!   * For patterns like `X -[e]-> Y -[if a=1]-> Z` where we know that `a=1` in `X`
//!   (and `Y` doesn't overwrite `a`), rewrite to `X -[e]-> Z`.
//!   * For patterns like `X -[if a=1]-> Y` where we know that `a=1` in `X`,
//!   rewrite to `X -[jump]-> Y` and remove all other outgoing edges from `X`.
//!   If this is the only incoming branch to `Y`, a future optimize_direct_jumps
//!   pass will merge the two blocks entirely.
//!
//! The boolean value analysis should converge quickly given the structure of
//! the CFGs we generate, but the current implementation involves lots of
//! copying of data: If the CFG were in SSI form (SSA + variable splits on
//! branches), I believe that we could build a more efficient analysis by
//! looking at nodes where variables are assigned (or branch targets) and
//! relying on dominance information to infer whether a boolean variable has a
//! known value at a node.

use std::{collections::VecDeque, io::Write, mem};

use crate::cfg::{BasicBlock, BlockName, Branch, BranchOp, CondVal, Identifier, SimpleCfgFunction};
use bril_rs::{Argument, Instruction, Literal, Type, ValueOps};
use hashbrown::{HashMap, HashSet};
use indexmap::{IndexMap, IndexSet};
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::{Dfs, NodeIndexable},
    Direction,
};

impl SimpleCfgFunction {
    pub(crate) fn simplify_branches(&mut self) {
        // Step 1: compute some information about the CFG.
        // * Find "administrative" nodes.
        // * Find conditional branches.
        // * Start off a Value Analysis for the function.
        let branch_meta = self.get_branch_metadata();
        let mut val_analysis = ValueAnalysis::new(self);
        // Step 2: split conditional branches and mark the relevant constants as
        // known in the later nodes. This lets us simplify the value analysis by
        // having empty nodes enncapsulate the information imparted by the
        // branch.
        for (id, edge, val) in branch_meta
            .branches
            .iter()
            .flat_map(|(id, edges)| edges.iter().map(move |(edge, val)| (id, *edge, val)))
        {
            let Some(lit) = to_lit(val) else {
                continue;
            };
            // Count downwards from usize::MAX to avoid collisions with other placeholders
            let node_bound = usize::MAX - self.graph.node_bound();
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            let weight = self.graph.remove_edge(edge).unwrap();
            let block_name = BlockName::Placeholder(node_bound);
            let mid = self.graph.add_node(BasicBlock::empty(block_name));
            self.graph.add_edge(source, mid, weight);
            // NB: We rely on the optimize_direct_jumps pass to collapse this
            // back down. See really high placeholders in the final output? It's
            // probably a bug in one of these two.
            self.graph.add_edge(
                mid,
                target,
                Branch {
                    op: BranchOp::Jmp,
                    pos: None,
                },
            );
            val_analysis.add_assignment(mid, id.clone(), ValueInfo::Known(lit));
        }
        // Step 3: Compute the fixpoint of the value analysis.
        val_analysis.compute_fixpoint(self);
        // Step 4: Rewrite branches:
        // * For each administrative node `n``...
        // * For each outgoing branch [edge e1] with cond val `v` for `id`
        // * Check if `id` was written to in `n`, if it was, then move on
        // _unless_ we know the value of `id`; in which case we can replace the branch with a jump.
        // * Otherwise, check if a predecessor [via edge e2] node has `v` as a
        // known value for `id`.
        // * If so, copy the contents of the admin node to that predecessor, and
        // reroute e2 to the target of e1.
        let mut scratch = Vec::new();
        for admin_node in &branch_meta.admin_nodes {
            let mut walker = self
                .graph
                .neighbors_directed(*admin_node, Direction::Outgoing)
                .detach();
            // Don't reroute past the exit node. We want to make sure it stays reachable.
            if admin_node == &self.exit {
                continue;
            }
            while let Some((outgoing, succ)) = walker.next(&self.graph) {
                let BranchOp::Cond { arg, val, .. } = self.graph[outgoing].op.clone() else {
                    continue;
                };
                let Some(val) = to_lit(&val) else {
                    continue;
                };
                if val_analysis.data[admin_node].kills.contains(&arg) {
                    if succ != self.exit
                        && self.graph.neighbors(*admin_node).any(|x| x == self.exit)
                    {
                        // Don't remove any outgoing links to the exit node.
                        break;
                    }
                    // We assign to the branched-on argument in the admin
                    // node. See if we can fold the constant branch here.
                    let ValueInfo::Known(lit) = val_analysis.data[admin_node].get_output(&arg)
                    else {
                        continue;
                    };
                    if lit != val {
                        continue;
                    }
                    // okay, we have found a matching edge. Replace this branch
                    // with a jump.
                    let mut walker = self
                        .graph
                        .neighbors_directed(*admin_node, Direction::Outgoing)
                        .detach();
                    while let Some((outgoing, _)) = walker.next(&self.graph) {
                        self.graph.remove_edge(outgoing);
                    }
                    self.graph.add_edge(
                        *admin_node,
                        succ,
                        Branch {
                            op: BranchOp::Jmp,
                            pos: None,
                        },
                    );
                    // Don't run the rest of the inner loop.
                    break;
                }
                let mut incoming_walker = self
                    .graph
                    .neighbors_directed(*admin_node, Direction::Incoming)
                    .detach();
                while let Some((incoming, pred)) = incoming_walker.next(&self.graph) {
                    let can_reroute = matches!(val_analysis.data[&pred].get_output(&arg), ValueInfo::Known(v) if v == val);
                    if !can_reroute {
                        continue;
                    }

                    let weight = self.graph.remove_edge(incoming).unwrap();
                    // We only have to worry about `instrs` because we
                    // checked that the footer was empty when we populated
                    // admin_nodes. We do this because we more or less don't
                    // use footers on our way back to bril.
                    scratch.extend(self.graph[*admin_node].instrs.iter().cloned());
                    let (_, target) = self.graph.edge_endpoints(outgoing).unwrap();
                    let target_incoming = self
                        .graph
                        .neighbors_directed(target, Direction::Incoming)
                        .count();
                    let is_jump = matches!(weight.op, BranchOp::Jmp);
                    // Now it comes to move the block somewhere: if the
                    // incoming edge is a jump, then we would run all of the
                    // instructions in the current block anyway, we can just
                    // move them up.
                    if is_jump {
                        self.graph[pred].instrs.append(&mut scratch);
                        self.graph.add_edge(pred, target, weight);
                        break;
                    } else if target_incoming == 0 {
                        // The next safe case is if we are replacing the targets
                        // only incoming edge. In that case, we can move the
                        // data down.
                        let target_block = &mut self.graph[target];
                        scratch.append(&mut target_block.instrs);
                        mem::swap(&mut target_block.instrs, &mut scratch);
                        self.graph.add_edge(pred, target, weight);
                        break;
                    } else {
                        scratch.clear();
                        // Otherwise we may need some sort of compatibility check to
                        // merge the block somewhere. Add the edge back for now:
                        self.graph.add_edge(*admin_node, target, weight);
                    }
                }
            }
        }

        // Step 5: Remove any nodes no longer reachable from the entry.
        let mut walker = Dfs::new(&self.graph, self.entry);
        while walker.next(&self.graph).is_some() {}
        let mut to_remove = vec![];
        for node_id in self.graph.node_indices() {
            if !walker.discovered.contains(node_id.index()) {
                to_remove.push(node_id);
                assert_ne!(
                    node_id, self.exit,
                    "branch simplification removed the exit node!"
                );
            }
        }
        for node_id in to_remove {
            self.graph.remove_node(node_id);
        }
    }

    fn get_branch_metadata(&self) -> BranchMetadata {
        let mut res = BranchMetadata::default();
        for node in self.graph.node_indices() {
            let block = &self.graph[node];
            if block.footer.is_empty() && block.instrs.iter().all(is_admin_instr) {
                res.admin_nodes.insert(node);
            }
            for (id, lit) in block.instrs.iter().filter_map(constants_assigned) {
                res.constants_known.add_constant(node, id, lit);
            }
            let mut walker = self
                .graph
                .neighbors_directed(node, Direction::Outgoing)
                .detach();
            while let Some((edge, _)) = walker.next(&self.graph) {
                if let BranchOp::Cond { arg, val, .. } = &self.graph[edge].op {
                    res.branches
                        .entry(arg.clone())
                        .or_default()
                        .insert(edge, *val);
                }
            }
        }
        res
    }
}

#[derive(Default, Debug)]
struct BranchMetadata {
    /// Nodes that only contain administrative instructions.
    admin_nodes: IndexSet<NodeIndex>,
    /// Information about known constant values at particular nodes.
    constants_known: ConstantInfo,
    /// Relevant values used as branches.
    branches: IndexMap<Identifier, IndexMap<EdgeIndex, CondVal>>,
}

/// Constants with a known value as of a given node.
///
/// For now, the constants are always booleans, but we keep arbitrary
/// Literals around to make it easier to handle multi-way branches later.
#[derive(Default, Debug)]
struct ConstantInfo {
    by_node: IndexMap<NodeIndex, IndexMap<Identifier, Literal>>,
    by_id: IndexMap<Identifier, Vec<(NodeIndex, Literal)>>,
}

impl ConstantInfo {
    fn add_constant(&mut self, node: NodeIndex, id: Identifier, lit: Literal) {
        if self
            .by_node
            .entry(node)
            .or_default()
            .insert(id.clone(), lit.clone())
            .is_none()
        {
            self.by_id.entry(id).or_default().push((node, lit));
        }
    }
}

/// "Administrative Instructions" are ones that will have essentially no runtime
/// cost once they go through instruction selection / register allocation. We
/// use these as a heuristic to find blocks that are safe to merge into their
/// predecessors in exchange for simpler control flow: RVSDG conversion overhead
/// is largely contained in blocks only containing these instructions.
fn is_admin_instr(inst: &Instruction) -> bool {
    matches!(
        inst,
        Instruction::Constant { .. }
            | Instruction::Value {
                op: ValueOps::Id,
                ..
            }
    )
}

fn constants_assigned(inst: &Instruction) -> Option<(Identifier, Literal)> {
    if let Instruction::Constant {
        dest,
        value: value @ Literal::Bool(..),
        ..
    } = inst
    {
        Some((dest.into(), value.clone()))
    } else {
        None
    }
}

fn to_lit(cv: &CondVal) -> Option<Literal> {
    if cv.of == 2 {
        Some(if cv.val == 0 {
            Literal::Bool(false)
        } else {
            Literal::Bool(true)
        })
    } else {
        // Not handling multi-way branches for now.
        None
    }
}

/// A basic semilattice describing the state of a value.
#[derive(Clone, Default, Debug)]
enum ValueInfo {
    #[default]
    Bot,
    Known(Literal),
    Top,
}

impl ValueInfo {
    fn merge(&mut self, other: &ValueInfo) -> bool {
        match (self, other) {
            (ValueInfo::Bot, ValueInfo::Bot) => false,
            (slf @ ValueInfo::Bot, x) => {
                *slf = x.clone();
                true
            }
            (ValueInfo::Top, _) => false,
            (slf, ValueInfo::Top) => {
                *slf = ValueInfo::Top;
                true
            }
            (ValueInfo::Known(l), ValueInfo::Known(r)) if l == r => false,
            (slf @ ValueInfo::Known(_), ValueInfo::Known(_)) => {
                *slf = ValueInfo::Top;
                true
            }
            (ValueInfo::Known(_), ValueInfo::Bot) => false,
        }
    }
}

/// Monotone transforms on ValueInfos.
#[derive(Debug)]
enum Transform {
    Id,
    Negate,
    OverWrite(ValueInfo),
}

impl Transform {
    fn apply(&self, val: &ValueInfo) -> ValueInfo {
        match self {
            Transform::Id => val.clone(),
            Transform::Negate => match val {
                ValueInfo::Bot => ValueInfo::Bot,
                ValueInfo::Known(Literal::Bool(b)) => ValueInfo::Known(Literal::Bool(!b)),
                ValueInfo::Known(..) => ValueInfo::Top,
                ValueInfo::Top => ValueInfo::Top,
            },
            Transform::OverWrite(info) => info.clone(),
        }
    }
}

/// The state of the (boolean) values in a particular basic block.
#[derive(Default, Debug)]
struct ValueState {
    /// The (pointwise) join of all of the values in incoming branches.
    inherited: IndexMap<Identifier, ValueInfo>,
    /// The transforms induced by any operations on variables in the block.
    transforms: VecDeque<(
        Identifier, /* dst  */
        Identifier, /* src */
        Transform,
    )>,
    /// The set of variables written to in this basic block.
    kills: HashSet<Identifier>,
    /// The materialized output of transforms on inherited.
    outputs: IndexMap<Identifier, ValueInfo>,
    /// A variable indicating if `outputs` is stale.
    recompute: bool,
}

impl ValueState {
    /// Recompute the outputs for this state, if necessary.
    fn maybe_recompute(&mut self) -> bool {
        let res = self.recompute;
        if self.recompute {
            self.outputs.clear();
            for (id, info) in &self.inherited {
                self.outputs.insert(id.clone(), info.clone());
            }
            for (dst, src, transform) in &self.transforms {
                let src_val = self.outputs.get(src).unwrap_or(&ValueInfo::Bot);
                let dst_val = transform.apply(src_val);
                self.outputs.insert(dst.clone(), dst_val);
                self.kills.insert(dst.clone());
            }

            self.recompute = false;
        }
        res
    }
    fn outputs(&self) -> impl Iterator<Item = (&Identifier, &ValueInfo)> {
        assert!(!self.recompute);
        self.outputs.iter()
    }

    fn get_output(&self, id: &Identifier) -> ValueInfo {
        assert!(!self.recompute);
        self.outputs.get(id).cloned().unwrap_or(ValueInfo::Bot)
    }

    /// A special case of `merge_from` to handle self-loops.
    fn merge_self(&mut self) {
        let mut changed = false;
        for (id, out) in self.outputs.iter() {
            changed |= self.inherited.entry(id.clone()).or_default().merge(out);
        }
        if changed {
            self.recompute = true;
        }
    }

    /// Update the given inputs with the contents of `other`.
    fn merge_from(&mut self, other: &ValueState) {
        let mut changed = false;
        for (id, out) in other.outputs() {
            changed |= self.inherited.entry(id.clone()).or_default().merge(out);
        }
        if changed {
            self.recompute = true;
        }
    }

    /// Populate the ValueState with relevant instructions from the given basic
    /// block.
    fn new(block: &BasicBlock) -> ValueState {
        let mut transforms = VecDeque::new();
        for instr in &block.instrs {
            match instr {
                Instruction::Constant {
                    dest,
                    value: lit @ Literal::Bool(..),
                    ..
                } => {
                    // The `src` identifier is unused in this case.
                    transforms.push_back((
                        Identifier::from(dest.clone()),
                        Identifier::Num(usize::MAX),
                        Transform::OverWrite(ValueInfo::Known(lit.clone())),
                    ));
                }
                Instruction::Value {
                    args,
                    dest,
                    op,
                    op_type: Type::Bool,
                    ..
                } => match op {
                    ValueOps::Id => {
                        assert_eq!(args.len(), 1);
                        transforms.push_back((
                            Identifier::from(dest.clone()),
                            args[0].clone().into(),
                            Transform::Id,
                        ));
                    }
                    ValueOps::Not => {
                        assert_eq!(args.len(), 1);
                        transforms.push_back((
                            Identifier::from(dest.clone()),
                            args[0].clone().into(),
                            Transform::Negate,
                        ));
                    }
                    _ => {
                        transforms.push_back((
                            Identifier::from(dest.clone()),
                            Identifier::Num(usize::MAX),
                            Transform::OverWrite(ValueInfo::Top),
                        ));
                    }
                },
                Instruction::Effect { .. } => {}
                Instruction::Constant { .. } => {}
                Instruction::Value { .. } => {}
            }
        }
        ValueState {
            transforms,
            recompute: true,
            ..Default::default()
        }
    }
}

struct ValueAnalysis {
    data: HashMap<NodeIndex, ValueState>,
}

impl ValueAnalysis {
    fn new(graph: &SimpleCfgFunction) -> ValueAnalysis {
        let mut res = ValueAnalysis {
            data: Default::default(),
        };
        for node in graph.graph.node_indices() {
            res.data.insert(node, ValueState::new(&graph.graph[node]));
        }
        for Argument { name, arg_type } in &graph.args {
            if let Type::Bool = arg_type {
                let id = Identifier::from(name.clone());
                res.add_assignment(graph.entry, id, ValueInfo::Top);
            }
        }
        res
    }

    /// Prepend a virtual `id` instruction to the analysis for this node.
    fn add_assignment(&mut self, node: NodeIndex, dst: Identifier, val: ValueInfo) {
        let state = self.data.entry(node).or_default();
        state
            .transforms
            .push_front((dst, Identifier::Num(usize::MAX), Transform::OverWrite(val)));
        state.recompute = true;
    }

    /// A simple worklist algorithm for propagating values through the CFG.
    fn compute_fixpoint(&mut self, func: &SimpleCfgFunction) {
        let mut worklist = IndexSet::<NodeIndex>::default();
        for node in func.graph.node_indices() {
            self.data.entry(node).or_default().maybe_recompute();
            worklist.insert(node);
        }
        while let Some(node) = worklist.pop() {
            let mut cur = mem::take(self.data.get_mut(&node).unwrap());
            for incoming in func.graph.neighbors_directed(node, Direction::Incoming) {
                if incoming == node {
                    cur.merge_self();
                } else {
                    cur.merge_from(&self.data[&incoming]);
                }
            }
            let changed = cur.maybe_recompute();
            self.data.insert(node, cur);
            if changed {
                for outgoing in func.graph.neighbors_directed(node, Direction::Outgoing) {
                    worklist.insert(outgoing);
                }
            }
        }
    }

    /// Debugging routine for printing out the state of the analysis.
    #[allow(unused)]
    fn render(&self, func: &SimpleCfgFunction) -> String {
        let mut buf = Vec::<u8>::new();
        for (node, state) in &self.data {
            let name = &func.graph[*node].name;
            writeln!(buf, "{name}.inputs: {{").unwrap();
            for (id, info) in &state.inherited {
                writeln!(buf, "  {id:?}: {info:?}").unwrap();
            }
            writeln!(buf, "}}").unwrap();
            writeln!(buf, "{name}.outputs: {{").unwrap();
            for (id, info) in &state.outputs {
                writeln!(buf, "  {id:?}: {info:?}").unwrap();
            }
            writeln!(buf, "}}").unwrap();
        }
        String::from_utf8(buf).unwrap()
    }
}
