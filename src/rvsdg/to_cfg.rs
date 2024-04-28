//! This module converts RVSDGs into bril CFGs (SimpleCfg).
//! It does it by doing a bottom-up translation of the RVSDG into a CFG,
//! recursively translating each demanded node.
//! When translating a RVSDG, it returns the variables that are assigned to the output ports.
//! These variables are guaranteed to be bound to the correct values anywhere in the CFG after that point.
//! To hook up the control flow, the translation keeps trackof [`IncompleteBranch`]es, since an RVSDG needs to return
//! control flow to the rest of the program after it is translated.
//!
//! In order to get sharing, we cache the resulting variable of each node.
//! However, this caching is context-sensative to the RVSDG body becuase
//! arguments refer to different arguments depending on the context.
//! The top-level context is None and other contexts are some Id corresponding to a Body.

use bril_rs::{Argument, ConstOps, EffectOps, Instruction, Literal, Type, ValueOps};

use hashbrown::HashMap;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;

use crate::{
    cfg::{
        BasicBlock, BlockName, Branch, BranchOp, CfgFunction, CfgProgram, Identifier, Simple,
        SimpleCfgFunction, SimpleCfgProgram,
    },
    util::FreshNameGen,
};

use super::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};

/// Represents the result of a RVSDG computation
#[derive(Clone, Debug)]
enum RvsdgValue {
    StateEdge,
    BrilValue(String, Type),
}

/// The return type of translating a part of an RVSDG
/// Translating an RVSDG node always creates a new
/// subgraph with a single entry and exit node (start and end).
/// These may be redundant or have a single input and output,
/// so TODO these should be removed in an optimization pass (Issue #66)
#[derive(Clone, Debug)]
struct TranslationResult {
    start: NodeIndex,
    end: NodeIndex,
    /// All RVSDG nodes return one or more values.
    /// After translating the node, these output values
    /// are bound to the following vector of [`RvsdgValue`]s.
    values: Vec<RvsdgValue>,
}

impl TranslationResult {
    pub(crate) fn get_single_res(&self) -> RvsdgValue {
        assert_eq!(self.values.len(), 1);
        self.values[0].clone()
    }
}

impl RvsdgValue {
    fn unwrap_name(&self) -> String {
        match self {
            RvsdgValue::StateEdge => panic!("Tried to unwrap state edge"),
            RvsdgValue::BrilValue(name, _) => name.clone(),
        }
    }

