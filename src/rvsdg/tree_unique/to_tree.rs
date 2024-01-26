//! Convert RVSDG programs to the tree
//! encoding of programs.
//! RVSDGs are close to this encoding,
//! but use a DAG-based semantics.
//! This means that nodes that are shared
//! are only computed once.
//! These shared nodes need to be let-bound so that they are only
//! computed once in the tree encoded
//! program.
#[cfg(test)]
use crate::{cfg::program_to_cfg, rvsdg::cfg_to_rvsdg, util::parse_from_string};

use crate::rvsdg::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};
use bril_rs::{Literal, ValueOps};
use hashbrown::HashMap;
use tree_unique_args::{
    ast::{add, arg, concat, function, get, num, print, program, sequence, tfalse, tlet, ttrue},
    Expr,
};

impl RvsdgProgram {
    pub fn to_tree_encoding(&self) -> Expr {
        program(
            self.functions
                .iter()
                .map(|f| f.to_tree_encoding())
                .collect(),
        )
    }
}

struct RegionTranslator<'a> {
    /// The number of arguments to this region
    num_args: usize,
    /// a stack of let bindings to generate
    bindings: Vec<Expr>,
    /// A map from the rvsdg node id
    /// to the index in the argument
    /// where the value is stored.
    /// After evaluating a node, do not evaluate it again.
    /// Instead find its index here.
    index_of: HashMap<Id, usize>,
    nodes: &'a Vec<RvsdgBody>,
}

/// helper that binds a new expression, adding it
/// to the environment using concat
fn cbind(expr: Expr, body: Expr) -> Expr {
    tlet(concat(arg(), expr), body)
}

impl<'a> RegionTranslator<'a> {
    /// Adds a binding and returns its index
    /// into the argument list.
    fn add_binding(&mut self, expr: Expr, id: Id) -> usize {
        self.bindings.push(expr);
        let res = self.bindings.len() - 1 + self.num_args;
        assert_eq!(
            self.index_of.insert(id, res),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        res
    }

    /// Build a translator and translate
    /// the operands to the tree encoding.
    /// Produces a tree-encoded term that evaluates
    /// to a tuple containing results.
    fn translate(num_args: usize, nodes: &'a Vec<RvsdgBody>, results: Vec<Operand>) -> Expr {
        let mut translator = RegionTranslator {
            num_args,
            bindings: Vec::new(),
            index_of: HashMap::new(),
            nodes,
        };

        let mut result_indices = Vec::new();
        for result in results {
            result_indices.push(translator.translate_operand(result));
        }

        let mut expr = sequence(result_indices.iter().map(|i| get(arg(), *i)).collect());

        for binding in translator.bindings.into_iter().rev() {
            expr = cbind(binding, expr);
        }
        expr
    }

    fn translate_operand(&mut self, operand: Operand) -> usize {
        match operand {
            Operand::Arg(index) => index,
            Operand::Id(id) => self.translate_node(id),
            Operand::Project(_id, _indexx) => {
                todo!("Doesn't handle subregions yet");
            }
        }
    }

    /// Translate a node or return the index of the already evaluated node.
    /// It's important not to evaluate a node twice, instead using the cached index
    /// in `self.index_of`
    fn translate_node(&mut self, id: Id) -> usize {
        if let Some(index) = self.index_of.get(&id) {
            *index
        } else {
            let node = &self.nodes[id];
            match node {
                RvsdgBody::BasicOp(expr) => self.translate_basic_expr(expr.clone(), id),
                _ => todo!("Doesn't handle subregions yet"),
            }
        }
    }

    /// Translate this expression at the given id,
    /// return the newly created index.
    fn translate_basic_expr(&mut self, expr: BasicExpr<Operand>, id: Id) -> usize {
        match expr {
            BasicExpr::Op(op, children, _ty) => {
                let children = children
                    .iter()
                    .map(|c| get(arg(), self.translate_operand(*c)))
                    .collect::<Vec<_>>();
                let expr = match (op, children.as_slice()) {
                    (ValueOps::Add, [a, b]) => add(a.clone(), b.clone()),
                    _ => todo!("handle other ops"),
                };
                self.add_binding(expr, id)
            }
            BasicExpr::Call(..) => {
                todo!("handle calls");
            }
            BasicExpr::Const(_op, literal, _ty) => match literal {
                Literal::Int(n) => {
                    let expr = num(n);
                    self.add_binding(expr, id)
                }
                Literal::Bool(b) => {
                    let expr = if b { ttrue() } else { tfalse() };
                    self.add_binding(expr, id)
                }
                _ => todo!("handle other literals"),
            },
            BasicExpr::Print(args) => {
                assert!(args.len() == 2, "print should have 2 arguments");
                let arg1 = self.translate_operand(args[0]);
                let _arg2 = self.translate_operand(args[1]);
                let expr = print(get(arg(), arg1));
                self.add_binding(expr, id)
            }
        }
    }
}

impl RvsdgFunction {
    /// Translates an RVSDG function to the
    /// tree encoding.
    /// It generates one let binding per
    /// node in the RVSDG, adding the value
    /// for that node to the end of the argument
    /// using the `concat` constructor.
    /// In the inner-most scope, the value of
    /// all nodes is available.
    pub fn to_tree_encoding(&self) -> Expr {
        function(RegionTranslator::translate(
            self.args.len(),
            &self.nodes,
            self.results.iter().map(|r| r.1).collect(),
        ))
    }
}

#[test]
fn simple_translation() {
    const PROGRAM: &str = r#"
  @add(): int {
    v0: int = const 1;
    res: int = add v0 v0;
    ret res;
  }
  "#;

    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    rvsdg
        .to_tree_encoding()
        .assert_eq_ignoring_ids(&program(vec![function(cbind(
            num(1),
            cbind(
                add(get(arg(), 1), get(arg(), 1)),
                sequence(vec![get(arg(), 2), get(arg(), 0)]), // returns res and print state (unit)
            ),
        ))]));
}

#[test]
fn two_print_translation() {
    const PROGRAM: &str = r#"
    @add() {
        v0: int = const 1;
        v1: int = const 2;
        v2: int = add v0 v1;
        print v2;
        print v1;
    }
    "#;

    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    rvsdg
        .to_tree_encoding()
        .assert_eq_ignoring_ids(&program(vec![function(cbind(
            num(2),
            cbind(
                num(1),
                cbind(
                    add(get(arg(), 2), get(arg(), 1)),
                    cbind(
                        print(get(arg(), 3)),
                        cbind(print(get(arg(), 1)), sequence(vec![get(arg(), 5)])),
                    ),
                ),
            ),
        ))]));
}
