//! Implements the passthrough optimization on
//! RVSDGs.
//! We do this in rust and in egglog.
//! The egglog rules are slow, so we do it in rust.
//! However, the egglog rules can run in conjunction with other optimizations so they are still valuable.

use hashbrown::HashMap;

use super::{Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};

impl RvsdgProgram {
    pub(crate) fn optimize_passthrough(&self) -> RvsdgProgram {
        let mut new_program: RvsdgProgram = self.clone();
        for func in new_program.functions.iter_mut() {
            let mut did_something = true;
            while did_something {
                let (new_func, changed) = func.optimize_passthrough();
                did_something = changed;
                *func = new_func;
            }
        }
        new_program
    }
}

impl RvsdgFunction {
    fn operand_remove_ith_arg(
        &mut self,
        operand: &Operand,
        ith: usize,
        cache: &mut HashMap<Id, bool>,
        perform_removal: bool,
    ) -> (bool, Operand) {
        match operand {
            Operand::Project(_, id) => (
                self.remove_ith_arg(*id, ith, cache, perform_removal),
                *operand,
            ),
            Operand::Arg(arg_index) => match arg_index.cmp(&ith) {
                std::cmp::Ordering::Greater => (
                    true,
                    if perform_removal {
                        Operand::Arg(arg_index - 1)
                    } else {
                        *operand
                    },
                ),
                std::cmp::Ordering::Less => (true, *operand),
                std::cmp::Ordering::Equal => {
                    assert!(!perform_removal);
                    (false, *operand)
                }
            },
        }
    }
    /// Checks if there are no references to the
    /// ith argument starting from this node.
    fn remove_ith_arg(
        &mut self,
        node_id: Id,
        ith: usize,
        cache: &mut HashMap<Id, bool>,
        perform_removal: bool,
    ) -> bool {
        if let Some(&result) = cache.get(&node_id) {
            return result;
        }
        let mut can_remove = true;
        let new_node = self.nodes[node_id]
            .clone()
            .map_same_region_operands(&mut |operand| {
                let (result, new_operand) =
                    self.operand_remove_ith_arg(operand, ith, cache, perform_removal);
                can_remove = can_remove && result;
                new_operand
            });

        self.nodes[node_id] = new_node;

        cache.insert(node_id, can_remove);
        can_remove
    }