    fn unwrap_type(&self) -> Type {
        match self {
            RvsdgValue::StateEdge => panic!("Tried to unwrap state edge"),
            RvsdgValue::BrilValue(_, ty) => ty.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RvsdgContext {
    /// The [`RvsdgBody`] that we are inside of.
    /// None for the top-level
    body: Option<Id>,
    /// for gammas, we also need to store which branch we are in
    /// zero for other bodies
    branch: usize,
}

struct RvsdgToCfg<'a> {
    function: &'a RvsdgFunction,
    fresh_name: FreshNameGen,
    /// The cfg graph we are building
    graph: StableDiGraph<BasicBlock, Branch>,

    /// cache common sub-expressions so that we can re-use variables
    /// the Option<Id> is the context, which is important becuase
    /// arguments are different in different contexts
    /// The context is none at the top level
    operand_cache: HashMap<(RvsdgContext, Operand), Vec<RvsdgValue>>,
    body_cache: HashMap<(RvsdgContext, Id), Vec<RvsdgValue>>,
}

impl RvsdgProgram {
    pub fn to_cfg(&self) -> SimpleCfgProgram {
        CfgProgram {
            functions: self.functions.iter().map(|f| f.to_cfg()).collect(),
        }
        .optimize_jumps()
    }
}

impl RvsdgFunction {
    pub fn to_cfg(&self) -> SimpleCfgFunction {
        let mut to_bril = RvsdgToCfg {
            function: self,
            fresh_name: FreshNameGen::new(),
            graph: Default::default(),
            operand_cache: Default::default(),
            body_cache: Default::default(),
        };

        let mut func_args = vec![];
        let mut rvsdg_args = vec![];
        for arg in &self.args {
            match arg {
                RvsdgType::PrintState => {
                    rvsdg_args.push(RvsdgValue::StateEdge);
                }
                RvsdgType::Bril(ty) => {
                    let name = to_bril.get_fresh();
                    func_args.push(Argument {
                        name: name.clone(),
                        arg_type: ty.clone(),
                    });
                    rvsdg_args.push(RvsdgValue::BrilValue(name, ty.clone()));
                }
            }
        }

        let new_block = to_bril.make_block(vec![]);
        let mut result = TranslationResult {
            start: new_block,
            end: new_block,
            values: vec![],
        };

        let mut final_block = None;
        for (ty, operand) in self.results.iter() {
            // it doesn't matter what var we assign to
            // TODO current args hardcoded to implicit print state
            let new_block = to_bril.operand_to_bril(
                *operand,
                &rvsdg_args,
                &RvsdgContext {
                    body: None,
                    branch: 0,
                },
            );

            // Record the return value if possible
            if let RvsdgType::Bril(_) = ty {
                // We don't allow a function to return multiple values
                assert!(final_block.is_none());
                final_block = Some(to_bril.make_block(vec![Instruction::Effect {
                    op: EffectOps::Return,
                    args: vec![new_block.get_single_res().unwrap_name()],
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                }]));
            }

            result = to_bril.sequence_results(&[result, new_block]);
        }

        // If the function returns a value
        if let Some(final_block) = final_block {
            result = to_bril.sequence_results(&[
                result,
                TranslationResult {
                    start: final_block,
                    end: final_block,
                    values: vec![],
                },
            ]);
        }

        let return_ty = self
            .results
            .iter()
            .filter_map(|(ty, _)| {
                if let RvsdgType::Bril(ty) = ty {
                    Some(ty)
                } else {
                    None
                }
            })
            .next()
            .cloned();

        CfgFunction {
            name: self.name.clone(),
            args: func_args,
            graph: to_bril.graph,
            entry: result.start,
            exit: result.end,
            return_ty,
            _phantom: Simple,
        }
    }
}

impl<'a> RvsdgToCfg<'a> {
    /// Given two [`TranslationResult`]s, sequences them in the CFG
    /// and returns a new [`TranslationResult`] with the same values as the second.
    fn sequence_results(&mut self, results: &[TranslationResult]) -> TranslationResult {
        for (res1, res2) in results.iter().zip(results.iter().skip(1)) {
            self.graph.add_edge(
                res1.end,
                res2.start,
                Branch {
                    op: BranchOp::Jmp,
                    pos: None,
                },
            );
        }
        if results.is_empty() {
            TranslationResult {
                start: self.make_block(vec![]),
                end: self.make_block(vec![]),
                values: vec![],
            }
        } else {
            TranslationResult {
                start: results[0].start,
                end: results.last().unwrap().end,
                values: results[results.len() - 1].values.clone(),
            }
        }
    }

    /// Like [`SequenceResults`], but combines all of the results into
    /// a multi-output [`TranslationResult`].
    fn combine_results(&mut self, results: &[TranslationResult]) -> TranslationResult {
        // first sequence results
        let sequenced = self.sequence_results(results);
        // now make a vec of all the values
        let values = results.iter().map(|res| res.get_single_res()).collect();

        TranslationResult {
            start: sequenced.start,
            end: sequenced.end,
            values,
        }
    }

    /// Translates an operand to bril,
    /// returning a [`TranslationResult`] with the result of the operand.
    fn operand_to_bril(
        &mut self,
        operand: Operand,
        current_args: &Vec<RvsdgValue>,
        context: &RvsdgContext,
    ) -> TranslationResult {
        if let Some(existing) = self.operand_cache.get(&(context.clone(), operand)).cloned() {
            // make an empty block
            let new_block = self.make_block(vec![]);
            return TranslationResult {
                start: new_block,
                end: new_block,
                values: existing.clone(),
            };
        }

        let res = match operand {
            Operand::Id(id) => {
                let res = self.body_to_bril(id, current_args, context);
                TranslationResult {
                    start: res.start,
                    end: res.end,
                    values: vec![res.values[0].clone()],
                }
            }
            Operand::Arg(index) => {
                let new_block = self.make_block(vec![]);
                TranslationResult {
                    start: new_block,
                    end: new_block,
                    values: vec![current_args[index].clone()],
                }
            }
            Operand::Project(arg, id) => {
                let res = self.body_to_bril(id, current_args, context);
                TranslationResult {
                    start: res.start,
                    end: res.end,
                    values: vec![res.values[arg].clone()],
                }
            }
        };

        self.operand_cache
            .insert((context.clone(), operand), res.values.clone());
        res
    }

    // helper function to assigning to a set of variables
    // this is helpful in looping for loop variables or assigning to shared
    // variables across branches in a gamma
    fn assign_to_vars(
        &mut self,
        input_vars: &[RvsdgValue],
        resulting_vars: &[RvsdgValue],
    ) -> TranslationResult {
        let mut instructions = vec![];
        assert_eq!(input_vars.len(), resulting_vars.len());

        // assign to the variables, making sure the types line up
        for (ivar, rvar) in input_vars.iter().zip(resulting_vars.iter()) {
            match (ivar, rvar) {
                (RvsdgValue::StateEdge, RvsdgValue::StateEdge) => {}
                (RvsdgValue::BrilValue(oname, oty), RvsdgValue::BrilValue(lname, lty)) => {
                    assert_eq!(oty, lty);
                    instructions.push(Instruction::Value {
                        dest: lname.clone(),
                        op: ValueOps::Id,
                        args: vec![oname.clone()],
                        funcs: vec![],
                        labels: vec![],
                        pos: None,
                        op_type: oty.clone(),
                    });
                }
                _ => panic!(
                    "Incompatible values in assign_to_vars: {:?} {:?}",
                    ivar, rvar
                ),
            }
        }
        let block = self.make_block(instructions);
        TranslationResult {
            start: block,
            end: block,
            values: resulting_vars.to_vec(),
        }
    }

    fn fresh_variables_for(&mut self, values: &[RvsdgValue]) -> Vec<RvsdgValue> {
        values
            .iter()
            .map(|ivar| match ivar {
                RvsdgValue::StateEdge => ivar.clone(),
                RvsdgValue::BrilValue(_name, ty) => {
                    RvsdgValue::BrilValue(self.get_fresh(), ty.clone())
                }
            })
            .collect::<Vec<_>>()
    }

    fn cast_bool(&mut self, pred: &RvsdgValue) -> TranslationResult {
        let mut instructions = vec![];
        let new_val = if pred.unwrap_type() == Type::Int {
            let one = self.get_fresh();
            instructions.push(Instruction::Constant {
                dest: one.clone(),
                op: ConstOps::Const,
                value: Literal::Int(1),
                pos: None,
                const_type: Type::Int,
            });
            let new_name = self.get_fresh();
            instructions.push(Instruction::Value {
                dest: new_name.clone(),
                op: ValueOps::Eq,
                args: vec![pred.unwrap_name(), one],
                funcs: vec![],
                labels: vec![],
                pos: None,
                op_type: Type::Bool,
            });
            new_name
        } else {
            assert!(pred.unwrap_type() == Type::Bool);
            pred.unwrap_name()
        };
        let new_block = self.make_block(instructions);
        TranslationResult {
            start: new_block,
            end: new_block,
            values: vec![RvsdgValue::BrilValue(new_val, Type::Bool)],
        }
    }

    /// Translates a body to bril.
    /// This is a helper for `body_to_bril`, which retrieves cached
    /// results when available and calls `rvsdg_body_to_bril` otherwise.
    fn rvsdg_body_to_bril(
        &mut self,
        id: Id,
        body: &RvsdgBody,
        current_args: &Vec<RvsdgValue>,
        ctx: &RvsdgContext,
    ) -> TranslationResult {
        match body {
            RvsdgBody::BasicOp(expr) => self.expr_to_bril(expr, current_args, ctx),
            RvsdgBody::If {
                pred,
                inputs,
                then_branch,
                else_branch,
            } => {
                let switch = RvsdgBody::Gamma {
                    pred: *pred,
                    inputs: inputs.clone(),
                    outputs: vec![else_branch.clone(), then_branch.clone()],
                };
                // Desugar to a switch with a boolean predicate,
                // then translate the switch
                self.rvsdg_body_to_bril(id, &switch, current_args, ctx)
            }
            // Handles translation for If and Gamma
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => {
                let pred = self.operand_to_bril(*pred, current_args, ctx);
                // convert the predicate to a bool, since this might
                // actually be an `If`
                let pred_bool = self.cast_bool(&pred.get_single_res());
                let pred_res = self.sequence_results(&[pred, pred_bool.clone()]);

                let input_vars = inputs
                    .iter()
                    .map(|operand| self.operand_to_bril(*operand, current_args, ctx))
                    .collect::<Vec<_>>();
                // combine the inputs into a single result
                let inputs_combined = self.combine_results(&input_vars);

                // tranlation result for everything before the gamma
                let pred_and_inputs = self.sequence_results(&[pred_res, inputs_combined.clone()]);

                let mut branch_blocks = vec![];
                // shared vars will be created an the first iteraion
                let mut shared_vars = None;

                // for each set of outputs in outputs, make a new block for them
                for (i, outputs) in outputs.iter().enumerate() {
                    // evaluate this branch
                    let translation_results = outputs
                        .iter()
                        .map(|operand| {
                            self.operand_to_bril(
                                *operand,
                                &inputs_combined.values,
                                &RvsdgContext {
                                    body: Some(id),
                                    branch: i,
                                },
                            )
                        })
                        .collect::<Vec<_>>();
                    let outputs_for_branch = self.combine_results(&translation_results);

                    // make the shared vars on the first iteration
                    if shared_vars.is_none() {
                        shared_vars = Some(self.fresh_variables_for(&outputs_for_branch.values));
                    }
                    // assign to the shared vars
                    let output_assigned = self
                        .assign_to_vars(&outputs_for_branch.values, shared_vars.as_ref().unwrap());

                    branch_blocks
                        .push(self.sequence_results(&[outputs_for_branch, output_assigned]));
                }

                // we need to conditionally jump to each of the branch blocks
                // based on the predicate
                // TODO right now we
                // just handle the case where we branch to two things
                assert_eq!(outputs.len(), 2);
                assert_eq!(branch_blocks.len(), 2);

                // add a conditional jump from the previous block to the branch blocks
                self.graph.add_edge(
                    pred_and_inputs.end,
                    branch_blocks[0].start,
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred_bool.get_single_res().unwrap_name()),
                            val: false.into(),
                            bril_type: Type::Bool,
                        },
                        pos: None,
                    },
                );
                self.graph.add_edge(
                    pred_and_inputs.end,
                    branch_blocks[1].start,
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred_bool.get_single_res().unwrap_name()),
                            val: true.into(),
                            bril_type: Type::Bool,
                        },
                        pos: None,
                    },
                );

                // now make a block at the end
                let end_block = self.make_block(vec![]);

                // every branch jumps to the end block
                for branch_block in branch_blocks {
                    self.graph.add_edge(
                        branch_block.end,
                        end_block,
                        Branch {
                            op: BranchOp::Jmp,
                            pos: None,
                        },
                    );
                }

                TranslationResult {
                    start: pred_and_inputs.start,
                    end: end_block,
                    values: shared_vars.unwrap(),
                }
            }
            RvsdgBody::Theta {
                pred: pred_operand,
                inputs,
                outputs,
            } => {
                // for the Theta case, we
                // 1) evaluate the inputs
                // 2) assign these inputs to loop variables
                // 3) start a new block for the header, evaluating outputs
                // 4) add a footer to the block, assigning to the *same* loop variables
                // 5) finish the block with a loop back to the header

                // evaluate the inputs
                let input_vars = inputs
                    .iter()
                    .map(|id| self.operand_to_bril(*id, current_args, ctx))
                    .collect::<Vec<_>>();
                let inputs_combined = self.combine_results(&input_vars);
                // loop vars are like inputs, but we can't re-use inputs
                // because there may be duplicate names
                let loop_vars = self.fresh_variables_for(&inputs_combined.values);
                // assign to each loop var
                let assigned = self.assign_to_vars(&inputs_combined.values, &loop_vars);
                let before_block = self.sequence_results(&[inputs_combined, assigned]);

                let mycontext = RvsdgContext {
                    body: Some(id),
                    branch: 0,
                };
                // now evaluate the outputs
                let output_vars = outputs
                    .iter()
                    .map(|operand| self.operand_to_bril(*operand, &loop_vars, &mycontext))
                    .collect::<Vec<_>>();
                let outputs_combined = self.combine_results(&output_vars);

                // then evalute the predicate
                let pred = self.operand_to_bril(*pred_operand, &loop_vars, &mycontext);
                // convert to a boolean if needed
                let pred_bool = self.cast_bool(&pred.get_single_res());
                // assign to the loop variables
                let assign_to_loop = self.assign_to_vars(&outputs_combined.values, &loop_vars);

                // combine all these into a loop body
                let loop_body = self.sequence_results(&[
                    outputs_combined,
                    pred,
                    pred_bool.clone(),
                    assign_to_loop,
                ]);

                // make an edge from before the loop to the loop
                self.sequence_results(&[before_block.clone(), loop_body.clone()]);

                // now make a block for the loop footer
                let loop_footer_block = self.make_block(vec![]);
                let loop_footer = TranslationResult {
                    start: loop_footer_block,
                    end: loop_footer_block,
                    values: loop_vars.clone(),
                };

                // add a conditional jump from the loop block back to header
                self.graph.add_edge(
                    loop_body.end,
                    loop_body.start,
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred_bool.get_single_res().unwrap_name()),
                            val: true.into(),
                            bril_type: Type::Bool,
                        },
                        pos: None,
                    },
                );
                // otherwise go to footer
                self.graph.add_edge(
                    loop_body.end,
                    loop_footer.start,
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred_bool.get_single_res().unwrap_name()),
                            val: false.into(),
                            bril_type: Type::Bool,
                        },
                        pos: None,
                    },
                );

                TranslationResult {
                    start: before_block.start,
                    end: loop_footer.end,
                    values: loop_vars,
                }
            }
        }
    }

    /// The result of body_to_bril must comply with the assign_to input
    /// However, unlike AssignTo, there may be duplicate variables in the result
    /// when AssignTo doesn't specify the variable.
    fn body_to_bril(
        &mut self,
        id: Id,
        current_args: &Vec<RvsdgValue>,
        ctx: &RvsdgContext,
    ) -> TranslationResult {
        if let Some(existing) = self.body_cache.get(&(ctx.clone(), id)).cloned() {
            let new_block = self.make_block(vec![]);
            return TranslationResult {
                start: new_block,
                end: new_block,
                values: existing.clone(),
            };
        }

        let body = &self.function.nodes[id];
        let res = self.rvsdg_body_to_bril(id, body, current_args, ctx);

        self.body_cache
            .insert((ctx.clone(), id), res.values.clone());

        res
    }

    fn make_block(&mut self, instrs: Vec<Instruction>) -> NodeIndex {
        let block = BasicBlock {
            instrs,
            footer: vec![],
            name: BlockName::Placeholder(self.fresh_name.fresh_usize()),
            pos: None,
        };
        self.graph.add_node(block)
    }

    fn get_fresh(&mut self) -> String {
        self.fresh_name.fresh()
    }

    /// Translates an expression to bril.
    /// Again, creates new blocks no matter what, but these can be optimized
    /// away in another pass.
    fn expr_to_bril(
        &mut self,
        expr: &BasicExpr<Operand>,
        current_args: &Vec<RvsdgValue>,
        ctx: &RvsdgContext,
    ) -> TranslationResult {
        match expr {
            BasicExpr::Op(value_op, operands, ty) => {
                let mut operand_results = operands
                    .iter()
                    .map(|op| self.operand_to_bril(*op, current_args, ctx))
                    .collect::<Vec<_>>();

                let name = self.get_fresh();
                match value_op {
                    ValueOps::Alloc | ValueOps::Load => {
                        let non_state_results = &operand_results[0..operand_results.len() - 1];
                        let operands = self.combine_results(non_state_results);
                        let args: Vec<String> =
                            operands.values.iter().map(|v| v.unwrap_name()).collect();
                        let new_block = self.make_block(vec![Instruction::Value {
                            args,
                            dest: name.clone(),
                            funcs: vec![],
                            labels: vec![],
                            op: *value_op,
                            pos: None,
                            op_type: ty.clone(),
                        }]);
                        let new_res = TranslationResult {
                            start: new_block,
                            end: new_block,
                            values: vec![
                                RvsdgValue::BrilValue(name, ty.clone()),
                                RvsdgValue::StateEdge,
                            ],
                        };
                        operand_results.push(new_res);
                        self.sequence_results(&operand_results)
                    }
                    _ => {
                        let operands = self.combine_results(&operand_results);
                        let new_block = self.make_block(vec![Instruction::Value {
                            dest: name.clone(),
                            op: *value_op,
                            args: operands.values.iter().map(|v| v.unwrap_name()).collect(),
                            funcs: vec![],
                            labels: vec![],
                            pos: None,
                            op_type: ty.clone(),
                        }]);
                        let new_res = TranslationResult {
                            start: new_block,
                            end: new_block,
                            values: vec![RvsdgValue::BrilValue(name, ty.clone())],
                        };
                        self.sequence_results(&[operands, new_res])
                    }
                }
            }
            BasicExpr::Call(func_name, operands, n_outs, return_type) => {
                // We need to remove the last argument if the function call is stateful.
                // let operand_len = operands.len() - ((!pure) as usize);
                let operand_results = operands
                    .iter()
                    .map(|op| self.operand_to_bril(*op, current_args, ctx))
                    .collect::<Vec<_>>();
                let operands = self.combine_results(&operand_results);
                // Keep only value arguments
                let args = operands
                    .values
                    .iter()
                    .flat_map(|v| {
                        if let RvsdgValue::BrilValue(name, _ty) = v {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                let var_name = self.get_fresh();
                match return_type {
                    Some(ty) => {
                        let instructions = vec![Instruction::Value {
                            args,
                            dest: var_name.clone(),
                            funcs: vec![func_name.clone()],
                            labels: vec![],
                            op: ValueOps::Call,
                            pos: None,
                            op_type: ty.clone(),
                        }];
                        let new_block = self.make_block(instructions);
                        let mut values = vec![RvsdgValue::BrilValue(var_name, ty.clone())];
                        for _ in 1..*n_outs {
                            values.push(RvsdgValue::StateEdge);
                        }
                        let new_res = TranslationResult {
                            start: new_block,
                            end: new_block,
                            values,
                        };
                        self.sequence_results(&[operands, new_res])
                    }
                    None => {
                        let instructions = vec![Instruction::Effect {
                            args,
                            funcs: vec![func_name.clone()],
                            labels: vec![],
                            op: EffectOps::Call,
                            pos: None,
                        }];
                        let new_block = self.make_block(instructions);
                        let new_res = TranslationResult {
                            start: new_block,
                            end: new_block,
                            values: vec![RvsdgValue::StateEdge; *n_outs],
                        };
                        self.sequence_results(&[operands, new_res])
                    }
                }
            }
            BasicExpr::Const(_const_op, lit, ty) => {
                let dest = self.get_fresh();
                let instructions = vec![Instruction::Constant {
                    dest: dest.clone(),
                    op: ConstOps::Const,
                    value: lit.clone(),
                    pos: None,
                    const_type: ty.clone(),
                }];
                let new_block = self.make_block(instructions);
                TranslationResult {
                    start: new_block,
                    end: new_block,
                    values: vec![RvsdgValue::BrilValue(dest, ty.clone())],
                }
            }
            BasicExpr::Effect(op, args) => {
                let mut results: Vec<TranslationResult> = args
                    .iter()
                    .map(|op| self.operand_to_bril(*op, current_args, ctx))
                    .collect();
                // Combine all arguments but the state edge.
                let main_result = self.combine_results(&results[0..results.len() - 1]);
                let args: Vec<String> = main_result
                    .values
                    .iter()
                    .map(|res| res.unwrap_name())
                    .collect();
                let new_block = self.make_block(vec![Instruction::Effect {
                    op: *op,
                    args,
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                }]);
                self.sequence_results(&[
                    main_result,
                    results.pop().unwrap(),
                    TranslationResult {
                        start: new_block,
                        end: new_block,
                        values: vec![RvsdgValue::StateEdge],
                    },
                ])
            }
        }
    }
}
