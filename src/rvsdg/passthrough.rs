//! Implements the passthrough optimization on
//! RVSDGs.
//! We do this in rust and in egglog.
//! The egglog rules are slow, so we do it in rust.
//! However, the egglog rules can run in conjunction with other optimizations so they are still valuable.

use super::{Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};

impl RvsdgProgram {
    pub(crate) fn optimize_passthrough(&self) -> RvsdgProgram {
        let mut new_program: RvsdgProgram = self.clone();
        for func in new_program.functions.iter_mut() {
            *func = func.optimize_passthrough();
        }
        new_program
    }
}

impl RvsdgFunction {
    fn passthrough_operand(&self, operand: &Operand) -> Operand {
        match operand {
            Operand::Arg(_) => *operand,
            Operand::Project(ith, region) => {
                let region: &RvsdgBody = &self.nodes[*region];
                match region {
                    RvsdgBody::If {
                        pred: _,
                        inputs,
                        then_branch,
                        else_branch,
                    } => {
                        let arg = then_branch[*ith];
                        // both branches need to match
                        if arg == else_branch[*ith] {
                            match arg {
                                Operand::Arg(input_index) => {
                                    // pass through to this argument
                                    inputs[input_index]
                                }
                                _ => *operand,
                            }
                        } else {
                            *operand
                        }
                    }
                    RvsdgBody::Gamma {
                        pred: _,
                        inputs,
                        outputs,
                    } => {
                        let arg = outputs.first().unwrap()[*ith];
                        // rest of the outputs need to match
                        if outputs.iter().all(|output| output[*ith] == arg) {
                            match arg {
                                Operand::Arg(input_index) => {
                                    // pass through to this argument
                                    inputs[input_index]
                                }
                                _ => *operand,
                            }
                        } else {
                            *operand
                        }
                    }
                    RvsdgBody::Theta {
                        pred: _,
                        inputs,
                        outputs,
                    } => {
                        if let Operand::Arg(input_index) = outputs[*ith] {
                            // for loops, input_index needs to match ith
                            if input_index == *ith {
                                inputs[input_index]
                            } else {
                                *operand
                            }
                        } else {
                            *operand
                        }
                    }
                    _ => *operand,
                }
            }
        }
    }

    fn optimize_passthrough(&self) -> RvsdgFunction {
        let mut new_func: RvsdgFunction = self.clone();

        // until fixed point, make operands pass through regions
        let mut did_something = true;
        while did_something {
            did_something = false;

            // for each node
            for node in new_func.nodes.iter_mut() {
                let old_node = node.clone();
                node.map_operands(|operand| self.passthrough_operand(operand));
                if old_node != *node {
                    did_something = true;
                }
            }
        }

        // now remove all of these extra passed through nodes, which won't be referenced any more
        //TODO

        new_func
    }
}
