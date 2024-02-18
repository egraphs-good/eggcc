//! encoding of programs.
//! Convert RVSDG programs to the tree
//! RVSDGs are close to this encoding,
//! but use a DAG-based semantics.
//! This means that nodes that are shared
//! are only computed once.
//! These shared nodes need to be let-bound so that they are only
//! computed once in the tree encoded
//! program.

#[cfg(test)]
use crate::{cfg::program_to_cfg, rvsdg::cfg_to_rvsdg, util::parse_from_string};
use tree_in_context::ast::{and, div, empty, eq, mul, sub, switch_vec, tif};
#[cfg(test)]
use tree_in_context::ast::{intt, parallel, program, push_par};
#[cfg(test)]
use tree_in_context::interpreter::Value;
#[cfg(test)]
use tree_in_context::schema::Constant;

use crate::rvsdg::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};
use bril_rs::{EffectOps, Literal, ValueOps};
use hashbrown::HashMap;
use tree_in_context::{
    ast::{
        add, arg, call, concat_par, dowhile, emptyt, function, getarg, int, less_than,
        parallel_vec, program_vec, single, tfalse, tlet, tprint, ttrue,
    },
    schema::{RcExpr, TreeProgram, Type as TreeType},
};

impl RvsdgProgram {
    /// Converts an RVSDG program to the tree encoding.
    /// When `optimize_lets` is true, the conversion will also
    /// try to prevent adding unnecessary let bindings.
    pub fn to_tree_encoding(&self, optimize_lets: bool) -> TreeProgram {
        let last_function = self.functions.last().unwrap();
        let rest_functions = self.functions.iter().take(self.functions.len() - 1);
        program_vec(
            last_function.to_tree_encoding(optimize_lets),
            rest_functions
                .map(|f| f.to_tree_encoding(optimize_lets))
                .collect::<Vec<_>>(),
        )
    }
}

/// Stores the location of a single value of an RVSDG node.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StoredValue {
    /// The value is stored at get(arg(), usize)
    Arg(usize),
    /// The value is a state edge, thus not stored.
    StateEdge,
    /// The value is an expression without any Write or Print effects.
    /// The expression may read, however.
    /// This variant should only be constructed when the `optimize_lets` flag is true.
    Expr { expr: RcExpr, is_pure: bool },
}

/// Stores the location of the values of
/// an RVSDG node.
/// During translation, values for each node are stored in the bindings.
type StoredNode = Vec<StoredValue>;

fn storedarg(index: usize) -> StoredNode {
    vec![StoredValue::Arg(index)]
}

fn storedstate() -> StoredNode {
    vec![StoredValue::StateEdge]
}

impl StoredValue {
    fn to_expr(&self) -> Option<RcExpr> {
        match self {
            StoredValue::Arg(index) => Some(getarg(*index)),
            StoredValue::StateEdge => None,
            StoredValue::Expr { expr, .. } => Some(expr.clone()),
        }
    }
}

struct RegionTranslator<'a> {
    /// The values of the arguments to this region.
    argument_values: Vec<StoredValue>,
    /// The number of arguments in the current environment.
    current_num_args: usize,
    /// a stack of let bindings to generate
    /// each `RcExpr` is an expression producing a tuple.
    /// These tuples are concatenated to the current argument during `build_translation`.
    bindings: Vec<RcExpr>,
    /// `stored_node` is a cache of already translated rvsdg nodes.
    /// A pure node can be stored as an expression.
    /// Reads may need to be let-bound in `bindings` so they can be
    /// computed before writes.
    stored_node: HashMap<Id, StoredNode>,
    /// Delay binding of read nodes until necessary.
    /// `to_bind` stores a set of nodes such that
    /// 1) `stored_node[node]` is a `StoredNode::Expr(e)`
    /// 2) `e` contains a read
    /// When `optimize_lets` is enabled, stored nodes can be expressions.
    /// These expressions may contain reads.
    /// After a write occurs, and the next time any of these nodes are referenced,
    /// we need to let bind them.
    /// Here's an example execution trace of four RVSDG nodes being processed:
    /// ```text
    /// node1: read(2)
    /// ; stored_node[node1] = read(2), to_bind = [node1]
    /// node2: read(3)
    /// ; stored_node[node2] = read(3), to_bind = [node1, node2]
    /// node3: write(2, 14)
    /// ; stored_node[node3] = write(2, 14), has_write_occurred = true, effects_to_bind = [node3]
    /// node4: add(node1, node1)
    /// ; node1, node2, and node3 get added to the bindings
    /// ; stored_node[node1] = StoredNode::Arg(0), stored_node[node2] = StoredNode::Arg(1), stored_node[node3] = StoredNode::StateEdge
    /// ; stored_node[node4] = StoredNode::Expr(add(getarg(0), getarg(0)))
    /// ```
    to_bind: Vec<Id>,
    /// Tracks a set of print or write nodes that need to be bound.
    effects_to_bind: Vec<Id>,
    /// Tracks whether a write has occurred since the last time we added
    /// `to_bind` nodes to the bindings.
    /// After a write, we need to add a binding the next time a read is encountered.
    has_write_occurred: bool,
    nodes: &'a [RvsdgBody],
    /// Whether to optimize let bindings
    optimize_lets: bool,
}

