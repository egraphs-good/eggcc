use bril_rs::{Code, Function, Instruction, Program};
use hashbrown::{HashMap, HashSet};

use crate::util::FreshNameGen;

use super::{Expr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};

struct RvsdgToBril<'a> {
    function: &'a RvsdgFunction,
    fresh_name: FreshNameGen,
    instrs: Vec<Code>,
    /// variable storing the result of an RVSDGBody
    /// Stores multiple outputs for a Gamma or Theta,
    /// but a vector with one variable for a BasicOp
    evaluated: HashMap<Id, Vec<String>>,
}

impl RvsdgProgram {
    pub fn to_bril(&self) -> Program {
        // TODO right now we only support one function
        // which is named main
        assert!(self.functions.len() == 1);
        Program {
            functions: self.functions.iter().map(|f| f.to_bril()).collect(),
            imports: vec![],
        }
    }
}

impl RvsdgFunction {
    pub fn to_bril(&self) -> Function {
        let mut to_bril = RvsdgToBril {
            function: self,
            fresh_name: FreshNameGen::new(),
            instrs: vec![],
            evaluated: HashMap::new(),
        };

        to_bril.operand_to_bril(self.state);
        if let Some(operand) = self.result {
            to_bril.operand_to_bril(operand);
        }

        // TODO hard-coded name
        Function {
            name: "main".into(),
            args: vec![],
            instrs: to_bril.instrs,
            pos: None,
            return_type: None,
        }
    }
}

impl<'a> RvsdgToBril<'a> {
    fn operand_to_bril(&mut self, operand: Operand) -> String {
        match operand {
            Operand::Id(id) => {
                let res = self.body_to_bril(id);
                assert!(res.len() == 1);
                res[0].clone()
            }
            Operand::Arg(_) => panic!("args not supported yet"),
            Operand::Project(arg, id) => {
                let res = self.body_to_bril(id);
                res[arg].clone()
            }
        }
    }

    fn body_to_bril(&mut self, id: Id) -> Vec<String> {
        if let Some(res) = self.evaluated.get(&id) {
            return res.clone();
        }
        let body = self.function.nodes[id];
        let res = match body {
            RvsdgBody::BasicOp(expr) => vec![self.expr_to_bril(&expr)],
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => todo!(),
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            } => todo!(),
        };

        todo!()
    }

    fn expr_to_bril(&mut self, expr: &Expr<Operand>) -> String {
        match expr {
            Expr::Op(value_op, operands) => {
                let operands = operands
                    .iter()
                    .map(|op| self.operand_to_bril(*op))
                    .collect();
                let name = self.fresh_name.fresh();
                self.instrs.push(Code::Instruction(Instruction::Value {
                    dest: name.clone(),
                    op: value_op.clone(),
                    args: operands,
                    funcs: vec![],
                    labels: vec![],
                    pos: None,
                }));
                name
            }
        }
    }
}
