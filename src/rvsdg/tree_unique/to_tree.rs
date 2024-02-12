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
use tree_assume::ast::{emptyt, intt, parallel, program, push_par};
#[cfg(test)]
use tree_assume::interpreter::Value;
#[cfg(test)]
use tree_assume::schema::Constant;

use crate::rvsdg::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};
use bril_rs::{EffectOps, Literal, ValueOps};
use hashbrown::HashMap;
use tree_assume::{
    ast::{
        add, arg, concat_par, dowhile, function, getarg, int, less_than, parallel_vec, program_vec,
        single, tfalse, tlet, tprint, ttrue,
    },
    schema::{RcExpr, TreeProgram, Type as TreeType},
};

impl RvsdgProgram {
    pub fn to_tree_encoding(&self) -> TreeProgram {
        let first_function = self.functions.first().unwrap();
        let rest_functions = self.functions.iter().skip(1);
        program_vec(
            first_function.to_tree_encoding(),
            rest_functions
                .map(|f| f.to_tree_encoding())
                .collect::<Vec<_>>(),
        )
    }
}

/// A `ValueIndex` stores the location of the value or values of
/// an RVSDG node.
/// During translation, values for each node are stored in the bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StoredNode {
    /// The value is stored at get(arg(), usize)
    Arg(usize),
    /// The node is a region with values stored at
    /// the given indices.
    /// The indices in the vec will never be another Region, but might be a state edge.
    Region(Vec<StoredNode>),
    /// The value is a state edge, thus not stored.
    StateEdge,
}

impl StoredNode {
    /// Translates a value index to an expression
    /// that returns the value of the index.
    /// Returns None for state edges.
    fn to_expr(&self) -> Option<RcExpr> {
        match self {
            StoredNode::Arg(index) => Some(getarg(*index)),
            StoredNode::Region(indices) => {
                let exprs = indices
                    .iter()
                    .filter_map(|i| i.to_expr())
                    .collect::<Vec<_>>();
                Some(parallel_vec(exprs))
            }
            StoredNode::StateEdge => None,
        }
    }
}

struct RegionTranslator<'a> {
    /// The values of the arguments to this region.
    /// Must be either StoredNode::Arg or StoredNode::StateEdge.
    argument_values: Vec<StoredNode>,
    /// The number of arguments in the current environment.
    current_num_args: usize,
    /// a stack of let bindings to generate
    /// each `RcExpr` is an expression producing a tuple.
    /// These tuples are concatenated to the current argument during `build_translation`.
    bindings: Vec<RcExpr>,
    /// After evaluating a node, do not evaluate it again.
    /// Instead find its index here.
    index_of: HashMap<Id, StoredNode>,
    nodes: &'a Vec<RvsdgBody>,
}

/// helper that binds a new expression, adding it
/// to the environment by concatenating all previous values
/// with the new one
fn bind_tuple(expr: RcExpr, body: RcExpr) -> RcExpr {
    tlet(concat_par(arg(), expr), body)
}

#[cfg(test)]
fn bind_value(expr: RcExpr, body: RcExpr) -> RcExpr {
    tlet(push_par(expr, arg()), body)
}