/// helper that binds a new expression, adding it
/// to the environment by concatenating all previous values
/// with the new one
fn bind_tuple(new_tuple_expr: RcExpr, body: RcExpr) -> RcExpr {
    tlet(concat_par(arg(), new_tuple_expr), body)
}

/// Bind a single value instead of a tuple, convenient for testing
#[cfg(test)]
fn bind_value(expr: RcExpr, body: RcExpr) -> RcExpr {
    tlet(push_par(expr, arg()), body)
}

impl<'a> RegionTranslator<'a> {
    /// Adds a pure expression to the cache.
    fn add_pure_value(&mut self, expr: RcExpr, id: Id) -> StoredNode {
        if self.optimize_lets {
            let res = StoredValue::Expr {
                expr: expr.clone(),
                is_pure: true,
            };
            self.stored_node.insert(id, vec![res.clone()]);
            vec![res]
        } else {
            self.add_binding(expr, id)
        }
    }

    /// Adds a binding and returns its index
    /// into the argument list.
    /// `expr` must produce a single value, not a tuple.
    fn add_binding(&mut self, expr: RcExpr, id: Id) -> StoredNode {
        let res = storedarg(self.current_num_args);
        // produces a value, so wrap in `single` and push to bindings
        self.bindings.push(single(expr));
        self.current_num_args += 1;

        assert_eq!(
            self.stored_node.insert(id, res.clone()),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        res
    }

    /// Adds a binding for a state edge (e.g. a print or write)
    fn add_state_edge_binding(&mut self, expr: RcExpr, id: Id) -> StoredNode {
        self.bindings.push(expr);
        let res = storedstate();
        assert_eq!(
            self.stored_node.insert(id, res.clone()),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        res
    }

    /// Adds a tuple to the bindings.
    /// `values` is a vector refering to each value in the tuple.
    fn add_region_binding(&mut self, expr: RcExpr, id: Id, values: Vec<StoredValue>) -> StoredNode {
        self.bindings.push(expr);
        assert_eq!(
            self.stored_node.insert(id, values.clone()),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        values
    }

    /// Make a new translator for a region with
    /// num_args and the given nodes.
    fn new(
        nodes: &'a [RvsdgBody],
        argument_values: Vec<StoredValue>,
        num_args: usize,
        optimize_lets: bool,
    ) -> RegionTranslator {
        RegionTranslator {
            current_num_args: num_args,
            bindings: Vec::new(),
            stored_node: HashMap::new(),
            argument_values,
            nodes,
            optimize_lets,
            to_bind: Vec::new(),
            effects_to_bind: Vec::new(),
            has_write_occurred: false,
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

    /// Stores the operand in the bindings, returning
    /// the SingleStoredNode as a result.
    fn translate_operand(&mut self, operand: Operand) -> StoredValue {
        match operand {
            Operand::Arg(index) => self.argument_values[index].clone(),
            Operand::Id(id) => {
                let res = self.translate_node(id);
                if !res.len() == 1 {
                    panic!("Expected a single value, found a region");
                }
                res.into_iter().next().unwrap()
            }
            Operand::Project(p_index, id) => {
                let values = self.translate_node(id);
                values[p_index].clone()
            }
        }
    }

    /// Translate a node or return the index of the already-translated node.
    /// For regions, translates the region and returns the index of the
    /// tuple containing the results.
    /// It's important not to evaluate a node twice, instead using the cached index
    /// in `self.stored_node`
    fn translate_node(&mut self, id: Id) -> StoredNode {
        if let Some(index) = self.stored_node.get(&id) {
            index.clone()
        } else {
            let node = &self.nodes[id];
            match node {
                RvsdgBody::BasicOp(expr) => self.translate_basic_expr(expr.clone(), id),
                RvsdgBody::If {
                    pred,
                    inputs,
                    then_branch,
                    else_branch,
                } => {
                    let mut input_values = vec![];
                    for input in inputs {
                        input_values.push(self.translate_operand(*input));
                    }
                    let pred = self
                        .translate_operand(*pred)
                        .to_expr()
                        .expect("Pred was a state edge");

                    let before_current_args = self.current_num_args;
                    let mut then_translator = RegionTranslator::new(
                        self.nodes,
                        input_values.clone(),
                        before_current_args,
                        self.optimize_lets,
                    );

                    let mut resulting_values = vec![];
                    let then_values = then_branch
                        .iter()
                        .filter_map(|operand| {
                            let res = then_translator.translate_operand(*operand);
                            match res {
                                StoredValue::Arg(_) | StoredValue::Expr { .. } => {
                                    resulting_values.push(StoredValue::Arg(self.current_num_args));
                                    self.current_num_args += 1;
                                }
                                StoredValue::StateEdge => {
                                    resulting_values.push(StoredValue::StateEdge);
                                }
                            }

                            res.to_expr()
                        })
                        .collect::<Vec<_>>();
                    let then_expr = parallel_vec(then_values);
                    let then_translated = then_translator.build_translation(then_expr);

                    let mut else_translator = RegionTranslator::new(
                        self.nodes,
                        input_values,
                        before_current_args,
                        self.optimize_lets,
                    );
                    let else_values = else_branch
                        .iter()
                        .filter_map(|operand| else_translator.translate_operand(*operand).to_expr())
                        .collect::<Vec<_>>();
                    let else_expr = parallel_vec(else_values);
                    let else_translated = else_translator.build_translation(else_expr);

                    let expr = tif(pred, then_translated, else_translated);
                    self.add_region_binding(expr, id, resulting_values)
                }
                RvsdgBody::Gamma {
                    pred,
                    inputs,
                    outputs,
                } => {
                    let mut inputs_values = vec![];
                    for input in inputs {
                        inputs_values.push(self.translate_operand(*input));
                    }

                    let pred = self
                        .translate_operand(*pred)
                        .to_expr()
                        .expect("Pred was a state edge");

                    let before_num_args = self.current_num_args;
                    let mut resulting_values = vec![];
                    let mut branches = vec![];
                    for (i, output_vec) in outputs.iter().enumerate() {
                        let mut output_translator = RegionTranslator::new(
                            self.nodes,
                            inputs_values.clone(),
                            before_num_args,
                            self.optimize_lets,
                        );
                        let branch_values = output_vec
                            .iter()
                            .filter_map(|operand| {
                                let res = output_translator.translate_operand(*operand);
                                // during the first iteration, fill in the resulting values
                                if i == 0 {
                                    match res {
                                        StoredValue::Arg(_) | StoredValue::Expr { .. } => {
                                            resulting_values
                                                .push(StoredValue::Arg(self.current_num_args));
                                            self.current_num_args += 1;
                                        }
                                        StoredValue::StateEdge => {
                                            resulting_values.push(StoredValue::StateEdge);
                                        }
                                    }
                                }
                                res.to_expr()
                            })
                            .collect::<Vec<_>>();
                        let branch_expr = parallel_vec(branch_values);
                        let branch_translated = output_translator.build_translation(branch_expr);
                        branches.push(branch_translated);
                    }
                    let expr = switch_vec(pred, branches);
                    self.add_region_binding(expr, id, resulting_values)
                }
                RvsdgBody::Theta {
                    pred,
                    inputs,
                    outputs,
                } => {
                    let mut input_values = vec![];
                    for input in inputs {
                        input_values.push(self.translate_operand(*input));
                    }

                    let mut argument_values = vec![];
                    let mut new_arg_index = 0;
                    for input_val in &input_values {
                        match input_val {
                            StoredValue::Arg(_) | StoredValue::Expr { .. } => {
                                argument_values.push(StoredValue::Arg(new_arg_index));
                                new_arg_index += 1;
                            }
                            StoredValue::StateEdge => {
                                argument_values.push(StoredValue::StateEdge);
                            }
                        }
                    }

                    // For the sub-region, we need a new region translator
                    // with its own arguments and bindings.
                    // We then put the whole loop in a let binding and move on.
                    let mut sub_translator = RegionTranslator::new(
                        self.nodes,
                        argument_values,
                        new_arg_index,
                        self.optimize_lets,
                    );
                    let mut pred_outputs = vec![sub_translator
                        .translate_operand(*pred)
                        .to_expr()
                        .expect("Pred was a state edge")];
                    let outputs_translated = outputs
                        .iter()
                        .map(|o| sub_translator.translate_operand(*o))
                        .collect::<Vec<StoredValue>>();
                    for output in outputs_translated.iter().filter_map(|o| o.to_expr()) {
                        pred_outputs.push(output);
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
                            StoredValue::StateEdge => output_values.push(StoredValue::StateEdge),
                            StoredValue::Arg(_) | StoredValue::Expr { .. } => {
                                output_values.push(StoredValue::Arg(self.current_num_args));
                                self.current_num_args += 1;
                            }
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
                            .expect("State edge in op")
                    })
                    .collect::<Vec<_>>();
                let expr = match (op, children.as_slice()) {
                    (ValueOps::Add, [a, b]) => add(a.clone(), b.clone()),
                    (ValueOps::Lt, [a, b]) => less_than(a.clone(), b.clone()),
                    (ValueOps::Mul, [a, b]) => mul(a.clone(), b.clone()),
                    (ValueOps::Sub, [a, b]) => sub(a.clone(), b.clone()),
                    (ValueOps::Div, [a, b]) => div(a.clone(), b.clone()),
                    (ValueOps::Eq, [a, b]) => eq(a.clone(), b.clone()),
                    (ValueOps::And, [a, b]) => and(a.clone(), b.clone()),
                    _ => todo!("handle {} op", op),
                };
                // All ops handled here are pure
                self.add_pure_value(expr, id)
            }
            BasicExpr::Call(name, inputs, num_ret_values, output_type) => {
                let mut input_values = vec![];
                for input in inputs {
                    input_values.push(self.translate_operand(input));
                }
                let expr = call(
                    name.as_str(),
                    parallel_vec(input_values.into_iter().filter_map(|val| val.to_expr())),
                );
                let mut ret_values = vec![];
                if output_type.is_some() {
                    ret_values.push(StoredValue::Arg(self.current_num_args));
                    self.current_num_args += 1;
                }
                let num_state_edges = num_ret_values - output_type.is_some() as usize;

                // push a state edge for every extra return value
                for _ in 0..num_state_edges {
                    ret_values.push(StoredValue::StateEdge);
                }

                match output_type {
                    None => self.add_region_binding(expr, id, ret_values),
                    Some(_type) => self.add_region_binding(single(expr), id, ret_values),
                }
            }
            BasicExpr::Const(_op, literal, _ty) => match literal {
                Literal::Int(n) => {
                    let expr = int(n);
                    self.add_pure_value(expr, id)
                }
                Literal::Bool(b) => {
                    let expr = if b { ttrue() } else { tfalse() };
                    self.add_pure_value(expr, id)
                }
                _ => todo!("handle other literals"),
            },
            BasicExpr::Effect(EffectOps::Print, args) => {
                assert!(args.len() == 2, "print should have 2 arguments");
                let arg1 = self
                    .translate_operand(args[0])
                    .to_expr()
                    .expect("Print buffer expr should be a single value");
                let arg2 = self.translate_operand(args[1]);

                // Print returns a state edge, which should be translated as
                // a unit value
                assert_eq!(
                    arg2,
                    StoredValue::StateEdge,
                    "Print buffer second argument should be state edge. Found {:?}",
                    arg2
                );
                // print outputs a new unit value
                let expr = tprint(arg1);
                self.add_state_edge_binding(expr, id)
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
    ///
    /// When `optimize_lets` is true, the conversion will also
    /// try to prevent adding unnecessary let bindings.
    pub fn to_tree_encoding(&self, optimize_lets: bool) -> RcExpr {
        let mut arg_index = 0;
        let argument_values = self
            .args
            .iter()
            .map(|ty| match ty {
                RvsdgType::PrintState => StoredValue::StateEdge,
                RvsdgType::Bril(_) => {
                    arg_index += 1;
                    StoredValue::Arg(arg_index - 1)
                }
            })
            .collect();
        let mut translator =
            RegionTranslator::new(&self.nodes, argument_values, arg_index, optimize_lets);
        let translated_results = self
            .results
            .iter()
            .filter_map(|r| translator.translate_operand(r.1).to_expr())
            .collect::<Vec<_>>();
        let result_types = self
            .results
            .iter()
            .filter_map(|r| r.0.to_tree_type())
            .collect::<Vec<_>>();
        let (single_result, single_type) =
            match (translated_results.as_slice(), result_types.as_slice()) {
                ([single_result], [single_type]) => (single_result.clone(), single_type.clone()),
                ([], []) => (empty(), emptyt()),
                _ => panic!("Expected a single result type, found {:?}", result_types),
            };

        function(
            self.name.as_str(),
            TreeType::TupleT(
                self.args
                    .iter()
                    .filter_map(|ty| ty.to_tree_type())
                    .collect(),
            ),
            single_type.clone(),
            translator.build_translation(single_result.clone()),
        )
    }
}

/// Assert that two programs are equal, and
/// test them on a particular input and output value
#[cfg(test)]
fn assert_progs_eq(prog1: &TreeProgram, expected: &TreeProgram, error_msg: &str) {
    if prog1 != expected {
        panic!(
            "{error_msg}\nFound:\n{}\nExpected:\n{}",
            prog1.pretty(),
            expected.pretty()
        );
    }
}

#[cfg(test)]
fn let_translation_test(
    program: &str,
    expected: TreeProgram,
    expected_optimized: TreeProgram,
    input_val: Value,
    expected_val: Value,
    expected_printlog: Vec<String>,
) {
    let prog = parse_from_string(program);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();
    let result = rvsdg.to_tree_encoding(false);
    let result_optimized = rvsdg.to_tree_encoding(true);

    use tree_in_context::interpreter::interpret_tree_prog;

    let (found_val, found_printlog) = interpret_tree_prog(&expected, input_val.clone());
    assert_eq!(
        expected_val, found_val,
        "Reference program produced incorrect result. Expected {:?}, found {:?}",
        expected_val, found_val
    );
    assert_eq!(
        expected_printlog, found_printlog,
        "Reference program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );
    let (found_val, found_printlog) = interpret_tree_prog(&expected_optimized, input_val.clone());
    assert_eq!(
        expected_val, found_val,
        "Reference optimized program produced incorrect result. Expected {:?}, found {:?}",
        expected_val, found_val
    );
    assert_eq!(
        expected_printlog, found_printlog,
        "Reference optimized program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );

    let (found_val, found_printlog) = interpret_tree_prog(&result, input_val.clone());
    assert_eq!(
        expected_val, found_val,
        "Resulting program produced incorrect result. Expected {:?}, found {:?}",
        expected_val, found_val
    );
    assert_eq!(
        expected_printlog, found_printlog,
        "Resulting program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );

    let (found_val, found_printlog) = interpret_tree_prog(&result_optimized, input_val);
    assert_eq!(
        expected_val, found_val,
        "Resulting optimized program produced incorrect result. Expected {:?}, found {:?}",
        expected_val, found_val
    );
    assert_eq!(
        expected_printlog, found_printlog,
        "Resulting optimized program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );

    assert_progs_eq(&result, &expected, "Resulting program is incorrect");
    assert_progs_eq(
        &result_optimized,
        &expected_optimized,
        "Resulting optimized program is incorrect",
    );
}

#[test]
fn simple_translation() {
    let_translation_test(
        r#"
  @add(): int {
    v0: int = const 1;
    res: int = add v0 v0;
    ret res;
  }
  "#,
        program!(function(
            "add",
            TreeType::TupleT(vec![]),
            intt(),
            bind_value(
                int(1),
                bind_value(
                    add(getarg(0), getarg(0)),
                    getarg(1), // returns res
                ),
            )
        ),),
        program!(function(
            "add",
            TreeType::TupleT(vec![]),
            intt(),
            add(int(1), int(1)),
        ),),
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
        vec![],
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
        &rvsdg.to_tree_encoding(false),
        &program!(function(
            "myfunc",
            emptyt(),
            intt(),
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
                        getarg(2) // return 1
                    ),
                )
            )
        ),),
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(1)),
        vec![],
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

    assert_progs_eq(
        &rvsdg.to_tree_encoding(false),
        &program!(function(
            "main",
            TreeType::TupleT(vec![]),
            TreeType::TupleT(vec![]),
            bind_value(
                int(0), // [0]
                bind_tuple(
                    dowhile(
                        parallel!(getarg(0)), // [i]
                        bind_value(
                            int(1), // [i, 1]
                            bind_value(
                                add(getarg(0), getarg(1)), // [i, 1, i+1]
                                bind_value(
                                    int(10), // [i, 1, i+1, 10]
                                    bind_value(
                                        less_than(getarg(2), getarg(3)), // [i, 1, i+1, 10, i+1<10]
                                        parallel!(getarg(4), getarg(2))
                                    )
                                )
                            )
                        )
                    ), // [0, 10]
                    bind_tuple(
                        tprint(getarg(1)), // [0, 10]
                        parallel!()
                    )
                )
            ),
        ),),
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
    );
}

#[test]
fn simple_if_translation() {
    const PROGRAM: &str = r#"
@main(): int {
    .entry:
        v0: int = const 1;
        cond: bool = lt v0 v0;
        br cond .then .else;
    .then:
        ret v0;
    .else:
        v1: int = const 2;
        ret v1;
}"#;

    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    assert_progs_eq(
        &rvsdg.to_tree_encoding(false),
        &program!(function(
            "main",
            emptyt(),
            intt(),
            bind_value(
                int(1), // [1]
                bind_value(
                    less_than(getarg(0), getarg(0)), // [1, 1<1]
                    bind_tuple(
                        tif(
                            getarg(1),
                            parallel!(getarg(0)),
                            bind_value(int(2), parallel!(getarg(2)))
                        ), // [1, 1<1, 2]
                        getarg(2)
                    ),
                ),
            ),
        ),),
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
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

    assert_progs_eq(
        &rvsdg.to_tree_encoding(false),
        &program!(function(
            "add",
            TreeType::TupleT(vec![]),
            TreeType::TupleT(vec![]),
            bind_value(
                int(2), // [2]
                bind_value(
                    int(1), // [2, 1]
                    bind_value(
                        add(getarg(1), getarg(0)), // [2, 1, 3]
                        bind_tuple(
                            tprint(getarg(2)),
                            bind_tuple(tprint(getarg(0)), parallel!()),
                        ),
                    ),
                ),
            )
        ),),
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
    );
}

#[test]
fn multi_function_translation() {
    const PROGRAM: &str = r#"
@myadd(): int {
    v0: int = const 1;
    res: int = add v0 v0;
    ret res;
}

@main() {
    v0: int = call @myadd;
    print v0;
}
"#;
    let prog = parse_from_string(PROGRAM);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();

    assert_progs_eq(
        &rvsdg.to_tree_encoding(false),
        &program!(
            function(
                "main",
                TreeType::TupleT(vec![]),
                TreeType::TupleT(vec![]),
                bind_value(
                    call("myadd", parallel!()),
                    bind_tuple(tprint(getarg(0)), parallel!()),
                ),
            ),
            function(
                "myadd",
                TreeType::TupleT(vec![]),
                intt(),
                bind_value(
                    int(1),
                    bind_value(
                        add(getarg(0), getarg(0)),
                        getarg(1), // returns res
                    ),
                ),
            ),
        ),
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
    );
}
*/
