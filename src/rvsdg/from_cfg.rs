//! Conversion from (structured) CFG to RVSDG.
//!
//! This works by running a single pass (inspired by optir) over the
//! restructured CFG. The core idea is to "symbolically execute" the structured
//! CFG where unknown values (e.g. those of loop variables, or those of join
//! points) are replaced with references to either arguments to the enclosing
//! region or outputs from some region. To detect the start of loop regions, we
//! look for back-edges dominated by the current node. To detect the start of
//! branch regions, we look for nodes with more than one successor.

use std::fs::File;
use std::io::Write;
use std::process::Command;

use bril_rs::{ConstOps, EffectOps, Instruction, Literal, Position, Type, ValueOps};
use hashbrown::HashMap;
use petgraph::algo::dominators;

use petgraph::dot::Dot;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use petgraph::{algo::dominators::Dominators, stable_graph::NodeIndex};
use smallvec::SmallVec;

use crate::cfg::{ret_id, Annotation, BranchOp, CondVal, SwitchCfgFunction};
use crate::rvsdg::Result;

use super::live_variables::{live_variables, Names, VarType};
use super::{
    live_variables::{LiveVariableAnalysis, VarId},
    BasicExpr, Id, Operand, RvsdgBody, RvsdgError,
};
use super::{RvsdgFunction, RvsdgType};

// When this value is true, we write out intermediate visualizations of the
// program. This is very helpful when debugging.
//
// We use a normal boolean here so as to not confuse various lints around unused
// imports, dead code, etc.

#[cfg(feature = "write-intermediates")]
const WRITE_INTERMEDIATES: bool = true;
#[cfg(not(feature = "write-intermediates"))]
const WRITE_INTERMEDIATES: bool = false;

pub(crate) fn cfg_func_to_rvsdg(
    cfg: &mut SwitchCfgFunction,
    function_types: &FunctionTypes,
) -> Result<RvsdgFunction> {
    if WRITE_INTERMEDIATES {
        File::create("/tmp/cfg-unstructured.dot")
            .unwrap()
            .write_fmt(format_args!(
                "{:#?}",
                Dot::new(&cfg.graph.map(|ni, n| (ni, n.clone()), |_, e| e.clone()))
            ))
            .unwrap();
        Command::new("dot")
            .arg("-Tpng")
            .arg("/tmp/cfg-unstructured.dot")
            .arg("-o")
            .arg("/tmp/cfg-unstructured.png")
            .output()
            .expect("failed to execute process");
    }
    cfg.restructure();
    if WRITE_INTERMEDIATES {
        File::create("/tmp/cfg-structured.dot")
            .unwrap()
            .write_fmt(format_args!(
                "{:#?}",
                Dot::new(&cfg.graph.map(|ni, n| (ni, n.clone()), |_, e| e.clone()))
            ))
            .unwrap();
        Command::new("dot")
            .arg("-Tpng")
            .arg("/tmp/cfg-structured.dot")
            .arg("-o")
            .arg("/tmp/cfg-structured.png")
            .output()
            .expect("failed to execute process");
    }
    let analysis = live_variables(cfg);
    let dom = dominators::simple_fast(&cfg.graph, cfg.entry);
    let name = cfg.name.clone();
    let mut builder = RvsdgBuilder {
        cfg,
        expr: Default::default(),
        analysis,
        dom,
        store: Default::default(),
        join_point: Default::default(),
        function_types: function_types.clone(),
    };

    let start = builder.cfg.entry;
    builder.compute_branch_info(vec![], start);

    let state_var = builder.analysis.state_var;

    for (i, arg) in builder.cfg.args.iter().enumerate() {
        let arg_var = builder.analysis.intern.intern(&arg.name);
        builder.store.insert(arg_var, Operand::Arg(i));
    }
    builder
        .store
        .insert(state_var, Operand::Arg(builder.cfg.args.len()));

    let mut cur = builder.cfg.entry;
    while let Some(next) = builder.try_loop(cur)? {
        if next == builder.cfg.exit {
            break;
        }
        cur = next;
    }
    let mut results = match &builder.cfg.return_ty {
        Some(return_ty) => {
            let ret_var = builder.analysis.intern.intern(ret_id());
            vec![(
                RvsdgType::Bril(return_ty.clone()),
                get_op(ret_var, &None, &builder.store, &builder.analysis.intern)?,
            )]
        }
        None => vec![],
    };
    results.push((RvsdgType::PrintState, builder.store[&state_var]));

    let mut args: Vec<RvsdgType> = builder
        .cfg
        .args
        .iter()
        .map(|arg| RvsdgType::Bril(arg.arg_type.clone()))
        .collect();
    args.push(RvsdgType::PrintState);

    let res = RvsdgFunction {
        name,
        args,
        nodes: builder.expr,
        results,
    };

    if WRITE_INTERMEDIATES {
        File::create("/tmp/rvsdg.svg")
            .unwrap()
            .write_all(res.to_svg().as_bytes())
            .unwrap();
    }

    Ok(res)
}

