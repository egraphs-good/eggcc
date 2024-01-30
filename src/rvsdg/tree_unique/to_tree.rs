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
#[cfg(test)]
use bril_rs::Type;
#[cfg(test)]
use tree_optimizer::ast::program;

use crate::rvsdg::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};
use bril_rs::{Literal, ValueOps};
use hashbrown::HashMap;
use tree_optimizer::{
    ast::{
        add, arg, concat, function, get, getarg, lessthan, num, parallel, parallel_vec, print,
        program_vec, tfalse, tlet, tloop, ttrue,
    },
    expr::{Expr, TreeType},
};

impl RvsdgProgram {
    pub fn to_tree_encoding(&self) -> Expr {
        program_vec(
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

    /// Make a new translator for a region with
    /// num_args and the given nodes.
    fn new(num_args: usize, nodes: &'a Vec<RvsdgBody>) -> RegionTranslator {
        RegionTranslator {
            num_args,
            bindings: Vec::new(),
            index_of: HashMap::new(),
            nodes,
        }
    }

    /// Wrap the given expression in all the
    /// bindings that have been generated.
    fn build_translation(&self, inner: Expr) -> Expr {
        let mut expr = inner;

        for binding in self.bindings.iter().rev() {
            expr = cbind(binding.clone(), expr);
        }
        expr
    }

    /// Returns a pure expression (e.g. `getarg(0)`) that
    /// returns the value for this operand.
    /// The value of the operand is let-bound
    /// and the expression refers to it.
    fn translate_operand(&mut self, operand: Operand) -> Expr {
        match operand {
            Operand::Arg(index) => getarg(index),
            Operand::Id(id) => getarg(self.translate_node(id)),
            Operand::Project(p_index, id) => {
                // Translated region becomes a tuple in the environment.
                // This is the index of that tuple.
                let index = self.translate_node(id);
                get(getarg(index), p_index)
            }
        }
    }

    /// Translate a node or return the index of the already-translated node.
    /// For regions, translates the region and returns the index of the
    /// tuple containing the results.
    /// It's important not to evaluate a node twice, instead using the cached index
    /// in `self.index_of`
    fn translate_node(&mut self, id: Id) -> usize {
        if let Some(index) = self.index_of.get(&id) {
            *index
        } else {
            let node = &self.nodes[id];
            match node {
                RvsdgBody::BasicOp(expr) => self.translate_basic_expr(expr.clone(), id),
                RvsdgBody::Gamma { .. } => todo!("Doesn't handle gamma yet"),
                RvsdgBody::Theta {
                    pred,
                    inputs,
                    outputs,
                } => {
                    let mut translated_inputs = vec![];
                    // for loop instead of iterator because of lifetimes
                    for input in inputs {
                        translated_inputs.push(self.translate_operand(*input));
                    }

                    let mut sub_translator = RegionTranslator::new(inputs.len(), self.nodes);
                    let pred_translated = sub_translator.translate_operand(*pred);
                    let outputs_translated =
                        outputs.iter().map(|o| sub_translator.translate_operand(*o));
                    let pred_and_outputs =
                        parallel!(pred_translated, parallel_vec(outputs_translated.collect()));
                    let loop_translated = sub_translator.build_translation(pred_and_outputs);

                    let loop_expr = tloop(parallel_vec(translated_inputs), loop_translated);
                    self.add_binding(loop_expr, id)
                }
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
                    .map(|c| self.translate_operand(*c))
                    .collect::<Vec<_>>();
                let expr = match (op, children.as_slice()) {
                    (ValueOps::Add, [a, b]) => add(a.clone(), b.clone()),
                    (ValueOps::Lt, [a, b]) => lessthan(a.clone(), b.clone()),
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
                // argument 2 should have value unit, since it is
                // the print buffer value.
                let _arg2 = self.translate_operand(args[1]);
                // print outputs a new unit value
                let expr = print(arg1);
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
        let mut translator = RegionTranslator::new(self.args.len(), &self.nodes);
        let translated_results = self
            .results
            .iter()
            .map(|r| translator.translate_operand(r.1))
            .collect::<Vec<_>>();

        function(
            self.name.as_str(),
            TreeType::Tuple(self.args.iter().map(|ty| ty.to_tree_type()).collect()),
            TreeType::Tuple(self.results.iter().map(|r| r.0.to_tree_type()).collect()),
            translator.build_translation(parallel_vec(translated_results)),
        )
    }
}

#[test]
fn translate_simple_loop() {
    const PROGRAM: &str = r#"
@myfunc(): int {
    .entry:
        one: int = const 1;
        two: int = const 2;
    .loop:
        cond: bool = lt one two;
        br cond .loop .exit;
    .exit:
        ret one;
}
"#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    rvsdg
        .to_tree_encoding()
        .assert_eq_ignoring_ids(&program!(function(
            "myfunc",
            TreeType::Tuple(vec![TreeType::Unit]),
            TreeType::Tuple(vec![TreeType::Bril(Type::Int), TreeType::Unit]),
            cbind(
                num(1), // [(), 1]
                cbind(
                    num(2), // [(), 1, 2]
                    cbind(
                        tloop(
                            parallel!(getarg(0), getarg(1), getarg(2)), // [(), 1, 2]
                            cbind(
                                lessthan(getarg(1), getarg(2)), // [(), 1, 2, 1<2]
                                parallel!(getarg(3), parallel!(getarg(0), getarg(1), getarg(2)))
                            )
                        ), // [(), 1, 2, [(), 1, 2]]
                        parallel!(get(getarg(3), 1), get(getarg(3), 0)) // return [1, ()]
                    ),
                )
            )
        )));
}

#[test]
fn translate_loop() {
    const PROGRAM: &str = r#"
@main {
    .entry:
        i: int = const 0;
    .loop:
        max: int = const 10;
        one: int = const 1;
        i: int = add i one;
        cond: bool = lt i max;
        br cond .loop .exit;
    .exit:
        print i;
}
"#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    rvsdg
        .to_tree_encoding()
        .assert_eq_ignoring_ids(&program!(function(
            "main",
            TreeType::Tuple(vec![TreeType::Unit]),
            TreeType::Tuple(vec![TreeType::Unit]),
            cbind(
                num(0), // [(), 0]
                cbind(
                    tloop(
                        parallel!(getarg(0), getarg(1)),
                        cbind(
                            num(1), // [(), i, 1]
                            cbind(
                                add(getarg(1), getarg(2)), // [(), i, 1, i+1]
                                cbind(
                                    num(10), // [(), i, 1, i+1, 10]
                                    cbind(
                                        lessthan(getarg(3), getarg(4)), // [(), i, 1, i+1, 10, i<10]
                                        parallel!(getarg(5), parallel!(getarg(0), getarg(3)))
                                    )
                                )
                            )
                        )
                    ),
                    cbind(
                        print(get(getarg(2), 1)), // [(), 0, [() i]]
                        parallel!(getarg(3))
                    )
                ),
            )
        )));
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
        .assert_eq_ignoring_ids(&program!(function(
            "add",
            TreeType::Tuple(vec![TreeType::Unit]),
            TreeType::Tuple(vec![TreeType::Bril(Type::Int), TreeType::Unit]),
            cbind(
                num(1),
                cbind(
                    add(get(arg(), 1), get(arg(), 1)),
                    parallel!(get(arg(), 2), get(arg(), 0)), // returns res and print state (unit)
                ),
            )
        )));
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
        .assert_eq_ignoring_ids(&program!(function(
            "add",
            TreeType::Tuple(vec![TreeType::Unit]),
            TreeType::Tuple(vec![TreeType::Unit]),
            cbind(
                num(2),
                cbind(
                    num(1),
                    cbind(
                        add(get(arg(), 2), get(arg(), 1)),
                        cbind(
                            print(get(arg(), 3)),
                            cbind(print(get(arg(), 1)), parallel!(get(arg(), 5))),
                        ),
                    ),
                ),
            )
        )));
}