    /// Given a node id and an operand index,
    /// check if the node is a region and the index is a passed through value.
    /// If so, remove it without fixing up any references to the node.
    /// Returns the input operand that was passed through if successful.
    fn passthrough_operand(&mut self, node_id: Id, ith: usize) -> Option<Operand> {
        let region: RvsdgBody = self.nodes[node_id].clone();
        match region {
            RvsdgBody::If {
                pred,
                mut inputs,
                mut then_branch,
                mut else_branch,
            } => {
                let arg = then_branch[ith];
                // both branches need to match
                if arg == else_branch[ith] {
                    if let Operand::Arg(input_index) = arg {
                        // check if we can pass through both then and else branches
                        let mut cache = Default::default();
                        let mut can_remove = true;
                        for (index, op) in then_branch.clone().iter().enumerate() {
                            if index != ith {
                                can_remove = can_remove
                                    && self
                                        .operand_remove_ith_arg(op, input_index, &mut cache, false)
                                        .0;
                            }
                        }
                        for (index, op) in else_branch.clone().iter().enumerate() {
                            if index != ith {
                                can_remove = can_remove
                                    && self
                                        .operand_remove_ith_arg(op, input_index, &mut cache, false)
                                        .0;
                            }
                        }

                        if can_remove {
                            // remove input index from inputs
                            let passed_through = inputs.remove(input_index);
                            // remove ith from then and else branches
                            then_branch.remove(ith);
                            else_branch.remove(ith);

                            // now remove the ith argument from the body
                            let mut cache = Default::default();
                            for op in then_branch.iter_mut() {
                                *op = self
                                    .operand_remove_ith_arg(op, input_index, &mut cache, true)
                                    .1;
                            }
                            for op in else_branch.iter_mut() {
                                *op = self
                                    .operand_remove_ith_arg(op, input_index, &mut cache, true)
                                    .1;
                            }
                            let new_region = RvsdgBody::If {
                                pred,
                                inputs,
                                then_branch,
                                else_branch,
                            };
                            self.nodes[node_id] = new_region;
                            return Some(passed_through);
                        }
                    }
                }
                None
            }
            RvsdgBody::Gamma {
                pred,
                mut inputs,
                mut outputs,
            } => {
                let arg = outputs.first().unwrap()[ith];
                // rest of the outputs need to match
                if outputs.iter().all(|output| output[ith] == arg) {
                    if let Operand::Arg(input_index) = arg {
                        // check if we can pass through all outputs
                        let mut cache = Default::default();
                        let mut can_remove = true;
                        for output_region in outputs.iter() {
                            for (index, output) in output_region.iter().enumerate() {
                                if index != ith {
                                    can_remove = can_remove
                                        && self
                                            .operand_remove_ith_arg(
                                                output,
                                                input_index,
                                                &mut cache,
                                                false,
                                            )
                                            .0;
                                }
                            }
                        }

                        if can_remove {
                            // remove the ith argument from the body of each
                            let mut cache = Default::default();
                            for output_region in outputs.iter_mut() {
                                for output in output_region.iter_mut() {
                                    *output = self
                                        .operand_remove_ith_arg(
                                            output,
                                            input_index,
                                            &mut cache,
                                            true,
                                        )
                                        .1;
                                }
                            }
                            // remove input index from inputs
                            let passed_through = inputs.remove(input_index);
                            // remove ith from outputs
                            for output_region in outputs.iter_mut() {
                                output_region.remove(ith);
                            }
                            let new_region = RvsdgBody::Gamma {
                                pred,
                                inputs,
                                outputs,
                            };
                            self.nodes[node_id] = new_region;
                            return Some(passed_through);
                        }
                    }
                }
                None
            }
            RvsdgBody::Theta {
                mut pred,
                mut inputs,
                mut outputs,
            } => {
                if let Operand::Arg(input_index) = outputs[ith] {
                    // for loops, input_index needs to match ith
                    if input_index == ith {
                        // check that we can remove ith arg
                        let mut cache = Default::default();
                        let mut can_remove = true;
                        for (index, output) in outputs.iter().enumerate() {
                            if index != ith {
                                can_remove = can_remove
                                    && self
                                        .operand_remove_ith_arg(
                                            output,
                                            input_index,
                                            &mut cache,
                                            false,
                                        )
                                        .0;
                            }
                        }
                        can_remove = can_remove
                            && self
                                .operand_remove_ith_arg(&pred, input_index, &mut cache, false)
                                .0;

                        if can_remove {
                            // remove ith from outputs
                            outputs.remove(ith);

                            // remove the ith argument from the body of each
                            let mut cache = Default::default();
                            for output in outputs.iter_mut() {
                                *output = self
                                    .operand_remove_ith_arg(output, input_index, &mut cache, true)
                                    .1;
                            }
                            // remove ith argument from pred
                            pred = self
                                .operand_remove_ith_arg(&pred, input_index, &mut cache, true)
                                .1;

                            // remove input index from inputs
                            let passed_through = inputs.remove(input_index);

                            let new_region = RvsdgBody::Theta {
                                pred,
                                inputs,
                                outputs,
                            };
                            self.nodes[node_id] = new_region;
                            return Some(passed_through);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Returns a new function with some passthrough optimizations.
    /// Also returns a boolean indicating if any changes were made.
    fn optimize_passthrough(&self) -> (RvsdgFunction, bool) {
        let mut new_func: RvsdgFunction = self.clone();
        let mut did_something = false;

        // for each node and each output, try to pass through the value
        // if we can, fix up all references to this node
        let uses = new_func.uses_analysis();
        let empty_set = Default::default();
        for id in 0..new_func.nodes.len() {
            let id = id as Id;
            let mut output_index = 0;
            // be sure to calculate num_outputs() each iteration
            while output_index < new_func.nodes[id].num_outputs() {
                if let Some(passed_through_operand) = new_func.passthrough_operand(id, output_index)
                {
                    did_something = true;
                    // rewrite all uses of this node to use the passed through value or offset
                    for node_use in uses.get(&id).unwrap_or(&empty_set) {
                        let use_node = &mut new_func.nodes[*node_use];
                        use_node.map_operands(&mut |operand| {
                            if let Operand::Project(project_index, proj_id) = *operand {
                                if proj_id == id {
                                    match project_index.cmp(&output_index) {
                                        std::cmp::Ordering::Less => *operand,
                                        std::cmp::Ordering::Equal => passed_through_operand,
                                        std::cmp::Ordering::Greater => {
                                            Operand::Project(project_index - 1, id)
                                        }
                                    }
                                } else {
                                    *operand
                                }
                            } else {
                                *operand
                            }
                        });
                    }

                    // also rewrite the function results if needed
                    for (_ty, result) in new_func.results.iter_mut() {
                        if let Operand::Project(project_index, project_id) = *result {
                            if project_id == id && project_index > output_index {
                                *result = Operand::Project(project_index - 1, id);
                            } else if project_id == id && project_index == output_index {
                                *result = passed_through_operand;
                            }
                        }
                    }
                }
                output_index += 1;
            }
        }

        (new_func, did_something)
    }
}