/// FunctionTypes is a map from the name of the function
/// to the type of the function.
/// Bril doesn't have a void type, so this
/// is `None` when the function returns nothing.
pub(crate) type FunctionTypes = HashMap<String, Option<Type>>;

pub(crate) struct RvsdgBuilder<'a> {
    cfg: &'a mut SwitchCfgFunction,
    expr: Vec<RvsdgBody>,
    // Maps from branch node to join point.
    join_point: HashMap<NodeIndex, NodeIndex>,
    analysis: LiveVariableAnalysis,
    dom: Dominators<NodeIndex>,
    store: HashMap<VarId, Operand>,
    function_types: FunctionTypes,
}

impl<'a> RvsdgBuilder<'a> {
    fn try_loop(&mut self, block: NodeIndex) -> Result<Option<NodeIndex>> {
        // First, check if this is the head of a loop. There are two cases here:
        //
        // 1. The loop is a single block, in which case this block will have
        // itself as a neighbor.
        let is_self_loop = self
            .cfg
            .graph
            .neighbors_directed(block, Direction::Outgoing)
            .any(|n| n == block);
        // 2. this is the start of a "do-while" loop. We can check this by seeing
        // if `block` dominates any of its incoming edges.
        let loop_tail = if is_self_loop {
            None
        } else {
            self.cfg
                .graph
                .neighbors_directed(block, Direction::Incoming)
                .find(|pred| {
                    let Some(mut dom) = self.dom.dominators(*pred) else {
                        return false;
                    };
                    dom.any(|n| n == block)
                })
        };

        if !is_self_loop && loop_tail.is_none() {
            // This is not a loop! Look at the other cases.
            return self.try_branch(block);
        }

        // First, we need to record the live operands going into the loop. These
        // are the loop inputs.
        let live_vars = self.analysis.var_state(block).unwrap();

        let mut input_vars = Vec::with_capacity(live_vars.live_in.len());
        let mut inputs = Vec::new();
        let pos = self.cfg.graph[block].pos.clone();
        let mut arg = 0;
        for input in live_vars.live_in.iter() {
            let Some(op) = self.store.get(&input).copied() else {
                continue;
            };
            input_vars.push(input);
            inputs.push(op);
            self.store.insert(input, Operand::Arg(arg));
            arg += 1;
        }

        // Now we "run" the loop until we reach the end:
        let tail = if let Some(tail) = loop_tail {
            let mut next = self.try_branch(block)?.unwrap();
            while next != tail {
                next = self.try_loop(next)?.unwrap();
            }
            tail
        } else {
            debug_assert!(is_self_loop);
            block
        };

        self.translate_block(tail)?;

        let mut outputs = Vec::with_capacity(inputs.len());
        for input in input_vars.iter().copied() {
            outputs.push(get_op(input, &pos, &self.store, &self.analysis.intern)?);
        }

        // Now to discover the loop predicate:
        let branches = Vec::from_iter(
            self.cfg
                .graph
                .edges_connecting(tail, block)
                .map(|e| e.weight().op.clone()),
        );

        if branches.len() != 1 {
            return Err(RvsdgError::UnsupportedLoopTail {
                pos: self.cfg.graph[tail].pos.clone(),
            });
        }

        let pred = match branches.into_iter().next().unwrap() {
            BranchOp::Jmp
            | BranchOp::Cond {
                val: CondVal { of: 1, .. },
                ..
            } => {
                // Predicate is just "true"
                Operand::Id(get_id(
                    &mut self.expr,
                    RvsdgBody::BasicOp(BasicExpr::Const(
                        ConstOps::Const,
                        Literal::Bool(true),
                        Type::Bool,
                    )),
                ))
            }
            BranchOp::Cond {
                arg,
                val: CondVal { val, of },
                bril_type,
            } => {
                assert_eq!(
                    of, 2,
                    "loop predicate has more than two options (restructuring should avoid this)"
                );
                assert_eq!(
                    bril_type,
                    Type::Bool,
                    "loop predicate is not a boolean in RVSDG translation"
                );
                let var = self.analysis.intern.intern(arg);
                let op = get_op(var, &None, &self.store, &self.analysis.intern)?;
                if val == 0 {
                    // We need to negate the operand
                    Operand::Id(get_id(
                        &mut self.expr,
                        RvsdgBody::BasicOp(BasicExpr::Op(ValueOps::Not, vec![op], Type::Bool)),
                    ))
                } else {
                    op
                }
            }
        };

        let theta_node = get_id(
            &mut self.expr,
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            },
        );

