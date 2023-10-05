//! Conversion from (structured) CFG to RVSDG.
//!
//! This works by running a single pass (inspired by optir) over the
//! restructured CFG. The core idea is to "symbolically execute" the structured
//! CFG where unknown values (e.g. those of loop variables, or those of join
//! points) are replaced with references to either arguments to the enclosing
//! region or outputs from some region. To detect the start of loop regions, we
//! look for back-edges dominated by the current node. To detect the start of
//! branch regions, we look for nodes with more than one successor.

use bril_rs::{ConstOps, EffectOps, Instruction, Literal, Position, Type, ValueOps};
use hashbrown::HashMap;
use petgraph::algo::dominators;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use petgraph::{algo::dominators::Dominators, stable_graph::NodeIndex};

use crate::cfg::{ret_id, Annotation, BranchOp, CondVal, Identifier, SwitchCfgFunction};
use crate::rvsdg::Result;

use super::live_variables::{live_variables, Names};
use super::{
    live_variables::{LiveVariableAnalysis, VarId},
    Id, Operand, RvsdgBody, RvsdgError, RvsdgExpr,
};
use super::{RvsdgFunction, RvsdgType};

pub(crate) fn cfg_func_to_rvsdg(
    cfg: &mut SwitchCfgFunction,
    function_types: &FunctionTypes,
) -> Result<RvsdgFunction> {
    cfg.restructure();
    let analysis = live_variables(cfg);
    let dom = dominators::simple_fast(&cfg.graph, cfg.entry);
    let name = cfg.name.clone();
    let mut builder = RvsdgBuilder {
        cfg,
        expr: Default::default(),
        analysis,
        dom,
        store: Default::default(),
        function_types: function_types.clone(),
    };

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
    let result = match &builder.cfg.return_ty {
        Some(return_ty) => {
            let ret_var = builder.analysis.intern.intern(ret_id());
            Some((
                return_ty.clone(),
                get_op(ret_var, &None, &builder.store, &builder.analysis.intern)?,
            ))
        }
        None => None,
    };
    let n_args = builder.cfg.args.len();
    let state = builder.store[&state_var];

    let mut args: Vec<RvsdgType> = builder
        .cfg
        .args
        .iter()
        .map(|arg| RvsdgType::Bril(arg.arg_type.clone()))
        .collect();
    args.push(RvsdgType::PrintState);

    Ok(RvsdgFunction {
        name,
        n_args,
        args,
        nodes: builder.expr,
        result,
        state,
    })
}

/// FunctionTypes is a map from the name of the function
/// to the type of the function.
/// Bril doesn't have a void type, so this
/// is `None` when the function returns nothing.
pub(crate) type FunctionTypes = HashMap<String, Option<Type>>;

