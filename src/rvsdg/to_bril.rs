use bril_rs::{Code, ConstOps, EffectOps, Function, Instruction, Program};
use hashbrown::HashMap;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;

use crate::{
    cfg::{BasicBlock, BlockName, Branch, BranchOp, CfgFunction, CfgProgram, Identifier},
    util::FreshNameGen,
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
    // set the first time we create a block
    entry_block: Option<NodeIndex>,
}

impl RvsdgProgram {
    pub fn to_cfg(&self) -> CfgProgram {
        // TODO right now we only support one function
        // which is named main
        assert!(self.functions.len() == 1);
        CfgProgram {
            functions: self.functions.iter().map(|f| f.to_cfg()).collect(),
        }
    }
}

impl RvsdgFunction {
    pub fn to_cfg(&self) -> CfgFunction {
        let mut to_bril = RvsdgToCfg {
            function: self,
            fresh_name: FreshNameGen::new(),
            graph: Default::default(),
            current_instrs: vec![],
            incomplete_branches: vec![],
            entry_block: None,
        };

        to_bril.operand_to_bril(self.state, &AssignTo::Nothing);
        if let Some(operand) = self.result {
            // it doesn't matter what var we assign to
            let res = to_bril.operand_to_bril(operand, &AssignTo::Var(None));
            assert!(res.is_some());
            to_bril.current_instrs.push(Instruction::Effect {
                op: EffectOps::Return,
                args: vec![res.unwrap()],
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
        }
    }
}

#[derive(Debug, Clone)]
enum AssignTo {
    /// for effects like print, we don't assign to anything
    Nothing,
    /// assign to a single variable. If the variable is None,
    /// it doesn't matter.
    Var(Option<String>),
    /// assign to multiple variables. If the variable is None,
    /// it doesn't matter what name is chosen for it.
    /// if the length of vars is less than the number
    /// of outputs that's okay, count them as None
    /// Finally, no two variable names can be the same
    Vars(Vec<Option<String>>),
}

impl<'a> RvsdgToCfg<'a> {
    // Returns the name of the variable storing the result
    // or None if no value is returned.
    // The caller of this function should know what this
    // operand is being assigned to.
    fn operand_to_bril(&mut self, operand: Operand, assign_to: &AssignTo) -> Option<String> {
        match (operand, assign_to) {
            (Operand::Id(id), _) => {
                let res = self.body_to_bril(id, assign_to);
                if res.len() == 1 {
                    res[0].clone()
                } else if res.is_empty() {
                    None
                } else {
                    panic!("Got multiple outputs for an operand!");
                }
            }
            (Operand::Arg(_), _) => panic!("args not supported yet"),
            (Operand::Project(arg, id), AssignTo::Var(v)) => {
                let mut assign_to = vec![None; arg + 1];
                assign_to[arg] = v.clone();
                let res = self.body_to_bril(id, &AssignTo::Vars(assign_to));
                res.get(arg)
                    .unwrap_or_else(|| {
                        panic!(
                            "Tried to project argument {arg} but only recieved {} arguments",
                            res.len()
                        )
                    })
                    .clone()
            }
            _ => panic!("AssignTo was invalid: {:?} vs {:?}", operand, assign_to),
        }
    }

    fn body_to_bril(&mut self, id: Id, assign_to: &AssignTo) -> Vec<Option<String>> {
        // TODO share common sub-expressions
        let body = &self.function.nodes[id];
        match body {
            RvsdgBody::BasicOp(expr) => {
                vec![self.expr_to_bril(expr, assign_to)]
            }
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => todo!(),
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            } => {
                assert!(inputs.len() == outputs.len());
                // make variables for all the inputs
                let mut input_vars: Vec<Option<String>> = (0..inputs.len())
                    .map(|_| Some(self.fresh_name.fresh()))
                    .collect();

                let AssignTo::Vars(vars_todo) = assign_to else {
                    panic!("Expected AssignTo::Vars for theta, but got {:?}", assign_to);
                };

                // make input_vars comply with the input assign_to
                for (i, var) in vars_todo.iter().enumerate() {
                    if let Some(var) = var {
                        input_vars[i] = Some(var.clone());
                    }
                }

                let new_assign_to = AssignTo::Vars(input_vars.clone());

                // evaluate the inputs
                let input_vars_resulting = inputs
                    .iter()
                    .map(|id| self.operand_to_bril(*id, &new_assign_to))
                    .collect::<Vec<_>>();

                assert_eq!(input_vars, input_vars_resulting);
                // finish the block
                let prev_block = self.finish_block();
                self.incomplete_branches
                    .push(IncompleteBranch::Direct { from: prev_block });

                // now evaluate the outputs but with the same variables
                let output_vars = outputs
                    .iter()
                    .map(|id| self.operand_to_bril(*id, &new_assign_to))
                    .collect::<Vec<_>>();

                // then evalute the predicate
                let pred = self.operand_to_bril(*pred, &AssignTo::Var(None));

                let loop_block = self.finish_block();

                // add a conditional jump from the loop block back to itself
                self.graph.add_edge(
                    loop_block,
                    loop_block,
                    Branch {
                        op: BranchOp::Cond {
                            arg: Identifier::Name(pred.clone().unwrap()),
                            val: true.into(),
                        },
                        pos: None,
                    },
                );

                // add a unfinished conditional jump to the next block
                self.incomplete_branches.push(IncompleteBranch::Cond {
                    from: prev_block,
                    var: Identifier::Name(pred.unwrap()),
                    val: false,
                });

                assert_eq!(output_vars, input_vars);

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

    fn get_name(&mut self, assign_to: &AssignTo) -> String {
        if let AssignTo::Var(Some(name)) = assign_to {
            name.clone()
        } else if let AssignTo::Var(None) = assign_to {
            self.fresh_name.fresh()
        } else {
            panic!(
                "Expected a single variable to assign to, but got {:?}",
                assign_to
            );
        }
    }

    // Returns the name of the variable storing the result
    // of evaluating the expression.
    // This could be None when no value is returned,
    // as is the case when printing.
    fn expr_to_bril(&mut self, expr: &Expr<Operand>, assign_to: &AssignTo) -> Option<String> {
        match expr {
            Expr::Op(value_op, operands, ty) => {
                let operands = operands
                    .iter()
                    // filter map to get rid of None values
                    // from the implicit print state
                    .filter_map(|op| self.operand_to_bril(*op, &AssignTo::Var(None)))
                    .collect();
                let name = self.get_name(assign_to);
                self.current_instrs.push(Instruction::Value {
                    dest: name.clone(),
                    op: *value_op,
                    args: operands,
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                    op_type: ty.clone(),
                });
                Some(name)
            }
            Expr::Call(_name, _operands, _output_ports, _return_type_maybe) => {
                panic!("Not supported yet");
            }
            Expr::Const(_const_op, lit, ty) => {
                let dest = self.get_name(assign_to);
                self.current_instrs.push(Instruction::Constant {
                    dest: dest.clone(),
                    op: ConstOps::Const,
                    value: lit.clone(),
                    pos: None,
                    const_type: ty.clone(),
                });
                Some(dest)
            }
            Expr::Print(operands) => {
                if let AssignTo::Nothing = assign_to {
                } else {
                    panic!(
                        "Expected AssignTo::Nothing for print, but got {:?}",
                        assign_to
                    );
                }
                let operands = operands
                    .iter()
                    .filter_map(|op| self.operand_to_bril(*op, &AssignTo::Var(None)))
                    .collect();
                self.current_instrs.push(Instruction::Effect {
                    op: EffectOps::Print,
                    args: operands,
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                });
                None
            }
        }
    }
}