        for (i, var) in input_vars.iter().copied().enumerate() {
            self.store.insert(var, Operand::Project(i, theta_node));
        }
        Ok(self
            .cfg
            .graph
            .neighbors_directed(tail, Direction::Outgoing)
            .find(|succ| succ != &block))
    }

    fn try_branch(&mut self, block: NodeIndex) -> Result<Option<NodeIndex>> {
        self.translate_block(block)?;
        if self
            .cfg
            .graph
            .neighbors_directed(block, Direction::Outgoing)
            .nth(1)
            .is_none()
        {
            // This is a linear region
            return Ok(self
                .cfg
                .graph
                .neighbors_directed(block, Direction::Outgoing)
                .next());
        }

        let Some(join_point) = self.join_point.get(&block).copied() else {
            panic!("No join point for block {block:?}")
        };

        let mut succs_iter = self.cfg.graph.edges_directed(block, Direction::Outgoing);
        let mut succs = vec![];
        let first_e = succs_iter.next();
        // Bind pred, first_val, and bril_type from the first edge
        let Some(BranchOp::Cond {
            arg: pred,
            val: CondVal {
                val: first_val,
                of: _,
            },
            bril_type,
        }) = first_e.map(|e| e.weight().op.clone())
        else {
            panic!("Couldn't find a branch in block {block:?}");
        };
        succs.push((first_val, first_e.unwrap().target()));
        // for the rest of the edges, make sure pred and bril_type match up
        for e in succs_iter {
            if let BranchOp::Cond {
                arg,
                val: CondVal { val, of: _ },
                bril_type: other_bril_type,
            } = &e.weight().op
            {
                assert_eq!(
                    bril_type, *other_bril_type,
                    "Mismatched types in conditional branches in block {block:?}"
                );
                assert_eq!(pred, *arg, "Multiple predicates in block {block:?}");
                succs.push((*val, e.target()));
            } else {
                panic!("Invalid mix of conditional and non-conditional branches in block {block:?}")
            }
        }

        let pred_var = self.analysis.intern.intern(pred);
        let pred_op = get_op(
            pred_var,
            &self.cfg.graph[block].pos,
            &self.store,
            &self.analysis.intern,
        )?;
        succs.sort_by_key(|(val, _)| *val);
        // Branches should be contiguous.
        succs
            .iter()
            .enumerate()
            .for_each(|(i, (val, _))| assert_eq!(i, *val as usize));

        let mut inputs = Vec::<Operand>::new();
        let mut outputs = Vec::<Vec<Operand>>::new();
        let live_vars = self.analysis.var_state(block).unwrap();

        // Not all live variables have necessarily been bound yet.
        // `input_vars` and `output_vars` store the variables that are bound.
        let mut input_vars = Vec::with_capacity(live_vars.live_out.len());
        let mut output_vars = Vec::new();
        for var in live_vars.live_out.iter() {
            let Some(op) = self.store.get(&var).copied() else {
                continue;
            };
            inputs.push(op);
            input_vars.push(var);
        }

        for (_, succ) in succs {
            // First, make sure that all inputs are correctly bound to inputs to the block.
            for (i, var) in input_vars.iter().copied().enumerate() {
                self.store.insert(var, Operand::Arg(i));
            }
            // Loop until we reach a join point.
            let mut curr = succ;
            while curr != join_point {
                curr = self.try_loop(curr)?.unwrap();
            }

            // Use the join point's live outputs
            let live_vars = self.analysis.var_state(curr).unwrap();
            let mut output_vec = Vec::new();
            let fill_output = output_vars.is_empty();
            for var in live_vars.live_in.iter() {
                let op = self.store.get(&var).copied().unwrap_or_else(|| {
                    // We have a live variable input to the join point, but it's not bound in this branch.
                    // We need to bind it to some value; that value won't be
                    // reachable in the actual program so we could do anything.
                    let Some(ty) = self.analysis.var_types.get_type(var) else {
                        panic!(
                            "unknown type for variable {var:?} (name {:?}, join point {join_point:?})",
                            self.analysis.intern.get_var(var)
                        );
                    };
                    match ty {
                        VarType::Bril(bril_ty) => {
                            let lit = match bril_ty {
                                Type::Int => Literal::Int(0),
                                Type::Bool => Literal::Bool(false),
                                Type::Float => Literal::Float(0.0),
                                Type::Char => Literal::Char('x'),
                                Type::Pointer(_) => {
                                    unimplemented!("placeholder values for pointers aren't yet implemented")
                                },
                            };
                            let op = get_id(
                                &mut self.expr,
                                RvsdgBody::BasicOp(BasicExpr::Const(ConstOps::Const, lit, bril_ty)),
                            );
                            Operand::Project(0, op)
                        }
                        VarType::State => panic!("state variable unbound"),
                    }
                });
                output_vec.push(op);
                if fill_output {
                    output_vars.push(var);
                }
            }
            outputs.push(output_vec);
        }

        let pred = pred_op;
        let gamma_node = match bril_type {
            Type::Bool => {
                assert_eq!(
                    outputs.len(),
                    2,
                    "Found wrong number of branches for boolean.",
                );
                get_id(
                    &mut self.expr,
                    RvsdgBody::If {
                        pred,
                        inputs,
                        then_branch: outputs[1].clone(),
                        else_branch: outputs[0].clone(),
                    },
                )
            }
            Type::Int => {
                assert_eq!(
                    bril_type,
                    Type::Int,
                    "Branch predicate should be bool or integer"
                );
                get_id(
                    &mut self.expr,
                    RvsdgBody::Gamma {
                        pred,
                        inputs,
                        outputs,
                    },
                )
            }
            _ => panic!("Branch predicate should be bool or integer"),
        };
        // Remap all input variables to the output of this node.
        for (i, var) in output_vars.iter().copied().enumerate() {
            self.store.insert(var, Operand::Project(i, gamma_node));
        }

        Ok(Some(join_point))
    }

    fn translate_block(&mut self, block: NodeIndex) -> Result<()> {
        let block = &self.cfg.graph[block];

        fn convert_args(
            args: &[String],
            analysis: &mut LiveVariableAnalysis,
            env: &mut HashMap<VarId, Operand>,
            pos: &Option<Position>,
        ) -> Result<Vec<Operand>> {
            let mut ops = Vec::with_capacity(args.len());
            for arg in args {
                let arg_var = analysis.intern.intern(arg);
                let Some(arg_id) = env.get(&arg_var).copied() else {
                    return Err(RvsdgError::UndefinedId {
                        id: arg.into(),
                        pos: pos.clone(),
                    });
                };
                ops.push(arg_id);
            }
            Ok(ops)
        }

        for instr in &block.instrs {
            match instr {
                Instruction::Constant {
                    dest,
                    op,
                    const_type,
                    value,
                    ..
                } => {
                    let dest_var = self.analysis.intern.intern(dest);
                    let const_id = get_id(
                        &mut self.expr,
                        RvsdgBody::BasicOp(BasicExpr::Const(
                            *op,
                            value.clone(),
                            const_type.clone(),
                        )),
                    );
                    self.store.insert(dest_var, Operand::Id(const_id));
                }
                Instruction::Value {
                    args,
                    dest,
                    funcs,
                    labels: _,
                    op,
                    pos,
                    op_type,
                } => match op {
                    ValueOps::Load | ValueOps::Alloc => {
                        // NB: Load takes a state edge _and_ returns one. We
                        // could relax this later (making it easier to reorder
                        // loads, etc.), but only after confirming that it is
                        // safe wrt the model we're using. For example, in
                        // WebAssembly, loads can trap.
                        let dest_var = self.analysis.intern.intern(dest);
                        let mut ops = convert_args(args, &mut self.analysis, &mut self.store, pos)?;
                        ops.push(self.store[&self.analysis.state_var]);
                        let expr = BasicExpr::Op(*op, ops, op_type.clone());
                        let expr_id = get_id(&mut self.expr, RvsdgBody::BasicOp(expr));
                        self.store.insert(dest_var, Operand::Project(0, expr_id));
                        self.store
                            .insert(self.analysis.state_var, Operand::Project(1, expr_id));
                    }
                    ValueOps::Id => {
                        let dest_var = self.analysis.intern.intern(dest);
                        let src_var = self.analysis.intern.intern(&args[0]);
                        let Some(arg_id) = self.store.get(&src_var).copied() else {
                            return Err(RvsdgError::UndefinedId {
                                id: self.analysis.intern.get_var(src_var).clone(),
                                pos: pos.clone(),
                            });
                        };
                        self.store.insert(dest_var, arg_id);
                    }
                    ValueOps::Call => {
                        let dest_var = self.analysis.intern.intern(dest);
                        let mut ops = convert_args(args, &mut self.analysis, &mut self.store, pos)?;
                        ops.push(self.store[&self.analysis.state_var]);
                        let expr =
                            BasicExpr::Call((&funcs[0]).into(), ops, 2, Some(op_type.clone()));
                        let expr_id = get_id(&mut self.expr, RvsdgBody::BasicOp(expr));
                        self.store.insert(dest_var, Operand::Project(0, expr_id));
                        self.store
                            .insert(self.analysis.state_var, Operand::Project(1, expr_id));
                    }
                    _ => {
                        let dest_var = self.analysis.intern.intern(dest);
                        let ops = convert_args(args, &mut self.analysis, &mut self.store, pos)?;
                        let expr = BasicExpr::Op(*op, ops, op_type.clone());
                        let expr_id = get_id(&mut self.expr, RvsdgBody::BasicOp(expr));
                        self.store.insert(dest_var, Operand::Id(expr_id));
                    }
                },
                Instruction::Effect {
                    op: EffectOps::Nop, ..
                } => {}
                Instruction::Effect {
                    op: EffectOps::Call,
                    args,
                    funcs,
                    pos,
                    ..
                } => {
                    let mut ops = convert_args(args, &mut self.analysis, &mut self.store, pos)?;
                    ops.push(self.store[&self.analysis.state_var]);
                    let expr = BasicExpr::Call(
                        (&funcs[0]).into(),
                        ops,
                        1,
                        self.function_types
                            .get(&funcs[0])
                            .unwrap_or_else(|| panic!("unknown function {}", funcs[0]))
                            .clone(),
                    );
                    let expr_id = get_id(&mut self.expr, RvsdgBody::BasicOp(expr));
                    self.store
                        .insert(self.analysis.state_var, Operand::Id(expr_id));
                    debug_assert_eq!(funcs.len(), 1);
                }
                Instruction::Effect {
                    op: op @ (EffectOps::Print | EffectOps::Store | EffectOps::Free),
                    args,
                    pos,
                    ..
                } => {
                    let mut ops = convert_args(args, &mut self.analysis, &mut self.store, pos)?;
                    ops.push(self.store[&self.analysis.state_var]);
                    let expr = BasicExpr::Effect(*op, ops);
                    let expr_id = get_id(&mut self.expr, RvsdgBody::BasicOp(expr));
                    self.store
                        .insert(self.analysis.state_var, Operand::Id(expr_id));
                }
                Instruction::Effect { op, pos, .. } => {
                    // Note: Control flow like Return and Jmp _are_ supported,
                    // but the instructions should be eliminated as part of CFG
                    // conversion and they should instead show up as branches.
                    //
                    // Effects like `speculate` are truly unsupported
                    return Err(RvsdgError::UnsupportedEffect {
                        op: *op,
                        pos: pos.clone(),
                    });
                }
            }
        }

        for ann in &block.footer {
            match ann {
                Annotation::AssignCond { dst, cond } => {
                    let id = get_id(
                        &mut self.expr,
                        RvsdgBody::BasicOp(BasicExpr::Const(
                            ConstOps::Const,
                            match cond {
                                0 => bril_rs::Literal::Bool(false),
                                1 => bril_rs::Literal::Bool(true),
                                n => bril_rs::Literal::Int(*n as i64),
                            },
                            if *cond < 2 { Type::Bool } else { Type::Int },
                        )),
                    );
                    let dest_var = self.analysis.intern.intern(dst.clone());
                    self.store.insert(dest_var, Operand::Id(id));
                }
                Annotation::AssignRet { src } => {
                    let src_var = self.analysis.intern.intern(src.clone());
                    let ret_var = self.analysis.intern.intern(ret_id());
                    self.store.insert(
                        ret_var,
                        get_op(src_var, &block.pos, &self.store, &self.analysis.intern)?,
                    );
                }
            }
        }
        Ok(())
    }

    fn non_loop_neighbors(
        &self,
        cur: NodeIndex,
    ) -> (SmallVec<[NodeIndex; 2]>, SmallVec<[NodeIndex; 2]>) {
        let ins: SmallVec<[NodeIndex; 2]> = self
            .cfg
            .graph
            .neighbors_directed(cur, Direction::Incoming)
            .filter(|neigh| {
                !self
                    .dom
                    .dominators(*neigh)
                    .map(|mut doms| doms.any(|n| n == cur))
                    .unwrap_or(false)
            })
            .collect();
        let outs: SmallVec<[NodeIndex; 2]> = self
            .cfg
            .graph
            .neighbors_directed(cur, Direction::Outgoing)
            .filter(|neigh| {
                // Ignore back-edges: edges whose target dominates 'start'
                !self
                    .dom
                    .dominators(cur)
                    .map(|mut doms| doms.any(|n| n == *neigh))
                    .unwrap_or(false)
            })
            .collect();
        (ins, outs)
    }

    fn compute_branch_info(&mut self, mut last_branch: Vec<NodeIndex>, cur: NodeIndex) {
        // NB: we could get a big-O improvement by using a cons list. We could
        // even allocate nodes in an arena!
        let (ins, outs) = self.non_loop_neighbors(cur);
        if ins.len() > 1 {
            let branch = last_branch.pop().unwrap();
            assert_eq!(
                *self.join_point.entry(branch).or_insert(cur),
                cur,
                "Join point mismatch, branch={branch:?}"
            );
        }
        match outs.as_slice() {
            [] => {}
            [x] => self.compute_branch_info(last_branch, *x),
            rest => {
                last_branch.push(cur);
                for n in rest {
                    self.compute_branch_info(last_branch.clone(), *n)
                }
            }
        }
    }
}

fn get_id(exprs: &mut Vec<RvsdgBody>, body: RvsdgBody) -> Id {
    let id = exprs.len();
    exprs.push(body);
    id as Id
}

fn get_op(
    var: VarId,
    pos: &Option<Position>,
    env: &HashMap<VarId, Operand>,
    intern: &Names,
) -> Result<Operand> {
    match env.get(&var) {
        Some(op) => Ok(*op),
        None => Err(RvsdgError::UndefinedId {
            id: intern.get_var(var).clone(),
            pos: pos.clone(),
        }),
    }
}