pub(crate) struct RvsdgBuilder<'a> {
    cfg: &'a mut SwitchCfgFunction,
    expr: Vec<RvsdgBody>,
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
                    let Some(mut dom) = self.dom.dominators(*pred) else { return false; };
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
            let Some(op) = self.store.get(&input).copied() else { continue; };
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
                    RvsdgBody::BasicOp(RvsdgExpr::Const(
                        ConstOps::Const,
                        Literal::Bool(true),
                        Type::Bool,
                    )),
                ))
            }
            BranchOp::Cond {
                arg,
                val: CondVal { val, of },
            } => {
                assert_eq!(
                    of, 2,
                    "loop predicate has more than two options (restructuring should avoid this)"
                );
                let var = self.analysis.intern.intern(arg);
                let op = get_op(var, &None, &self.store, &self.analysis.intern)?;
                if val == 0 {
                    // We need to negate the operand
                    Operand::Id(get_id(
                        &mut self.expr,
                        RvsdgBody::BasicOp(RvsdgExpr::Op(ValueOps::Not, vec![op], Type::Bool)),
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
        let placeholder = Identifier::Num(!0);
        let mut pred = placeholder.clone();
        let mut succs = Vec::from_iter(self.cfg.graph.edges_directed(block, Direction::Outgoing).map(|e| {
            if let BranchOp::Cond { arg, val: CondVal { val, of:_ }} = &e.weight().op {
                if pred == placeholder {
                    pred = arg.clone();
                }
                (*val, e.target())
            } else {
                panic!("Invalid mix of conditional and non-conditional branches in block {block:?}")
            }
        }));
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
        let mut input_vars = Vec::with_capacity(live_vars.live_in.len());
        let mut output_vars = Vec::new();
        for var in live_vars.live_in.iter() {
            let Some(op) = self.store.get(&var).copied() else { continue; };
            inputs.push(op);
            input_vars.push(var);
        }

        let mut next = None;
        for (_, succ) in succs {
            // First, make sure that all inputs are correctly bound to inputs to the block.
            for (i, var) in input_vars.iter().copied().enumerate() {
                self.store.insert(var, Operand::Arg(i));
            }
            // Loop until we reach a join point.
            let mut curr = succ;
            loop {
                curr = self.try_loop(curr)?.unwrap();
                if self
                    .cfg
                    .graph
                    .neighbors_directed(curr, Direction::Incoming)
                    .nth(1)
                    .is_some()
                {
                    break;
                }
            }

            // Use the join point's live outputs
            let live_vars = self.analysis.var_state(curr).unwrap();
            let mut output_vec = Vec::new();
            let fill_output = output_vars.is_empty();
            for var in live_vars.live_in.iter() {
                let Some(op) = self.store.get(&var).copied() else { continue; };
                output_vec.push(op);
                if fill_output {
                    output_vars.push(var);
                }
            }
            outputs.push(output_vec);
            if let Some(next) = next {
                assert_eq!(next, curr);
            } else {
                next = Some(curr);
            }
        }

        let next = next.unwrap();
        let pred_var = self.analysis.intern.intern(pred);
        let pred = get_op(
            pred_var,
            &self.cfg.graph[block].pos,
            &self.store,
            &self.analysis.intern,
        )?;
        let gamma_node = get_id(
            &mut self.expr,
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            },
        );
        // Remap all input variables to the output of this node.
        for (i, var) in output_vars.iter().copied().enumerate() {
            self.store.insert(var, Operand::Project(i, gamma_node));
        }

        Ok(Some(next))
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
                        RvsdgBody::BasicOp(RvsdgExpr::Const(
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
                    ValueOps::Alloc | ValueOps::Load | ValueOps::PtrAdd => {
                        return Err(RvsdgError::UnsupportedOperation {
                            op: *op,
                            pos: pos.clone(),
                        });
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
                            RvsdgExpr::Call((&funcs[0]).into(), ops, 2, Some(op_type.clone()));
                        let expr_id = get_id(&mut self.expr, RvsdgBody::BasicOp(expr));
                        self.store.insert(dest_var, Operand::Id(expr_id));
                        self.store
                            .insert(self.analysis.state_var, Operand::Project(1, expr_id));
                    }
                    _ => {
                        let dest_var = self.analysis.intern.intern(dest);
                        let ops = convert_args(args, &mut self.analysis, &mut self.store, pos)?;
                        let expr = RvsdgExpr::Op(*op, ops, op_type.clone());
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
                    let expr = RvsdgExpr::Call(
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
                    op: EffectOps::Print,
                    args,
                    pos,
                    ..
                } => {
                    let mut ops = convert_args(args, &mut self.analysis, &mut self.store, pos)?;
                    ops.push(self.store[&self.analysis.state_var]);
                    let expr = RvsdgExpr::Print(ops);
                    let expr_id = get_id(&mut self.expr, RvsdgBody::BasicOp(expr));
                    self.store
                        .insert(self.analysis.state_var, Operand::Id(expr_id));
                }
                Instruction::Effect { op, pos, .. } => {
                    // Two notes here:
                    // * Control flow like Return and Jmp _are_ supported, but
                    // the instructions should be eliminated as part of CFG
                    // conversion and they should instead show up as branches.
                    //
                    // * Print isn't supported (yet!) because it would require
                    // some form of "state" plumbing to ensure it is actually
                    // run.
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
                        RvsdgBody::BasicOp(RvsdgExpr::Const(
                            ConstOps::Const,
                            Literal::Int(*cond as i64),
                            Type::Int,
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
