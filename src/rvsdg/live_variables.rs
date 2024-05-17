//! Basic live variable analysis for bril programs.
//!
//! The structure here follows that of optir, which in turn implements a (less
//! optimized) variant of the live variable analysis described in "Iterative
//! Data-flow Analysis, Revisited", by Keith D. Cooper, Timothy J. Harvey, and
//! Ken Kennedy.
use std::{collections::BTreeMap, fmt, mem};

use bril_rs::{self, EffectOps, Instruction, ValueOps};
use fixedbitset::FixedBitSet;
use hashbrown::HashMap;
use indexmap::IndexSet;
use petgraph::{
    stable_graph::NodeIndex,
    visit::{DfsPostOrder, VisitMap, Visitable},
    Direction,
};

use crate::{
    cfg::{
        ret_id, state_id, Annotation, BranchOp, CondVal, Identifier, NodeSet, SwitchCfgFunction,
    },
    util::ListDisplay,
};

pub(crate) fn live_variables(cfg: &SwitchCfgFunction) -> LiveVariableAnalysis {
    let mut analysis = LiveVariableAnalysis::default();
    let mut types = mem::take(&mut analysis.var_types);
    let mut names = mem::take(&mut analysis.intern);
    let mut dfs = DfsPostOrder::new(&cfg.graph, cfg.entry);
    let mut worklist = WorkList::new(cfg);
    if let Some(ty) = cfg.return_ty.clone().map(VarType::Bril) {
        let ret_var = names.intern(ret_id());
        types.set_var_type(ret_var, ty);
    }
    while let Some(block) = dfs.next(&cfg.graph) {
        let state = analysis.var_state_mut(block);
        let weight = &cfg.graph[block];

        if block == cfg.exit {
            state.gen.insert(names.intern(state_id()));
            if cfg.has_return_value() {
                // The exit block uses the return value, if there is one.
                state.gen.insert(names.intern(ret_id()));
            }
        }

        if block == cfg.entry {
            state.gen.insert(names.intern(state_id()));
        }

        // Live variable analysis is "bottom-up": we do everything in reverse
        // order. First, look at the branches:

        for edge in cfg.graph.edges_directed(block, Direction::Outgoing) {
            if let BranchOp::Cond {
                arg,
                val: CondVal { val: _, of },
                bril_type,
            } = &edge.weight().op
            {
                let var = names.intern(arg.clone());
                types.set_var_type(var, VarType::Bril(bril_type.clone()));
                if *of > 1 {
                    state.gen.insert(var);
                }
                // of == 1 is an unconditional jump.
            }
        }

        // Then the footers (in reverse order; though they shouldn't contain any
        // mutual dependencies).
        for ann in weight.footer.iter().rev() {
            match ann {
                Annotation::AssignCond { dst, .. } => {
                    let var = names.intern(dst.clone());
                    state.kills.insert(var);
                    state.gen.remove(var);
                }
                Annotation::AssignRet { src } => {
                    state.gen.insert(names.intern(src.clone()));
                    state.kills.insert(names.intern(ret_id()));
                }
            }
        }

        // Finally the instructions themselves.
        for instr in weight.instrs.iter().rev() {
            match instr {
                Instruction::Constant {
                    dest, const_type, ..
                } => {
                    let var = names.intern(dest);
                    types.set_var_type(var, VarType::Bril(const_type.clone()));
                    state.kills.insert(var);
                    state.gen.remove(var);
                }
                Instruction::Value {
                    args,
                    dest,
                    op,
                    op_type,
                    ..
                } => {
                    let dest = names.intern(dest);
                    types.set_var_type(dest, VarType::Bril(op_type.clone()));
                    state.kills.insert(dest);
                    state.gen.remove(dest);
                    for arg in args {
                        state.gen.insert(names.intern(arg));
                    }
                    if let ValueOps::Call = op {
                        state.kills.insert(names.intern(state_id()));
                        state.gen.insert(names.intern(state_id()));
                    }
                }
                Instruction::Effect { args, op, .. } => {
                    for arg in args {
                        state.gen.insert(names.intern(arg));
                    }

                    if let EffectOps::Print | EffectOps::Call = op {
                        state.kills.insert(names.intern(state_id()));
                        state.gen.insert(names.intern(state_id()));
                    }
                }
            }
        }
        worklist.push(block);
    }

    while let Some(block) = worklist.pop() {
        let mut changed = false;
        // Update live_out
        for succ in cfg.graph.neighbors(block) {
            changed |= analysis.union_out_in(block, succ);
        }

        // Update live_in
        let state = analysis.var_state_mut(block);
        changed |= state.live_in.merge(&state.gen);
        for x in state.live_out.vars.difference(&state.kills.vars) {
            changed |= state.live_in.insert(VarId(x as u32));
        }

        if changed {
            worklist.push(block);
            cfg.graph
                .neighbors_directed(block, Direction::Incoming)
                .for_each(|succ| worklist.push(succ))
        }
    }

    analysis.intern = names;
    analysis.var_types = types;
    analysis
}

/// An opaque Id representing a variable.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct VarId(u32);

#[derive(Default, Debug)]
pub(crate) struct Names {
    table: IndexSet<Identifier>,
}

impl Names {
    pub(crate) fn get_var(&self, id: VarId) -> &Identifier {
        self.table.get_index(id.0 as usize).unwrap()
    }

