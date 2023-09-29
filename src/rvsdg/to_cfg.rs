use bril_rs::{ConstOps, EffectOps, Instruction, Type, ValueOps};

use hashbrown::HashMap;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;

use crate::{
    cfg::{
        BasicBlock, BlockName, Branch, BranchOp, CfgFunction, CfgProgram, Identifier,
        SimpleCfgFunction, SimpleCfgProgram,
    },
    util::{FreshNameGen, Visualization},
};

use super::{Expr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};

enum IncompleteBranch {
    /// Conditional jump from a given block to the next one
    Cond {
        from: NodeIndex,
        var: Identifier,
        val: bool,
    },
    /// Direct jump from a given block to the next one
    Direct { from: NodeIndex },
}

/// Represents the result of a RVSDG computation
#[derive(Clone, Debug)]
enum RvsdgValue {
    StateEdge,
    BrilValue(String, Type),
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

struct RvsdgToCfg<'a> {
    function: &'a RvsdgFunction,
    fresh_name: FreshNameGen,
    /// The cfg graph we are building
    graph: StableDiGraph<BasicBlock, Branch>,
    /// The instructions we have generated for the next block.
    current_instrs: Vec<Instruction>,
    /// When we finish a block or some blocks in the case of conditionals,
    /// we need to add a way to connect it to the next block.
    incomplete_branches: Vec<IncompleteBranch>,
    /// set the first time we create a block
    entry_block: Option<NodeIndex>,

    /// cache common sub-expressions so that we can re-use variables
    /// the Option<Id> is the context, which is important becuase
    /// arguments are different in different contexts
    /// The context is none at the top level
    operand_cache: HashMap<(Option<Id>, Operand), RvsdgValue>,
    body_cache: HashMap<(Option<Id>, Id), Vec<RvsdgValue>>,
}

impl RvsdgProgram {
    pub fn to_cfg(&self) -> SimpleCfgProgram {
        // TODO right now we only support one function
        // which is named main
        assert!(self.functions.len() == 1);
        CfgProgram {
            functions: self.functions.iter().map(|f| f.to_cfg()).collect(),
        }
    }
}

impl RvsdgFunction {
    pub fn to_cfg(&self) -> SimpleCfgFunction {
        let mut to_bril = RvsdgToCfg {
            function: self,
            fresh_name: FreshNameGen::new(),
            graph: Default::default(),
            current_instrs: vec![],
            incomplete_branches: vec![],
            entry_block: None,
            operand_cache: Default::default(),
            body_cache: Default::default(),
        };

        // TODO current args hardcoded to implicit print state
        to_bril.operand_to_bril(self.state, &vec![RvsdgValue::StateEdge], &None);
        if let Some(operand) = self.result {
            // it doesn't matter what var we assign to
            // TODO current args hardcoded to implicit print state
            let res = to_bril.operand_to_bril(operand, &vec![RvsdgValue::StateEdge], &None);
            to_bril.current_instrs.push(Instruction::Effect {
                op: EffectOps::Return,
                args: vec![res.unwrap_name()],
                funcs: vec![],
                labels: vec![],
                pos: None,
            });
        }
        let last_block = to_bril.finish_block();

        // TODO hard-coded name
        CfgFunction {
            name: "main".into(),
            args: vec![],
            graph: to_bril.graph,
            entry: to_bril.entry_block.unwrap(),
            exit: last_block,
            return_ty: None,
            phantom: Default::default(),
        }
    }
}

impl<'a> RvsdgToCfg<'a> {
    // Returns the name of the variable storing the result
    // or None if no value is returned.
    // The caller of this function should know what this
    // operand is being assigned to.
    fn operand_to_bril(
        &mut self,
        operand: Operand,
        current_args: &Vec<RvsdgValue>,
        context: &Option<Id>,
    ) -> RvsdgValue {
        if let Some(existing) = self.operand_cache.get(&(*context, operand)) {
            return existing.clone();
        }

        let res = match operand {
            Operand::Id(id) => {
                let res = self.body_to_bril(id, current_args, context);
                assert!(res.len() == 1);
                res[0].clone()
            }
            Operand::Arg(index) => current_args[index].clone(),
            Operand::Project(arg, id) => {
                let res = self.body_to_bril(id, current_args, context);
                res.get(arg)
                    .unwrap_or_else(|| {
                        panic!(
                            "Tried to project argument {arg} but only recieved {} arguments",
                            res.len()
                        )
                    })
                    .clone()
            }
        };

        self.operand_cache.insert((*context, operand), res.clone());
        res
    }