impl<'a> RegionTranslator<'a> {
    /// Adds a binding and returns its index
    /// into the argument list.
    /// `expr` must not have a tuple type.
    fn add_binding(&mut self, expr: RcExpr, id: Id, is_state_edge: bool) -> StoredNode {
        self.bindings.push(single(expr));
        let res = if is_state_edge {
            StoredNode::StateEdge
        } else {
            self.current_num_args += 1;
            StoredNode::Arg(self.current_num_args - 1)
        };
        assert_eq!(
            self.index_of.insert(id, res.clone()),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        res
    }

    fn add_region_binding(&mut self, expr: RcExpr, id: Id, values: Vec<StoredNode>) -> StoredNode {
        self.bindings.push(expr);
        let res = StoredNode::Region(values);
        assert_eq!(
            self.index_of.insert(id, res.clone()),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        res
    }

    /// Make a new translator for a region with
    /// num_args and the given nodes.
    fn new(nodes: &'a Vec<RvsdgBody>, argument_values: Vec<StoredNode>) -> RegionTranslator {
        // count the number of non-state-edge args
        let num_args = argument_values
            .iter()
            .map(|ele| matches!(ele, StoredNode::Arg(_)) as usize)
            .sum();
        RegionTranslator {
            current_num_args: num_args,
            bindings: Vec::new(),
            index_of: HashMap::new(),
            argument_values,
            nodes,
        }
    }

    /// Wrap the given expression in all the
    /// bindings that have been generated.
    fn build_translation(&self, inner: RcExpr) -> RcExpr {
        let mut expr = inner;

        for binding in self.bindings.iter().rev() {
            expr = bind_tuple(binding.clone(), expr);
        }
        expr
    }

    /// Returns a ValueIndex for the given operand.
    /// The ValueIndex should not be a `Region`, since operands
    /// return one value.
    fn translate_operand(&mut self, operand: Operand) -> StoredNode {
        match operand {
            Operand::Arg(index) => {
                eprintln!("Arg {} in {:?}", index, self.argument_values);
                self.argument_values[index].clone()
            }
            Operand::Id(id) => {
                let res = self.translate_node(id);
                if matches!(res, StoredNode::Region(_)) {
                    panic!("Expected a single value, found a region");
                }
                res
            }
            Operand::Project(p_index, id) => {
                let StoredNode::Region(values) = self.translate_node(id) else {
                    panic!("Expected a region, found a single value");
                };
                let res = values[p_index].clone();
                if matches!(res, StoredNode::Region(_)) {
                    panic!("Found region inside of region value");
                }
                res
            }
        }
    }

    /// Translate a node or return the index of the already-translated node.
    /// For regions, translates the region and returns the index of the
    /// tuple containing the results.
    /// It's important not to evaluate a node twice, instead using the cached index
    /// in `self.index_of`
    fn translate_node(&mut self, id: Id) -> StoredNode {
        if let Some(index) = self.index_of.get(&id) {
            index.clone()
        } else {
            let node = &self.nodes[id];
            match node {
                RvsdgBody::BasicOp(expr) => self.translate_basic_expr(expr.clone(), id),
                RvsdgBody::If { .. } => todo!("Doesn't handle if yet"),
                RvsdgBody::Gamma { .. } => todo!("Doesn't handle gamma yet"),
                RvsdgBody::Theta {
                    pred,
                    inputs,
                    outputs,
                } => {
                    eprintln!("here");
                    let mut input_values = vec![];
                    for input in inputs {
                        input_values.push(self.translate_operand(*input));
                    }

                    let mut argument_values = vec![];
                    let mut new_arg_index = 0;
                    for input_val in &input_values {
                        match input_val {
                            StoredNode::Arg(_index) => {
                                argument_values.push(StoredNode::Arg(new_arg_index));
                                new_arg_index += 1;
                            }
                            StoredNode::StateEdge => {
                                argument_values.push(StoredNode::StateEdge);
                            }
                            StoredNode::Region(_) => {
                                panic!("Found region in theta input");
                            }
                        }
                    }

                    let mut sub_translator = RegionTranslator::new(self.nodes, argument_values);
                    let mut pred_outputs = vec![sub_translator
                        .translate_operand(*pred)
                        .to_expr()
                        .unwrap_or_else(|| panic!("Pred was a state edge"))];
                    let outputs_translated = outputs
                        .iter()
                        .map(|o| sub_translator.translate_operand(*o))
                        .collect::<Vec<StoredNode>>();
                    for output in outputs_translated.iter() {
                        if let Some(output_expr) = output.to_expr() {
                            pred_outputs.push(output_expr);
                        }
                    }
                    let loop_translated =
                        sub_translator.build_translation(parallel_vec(pred_outputs));

                    let loop_expr = dowhile(
                        parallel_vec(input_values.into_iter().filter_map(|val| val.to_expr())),
                        loop_translated,
                    );

                    // build the stored node
                    let mut output_values = vec![];
                    for output in outputs_translated {
                        match output {
                            StoredNode::StateEdge => (),
                            StoredNode::Arg(_) => {
                                output_values.push(StoredNode::Arg(self.current_num_args));
                                self.current_num_args += 1;
                            }
                            StoredNode::Region(_) => panic!("Found nested region in theta output"),
                        }
                    }
                    self.add_region_binding(loop_expr, id, output_values)
                }
            }
        }
    }

    /// Translate this expression at the given id,
    /// return the newly created index.
    fn translate_basic_expr(&mut self, expr: BasicExpr<Operand>, id: Id) -> StoredNode {
        match expr {
            BasicExpr::Op(op, children, _ty) => {
                let children = children
                    .iter()
                    .map(|c| {
                        self.translate_operand(*c)
                            .to_expr()
                            .unwrap_or_else(|| panic!("State edge in op"))
                    })
                    .collect::<Vec<_>>();
                let expr = match (op, children.as_slice()) {
                    (ValueOps::Add, [a, b]) => add(a.clone(), b.clone()),
                    (ValueOps::Lt, [a, b]) => less_than(a.clone(), b.clone()),
                    _ => todo!("handle other ops"),
                };
                self.add_binding(expr, id, false)
            }
            BasicExpr::Call(..) => {
                todo!("handle calls");
            }
            BasicExpr::Const(_op, literal, _ty) => match literal {
                Literal::Int(n) => {
                    let expr = int(n);
                    self.add_binding(expr, id, false)
                }
                Literal::Bool(b) => {
                    let expr = if b { ttrue() } else { tfalse() };
                    self.add_binding(expr, id, false)
                }
                _ => todo!("handle other literals"),
            },
            BasicExpr::Effect(EffectOps::Print, args) => {
                assert!(args.len() == 2, "print should have 2 arguments");
                let arg1 = self
                    .translate_operand(args[0])
                    .to_expr()
                    .unwrap_or_else(|| panic!("Print buffer expr should be a single value"));
                let arg2 = self.translate_operand(args[1]);

                // Print returns a state edge, which should be translated as
                // a unit value
                assert_eq!(
                    arg2,
                    StoredNode::StateEdge,
                    "Print buffer second argument should be state edge. Found {:?}",
                    arg2
                );
                // print outputs a new unit value
                let expr = tprint(arg1);
                self.add_binding(expr, id, true)
            }
            BasicExpr::Effect(..) => {
                todo!("handle memory operations")
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
    pub fn to_tree_encoding(&self) -> RcExpr {
        let mut arg_index = 0;
        let argument_values = self
            .args
            .iter()
            .map(|ty| match ty {
                RvsdgType::PrintState => StoredNode::StateEdge,
                RvsdgType::Bril(_) => {
                    arg_index += 1;
                    StoredNode::Arg(arg_index - 1)
                }
            })
            .collect();
        let mut translator = RegionTranslator::new(&self.nodes, argument_values);
        let translated_results = self
            .results
            .iter()
            .filter_map(|r| translator.translate_operand(r.1).to_expr())
            .collect::<Vec<_>>();

        function(
            self.name.as_str(),
            TreeType::TupleT(
                self.args
                    .iter()
                    .filter_map(|ty| ty.to_tree_type())
                    .collect(),
            ),
            TreeType::TupleT(
                self.results
                    .iter()
                    .filter_map(|r| r.0.to_tree_type())
                    .collect(),
            ),
            translator.build_translation(parallel_vec(translated_results)),
        )
    }
}

/// Assert that two programs are equal, and
/// test them on a particular input and output value
#[cfg(test)]
fn assert_progs_eq(
    prog1: &TreeProgram,
    expected: &TreeProgram,
    input_val: Value,
    output_val: Value,
) {
    // first, check expected works properly

    use tree_assume::interpreter::interpret;
    let expected_res = interpret(expected, input_val);
    assert_eq!(
        expected_res, output_val,
        "Reference program produced incorrect result. Expected {:?}, found {:?}",
        output_val, expected_res
    );

    assert_eq!(
        prog1, expected,
        "Found:\n{}\nExpected:\n{}",
        prog1, expected
    );
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

    assert_progs_eq(
        &rvsdg.to_tree_encoding(),
        &program!(function(
            "add",
            TreeType::TupleT(vec![]),
            TreeType::TupleT(vec![intt()]),
            bind_value(
                int(1),
                bind_value(
                    add(getarg(0), getarg(0)),
                    parallel!(getarg(1)), // returns res
                ),
            )
        ),),
        Value::Tuple(vec![]),
        Value::Tuple(vec![Value::Const(Constant::Int(2))]),
    );
}

#[test]
fn translate_simple_loop() {
    const PROGRAM: &str = r#"
@myfunc(): int {
    .entry:
        one: int = const 1;
        two: int = const 2;
    .loop:
        cond: bool = lt two one;
        br cond .loop .exit;
    .exit:
        ret one;
}
"#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    assert_progs_eq(
        &rvsdg.to_tree_encoding(),
        &program!(function(
            "myfunc",
            emptyt(),
            TreeType::TupleT(vec![intt()]),
            bind_value(
                int(1), // [1]
                bind_value(
                    int(2), // [1, 2]
                    bind_tuple(
                        dowhile(
                            parallel!(getarg(0), getarg(1)), // [1, 2]
                            bind_value(
                                less_than(getarg(1), getarg(0)), // [1, 2, 2<1]
                                parallel!(getarg(2), getarg(0), getarg(1))
                            )
                        ), // [1, 2, 1, 2]
                        parallel!(getarg(2)) // return [1]
                    ),
                )
            )
        ),),
        Value::Tuple(vec![]),
        Value::Tuple(vec![Value::Const(Constant::Int(1))]),
    );
}
/*
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

    assert_eq!(
        rvsdg.to_tree_encoding(),
        program!(function(
            "main",
            TreeType::TupleT(vec![emptyt()]),
            TreeType::TupleT(vec![emptyt()]),
            bind_value(
                1,
                int(0), // [(), 0]
                bind_value(
                    2,
                    dowhile(
                        parallel!(getarg(0), getarg(1)),
                        bind_value(
                            2,
                            int(1), // [(), i, 1]
                            bind_value(
                                3,
                                add(getarg(1), getarg(2)), // [(), i, 1, i+1]
                                bind_value(
                                    4,
                                    int(10), // [(), i, 1, i+1, 10]
                                    bind_value(
                                        5,
                                        less_than(getarg(3), getarg(4)), // [(), i, 1, i+1, 10, i<10]
                                        parallel!(getarg(5), parallel!(getarg(0), getarg(3)))
                                    )
                                )
                            )
                        )
                    ),
                    bind_value(
                        3,
                        tprint(get(getarg(2), 1)), // [(), 0, [() i], ()]
                        parallel!(getarg(3))
                    )
                ),
            )
        ),)
    );
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
            TreeType::TupleT(vec![emptyt()]),
            TreeType::TupleT(vec![emptyt()]),
            bind_value(
                1,
                int(2),
                bind_value(
                    2,
                    int(1),
                    bind_value(
                        3,
                        add(get(arg(), 2), get(arg(), 1)),
                        bind_value(
                            4,
                            tprint(get(arg(), 3)),
                            bind_value(5, tprint(get(arg(), 1)), parallel!(get(arg(), 5))),
                        ),
                    ),
                ),
            )
        )));
}
*/