    pub(crate) fn intern(&mut self, name: impl Into<Identifier>) -> VarId {
        let name = name.into();
        if let Some(id) = self.table.get_index_of(&name) {
            return VarId(id as u32);
        }
        let id = u32::try_from(self.table.len()).unwrap();
        self.table.insert(name);
        VarId(id)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct VarSet {
    vars: FixedBitSet,
}

impl VarSet {
    pub(crate) fn iter(&self) -> impl Iterator<Item = VarId> + '_ {
        self.vars.ones().map(|x| VarId(x as u32))
    }

    pub(crate) fn len(&self) -> usize {
        self.vars.count_ones(..)
    }

    fn insert(&mut self, var: VarId) -> bool {
        let bit = var.0 as usize;
        if self.vars.len() <= bit {
            self.vars.grow(bit + 1);
        }
        !self.vars.put(bit)
    }

    fn remove(&mut self, var: VarId) {
        let bit = var.0 as usize;
        if self.vars.len() <= bit {
            return;
        }
        self.vars.set(bit, false);
    }

    pub(crate) fn merge(&mut self, other: &VarSet) -> bool {
        if other.vars.is_subset(&self.vars) {
            return false;
        }
        self.vars.union_with(&other.vars);
        true
    }

    /// Pretty-print the contents of the variable set with the un-interned
    /// identifiers given by `names`.
    pub(crate) fn render(&self, names: &Names) -> String {
        format!(
            "{}",
            ListDisplay(
                self.iter()
                    .map(|x| format!("{:?}", names.get_var(x)))
                    .collect::<Vec<_>>(),
                ", "
            )
        )
    }
}

/// The type of a variable, as computed during live variable analysis.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum VarType {
    Bril(bril_rs::Type),
    State,
}

/// The per-basic block state associated with the live variable analysis.
#[derive(Debug)]
pub(crate) struct LiveVariableState {
    /// The variables live on entry to a given basic block.
    pub(crate) live_in: VarSet,
    /// The variables live on exit from the basic block.
    pub(crate) live_out: VarSet,
    /// The variables written to in the basic block.
    kills: VarSet,
    /// The variables used before they are written to in the basic block.
    gen: VarSet,
}

struct StateAndNames<'a>(&'a LiveVariableState, &'a Names);

impl<'a> fmt::Debug for StateAndNames<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("in", &self.0.live_in.render(self.1))
            .field("out", &self.0.live_out.render(self.1))
            .field("gen", &self.0.gen.render(self.1))
            .field("kills", &self.0.kills.render(self.1))
            .finish()
    }
}

/// Type information recorded during live variable analysis.
#[derive(Default)]
pub(crate) struct VarTypes {
    data: HashMap<VarId, VarType>,
}

impl VarTypes {
    fn set_var_type(&mut self, var: VarId, ty: VarType) {
        assert_eq!(*self.data.entry(var).or_insert(ty.clone()), ty);
    }

    pub(crate) fn get_type(&self, var: VarId) -> Option<VarType> {
        self.data.get(&var).cloned()
    }
}

pub(crate) struct LiveVariableAnalysis {
    pub(crate) intern: Names,
    /// The variable associated with [`state_id`].
    pub(crate) state_var: VarId,
    analysis: HashMap<NodeIndex, LiveVariableState>,
    /// Live variable analysis computes the type of each variable while
    /// iterating through the CFG. We use this to instantiate placeholder values
    /// during RVSDG construction.
    pub(crate) var_types: VarTypes,
}

impl Default for LiveVariableAnalysis {
    fn default() -> Self {
        let mut result = LiveVariableAnalysis {
            intern: Names::default(),
            state_var: VarId(0),
            analysis: HashMap::default(),
            var_types: Default::default(),
        };
        let state_var = result.intern.intern(state_id());
        result.state_var = state_var;
        result.var_types.set_var_type(state_var, VarType::State);
        result
    }
}

impl fmt::Debug for LiveVariableAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        // Write output sorted.
        for (node, state) in self.analysis.iter().collect::<BTreeMap<_, _>>() {
            map.entry(&node.index(), &StateAndNames(state, &self.intern));
        }
        map.finish()?;
        Ok(())
    }
}

impl LiveVariableAnalysis {
    fn var_state_mut(&mut self, node: NodeIndex) -> &mut LiveVariableState {
        self.analysis.entry(node).or_insert_with(|| {
            let var_set = VarSet {
                vars: FixedBitSet::with_capacity(self.intern.table.len()),
            };
            LiveVariableState {
                live_in: var_set.clone(),
                live_out: var_set.clone(),
                kills: var_set.clone(),
                gen: var_set,
            }
        })
    }

    pub(crate) fn var_state(&self, node: NodeIndex) -> Option<&LiveVariableState> {
        self.analysis.get(&node)
    }

    /// Union pred's `live_out` set with succ's `live_in` set.
    fn union_out_in(&mut self, pred: NodeIndex, succ: NodeIndex) -> bool {
        let Some([pred_state, succ_state]) = self.analysis.get_many_mut([&pred, &succ]) else {
            return false;
        };
        pred_state.live_out.merge(&succ_state.live_in)
    }
}

struct WorkList {
    node_set: NodeSet,
    stack: Vec<NodeIndex>,
}

impl WorkList {
    fn new(cfg: &SwitchCfgFunction) -> WorkList {
        WorkList {
            node_set: cfg.graph.visit_map(),
            stack: Default::default(),
        }
    }

    fn push(&mut self, node: NodeIndex) {
        if self.node_set.visit(node) {
            self.stack.push(node);
        }
    }

    fn pop(&mut self) -> Option<NodeIndex> {
        let res = self.stack.pop()?;
        self.node_set.set(res.index(), false);
        Some(res)
    }
}