    // helper function to assigning to a set of variables
    // this is helpful in looping for loop variables or assigning to shared
    // variables across branches in a gamma
    fn assign_to_vars(&mut self, input_vars: &[RvsdgValue], resulting_vars: &[RvsdgValue]) {
        // assign to the loop variables, making sure the types line up
        for (ivar, rvar) in input_vars.iter().zip(resulting_vars.iter()) {
            match (ivar, rvar) {
                (RvsdgValue::StateEdge, RvsdgValue::StateEdge) => {}
                (RvsdgValue::BrilValue(oname, oty), RvsdgValue::BrilValue(lname, lty)) => {
                    assert_eq!(oty, lty);
                    self.current_instrs.push(Instruction::Value {
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

    /// The result of body_to_bril must comply with the assign_to input
    /// However, unlike AssignTo, there may be duplicate variables in the result
    /// when AssignTo doesn't specify the variable.
    fn body_to_bril(
        &mut self,
        id: Id,
        current_args: &Vec<RvsdgValue>,
        ctx: &Option<Id>,
    ) -> Vec<RvsdgValue> {
        // TODO share common sub-expressions
        let body = &self.function.nodes[id];
        match body {
            RvsdgBody::BasicOp(expr) => {
                vec![self.expr_to_bril(expr, current_args, ctx)]
            }
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => {
                let input_vars = inputs
                    .iter()
                    .map(|operand| self.operand_to_bril(*operand, current_args, &Some(id)))
                    .collect::<Vec<_>>();

                // evaluate pred in this block as well
                let pred = self.operand_to_bril(*pred, current_args, ctx);

                let prev_block = self.finish_block();

                let mut branch_blocks = vec![];
                let mut shared_vars = None;
                // for each set of outputs in outputs, make a new block for them
                for outputs in outputs {
                    // evaluate this branch
                    let resulting_outputs = outputs
                        .iter()
                        .map(|operand| self.operand_to_bril(*operand, &input_vars, &Some(id)))
                        .collect::<Vec<_>>();
                    if shared_vars.is_none() {
                        shared_vars = Some(self.fresh_variables_for(&resulting_outputs));
                    }
                    // assign to the shared vars
                    self.assign_to_vars(&resulting_outputs, shared_vars.as_ref().unwrap());

                    branch_blocks.push(self.finish_block());
                }

                // we need to conditionally jump to each of the branch blocks
                // based on the predicate
                // TODO right now we
                // just handle the case where we branch to two things
                assert!(outputs.len() == 2);
                assert!(branch_blocks.len() == 2);

                // add a conditional jump from the previous block to the branch blocks
                self.graph.add_edge(
                    prev_block,
                    branch_blocks[0],
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred.clone().unwrap_name()),
                            val: false.into(),
                        },
                        pos: None,
                    },
                );
                self.graph.add_edge(
                    prev_block,
                    branch_blocks[1],
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred.unwrap_name()),
                            val: true.into(),
                        },
                        pos: None,
                    },
                );

                // now we have all the branches, make incomplete jumps for each of them
                for branch_block in branch_blocks {
                    self.incomplete_branches
                        .push(IncompleteBranch::Direct { from: branch_block });
                }

                shared_vars.unwrap()
            }
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            } => {
                // evaluate the inputs
                let input_vars = inputs
                    .iter()
                    .map(|id| self.operand_to_bril(*id, current_args, ctx))
                    .collect::<Vec<_>>();
                // loop vars are like inputs, but we can't re-use inputs
                // because there may be duplicate names
                let loop_vars = self.fresh_variables_for(&input_vars);

                // assign to each loop var
                self.assign_to_vars(&input_vars, &loop_vars);

                // finish the block
                let prev_block = self.finish_block();
                self.incomplete_branches
                    .push(IncompleteBranch::Direct { from: prev_block });

                // now evaluate the outputs
                let output_vars = outputs
                    .iter()
                    .map(|operand| self.operand_to_bril(*operand, &loop_vars, &Some(id)))
                    .collect::<Vec<_>>();

                // then evalute the predicate
                let pred = self.operand_to_bril(*pred, current_args, &Some(id));

                // assign to the loop variables
                self.assign_to_vars(&output_vars, &loop_vars);

                let loop_block = self.finish_block();

                // add a conditional jump from the loop block back to itself
                self.graph.add_edge(
                    loop_block,
                    loop_block,
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred.clone().unwrap_name()),
                            val: true.into(),
                        },
                        pos: None,
                    },
                );

                // add a unfinished conditional jump to the next block
                self.incomplete_branches.push(IncompleteBranch::Cond {
                    from: prev_block,
                    var: Identifier::Name(pred.unwrap_name()),
                    val: false,
                });

                output_vars
            }
        }
    }

    fn finish_block(&mut self) -> NodeIndex {
        let instrs = std::mem::take(&mut self.current_instrs);
        let block = BasicBlock {
            instrs,
            footer: vec![],
            name: BlockName::Placeholder(self.fresh_name.fresh_usize()),
            pos: None,
        };
        let res = self.graph.add_node(block);
        if self.entry_block.is_none() {
            self.entry_block = Some(res);
        }

        // drain the queue of incomplete branches
        for branch in std::mem::take(&mut self.incomplete_branches) {
            match branch {
                IncompleteBranch::Cond { from, var, val } => {
                    self.graph.add_edge(
                        from,
                        res,
                        Branch {
                            op: BranchOp::Cond {
                                arg: var,
                                val: val.into(),
                            },
                            pos: None,
                        },
                    );
                }
                IncompleteBranch::Direct { from } => {
                    self.graph.add_edge(
                        from,
                        res,
                        Branch {
                            op: BranchOp::Jmp,
                            pos: None,
                        },
                    );
                }
            }
        }

        res
    }

    fn get_fresh(&mut self) -> String {
        self.fresh_name.fresh()
    }

    // Returns the name of the variable storing the result
    // of evaluating the expression.
    // This could be None when no value is returned,
    // as is the case when printing.
    fn expr_to_bril(
        &mut self,
        expr: &Expr<Operand>,
        current_args: &Vec<RvsdgValue>,
        ctx: &Option<Id>,
    ) -> RvsdgValue {
        match expr {
            Expr::Op(value_op, operands, ty) => {
                let operands = operands
                    .iter()
                    .map(|op| self.operand_to_bril(*op, current_args, ctx).unwrap_name())
                    .collect();
                let name = self.get_fresh();
                self.current_instrs.push(Instruction::Value {
                    dest: name.clone(),
                    op: *value_op,
                    args: operands,
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                    op_type: ty.clone(),
                });
                RvsdgValue::BrilValue(name, ty.clone())
            }
            Expr::Call(_name, _operands, _output_ports, _return_type_maybe) => {
                panic!("Not supported yet");
            }
            Expr::Const(_const_op, lit, ty) => {
                let dest = self.get_fresh();
                self.current_instrs.push(Instruction::Constant {
                    dest: dest.clone(),
                    op: ConstOps::Const,
                    value: lit.clone(),
                    pos: None,
                    const_type: ty.clone(),
                });
                RvsdgValue::BrilValue(dest, ty.clone())
            }
            Expr::Print(print_operands) => {
                assert!(print_operands.len() == 2);
                let operands = vec![self
                    .operand_to_bril(print_operands[0], current_args, ctx)
                    .unwrap_name()];
                // also need to evaluate other prints before this one
                self.operand_to_bril(print_operands[1], current_args, ctx);

                self.current_instrs.push(Instruction::Effect {
                    op: EffectOps::Print,
                    args: operands,
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                });
                RvsdgValue::StateEdge
            }
        }
    }
}
