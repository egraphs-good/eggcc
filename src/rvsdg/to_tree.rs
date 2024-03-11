//! Convert RVSDG programs to the tree
//! RVSDGs are close to this encoding,
//! but use a DAG-based semantics.
//! This means that nodes that are shared
//! are only computed once.
//! These shared nodes need to be let-bound so that they are only
//! computed once in the tree encoded
//! program.
//! When `optimize_lets` is true, the conversion will also
//! prevent adding unnecessary let bindings for pure expressions.

use std::iter;

#[cfg(test)]
use crate::{cfg::program_to_cfg, rvsdg::cfg_to_rvsdg, util::parse_from_string};
use tree_in_context::ast::{
    alloc, and, arg, div, empty, eq, free, getat, greater_eq, less_eq, load, mul, not, ptradd,
    push_par, sub, switch_vec, tif, twrite,
};
#[cfg(test)]
use tree_in_context::ast::{intt, parallel, program};
#[cfg(test)]
use tree_in_context::interpreter::Value;
#[cfg(test)]
use tree_in_context::schema::Constant;

use crate::rvsdg::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};
use bril_rs::{EffectOps, Literal, ValueOps};
use hashbrown::HashMap;
use tree_in_context::{
    ast::{
        add, call, concat_par, dowhile, emptyt, function, int, less_than, parallel_vec,
        program_vec, single, tfalse, tlet, tprint, ttrue,
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
    /// An expression representing this stored value.
    /// Must be a completely pure expression (no reads, writes, prints, or loops).
    Expr(RcExpr),
}

/// Stores the location of the values of
/// an RVSDG node.
/// During translation, values for each node are stored in the bindings.
type StoredNode = Vec<StoredValue>;

impl StoredValue {
    fn to_expr(&self) -> Option<RcExpr> {
        match self {
            StoredValue::Arg(index) => Some(getat(*index)),
            StoredValue::StateEdge => None,
            StoredValue::Expr(expr) => Some(expr.clone()),
        }
    }
}

struct RegionTranslator<'a> {
    /// The values of the RVSDG arguments to this region.
    argument_values: Vec<StoredValue>,
    /// The number of tree arguments in the current environment.
    /// This is not equal to the length of `argument_values` because it refers to
    /// translated tree arguments, not the original RVSDG arguments.
    current_num_let_bound: usize,
    /// a stack of let bindings to generate
    /// each `RcExpr` is an expression producing a tuple.
    /// These tuples are concatenated to the current argument during `build_translation`.
    bindings: Vec<RcExpr>,
    /// `stored_node` is a cache of already translated rvsdg nodes.
    stored_node: HashMap<Id, StoredNode>,
    /// A reference to the nodes in the RVSDG.
    nodes: &'a [RvsdgBody],
    /// Whether to optimize let bindings
    optimize_lets: bool,
}

#[cfg(test)]
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
    /// Essentially inlines all references to this expression instead of binding it.
    /// Importantly, on translation back to RVSDG we should ensure that pure
    /// common subexpressions are not duplicated.
    fn add_pure_value(&mut self, expr: RcExpr, id: Id) -> StoredNode {
        if self.optimize_lets {
            let res = StoredValue::Expr(expr.clone());
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
        let res = vec![StoredValue::Arg(self.current_num_let_bound)];
        if self.current_num_let_bound == 0 {
            self.bindings.push(single(expr));
        } else {
            self.bindings.push(push_par(expr, arg()));
        }

        self.current_num_let_bound += 1;

        assert_eq!(
            self.stored_node.insert(id, res.clone()),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        res
    }

    /// Adds a binding for a state edge (ex. a print or write)
    /// Some stateful operators produce a value (ex. alloc)
    fn add_state_edge_binding(&mut self, expr: RcExpr, id: Id, produces_value: bool) -> StoredNode {
        let res = if produces_value {
            vec![
                StoredValue::Arg(self.current_num_let_bound),
                StoredValue::StateEdge,
            ]
        } else {
            vec![StoredValue::StateEdge]
        };
        if self.current_num_let_bound == 0 {
            if produces_value {
                self.bindings.push(single(expr));
            } else {
                self.bindings.push(expr);
            }
        } else if produces_value {
            self.bindings.push(push_par(expr, arg()));
        } else {
            self.bindings.push(concat_par(arg(), expr));
        }
        if produces_value {
            self.current_num_let_bound += 1;
        }
        assert_eq!(
            self.stored_node.insert(id, res.clone()),
            None,
            "Node already evaluated. Cycle in the RVSDG or similar bug."
        );
        res
    }

    /// Adds a tuple to the bindings.
    /// `values` is a vector refering to each value in the tuple.
    /// `expr_tuple_type_len` refers to the length of expr's tuple type.
    fn add_region_binding(&mut self, expr: RcExpr, id: Id, values: Vec<StoredValue>) -> StoredNode {
        if self.current_num_let_bound == 0 {
            self.bindings.push(expr);
        } else {
            self.bindings.push(concat_par(arg(), expr));
        }
        let num_not_state_edges = values
            .iter()
            .filter(|v| v != &&StoredValue::StateEdge)
            .count();
        self.current_num_let_bound += num_not_state_edges;
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
        args_let_bound: usize,
        optimize_lets: bool,
    ) -> RegionTranslator {
        RegionTranslator {
            current_num_let_bound: args_let_bound,
            bindings: Vec::new(),
            stored_node: HashMap::new(),
            argument_values,
            nodes,
            optimize_lets,
        }
    }

    /// Translate a sub-region by creating a new region
    /// translator and translating the operands.
    /// Returns the translated expression and the resulting
    /// values, assuming the new expression is let-bound
    /// using `add_region_binding`.
    /// Loops don't return their predicate, so skip
    /// the first output using `skip_outputs`.
    fn translate_subregion(
        &mut self,
        argument_values: Vec<StoredValue>,
        num_let_bound: usize,
        operands: impl Iterator<Item = Operand>,
        mut skip_outputs: usize,
    ) -> (RcExpr, Vec<StoredValue>) {
        let mut translator = RegionTranslator::new(
            self.nodes,
            argument_values,
            num_let_bound,
            self.optimize_lets,
        );
        let mut number_non_state_edges = 0;
        let mut resulting_values = vec![];
        let resulting_exprs = operands.filter_map(|operand| {
            let res = translator.translate_operand(operand);
            if skip_outputs > 0 {
                skip_outputs -= 1;
            } else {
                match res {
                    StoredValue::StateEdge => resulting_values.push(StoredValue::StateEdge),
                    _ => {
                        resulting_values.push(StoredValue::Arg(
                            self.current_num_let_bound + number_non_state_edges,
                        ));
                        number_non_state_edges += 1;
                    }
                }
            }
            res.to_expr()
        });
        let expr = parallel_vec(resulting_exprs);
        (translator.build_translation(expr), resulting_values)
    }

    /// Wrap the given expression in all the
    /// bindings that have been generated.
    fn build_translation(&self, inner: RcExpr) -> RcExpr {
        let mut expr = inner;

        for binding in self.bindings.iter().rev() {
            expr = tlet(binding.clone(), expr);
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
                    let (then_translated, resulting_values) = self.translate_subregion(
                        input_values.clone(),
                        self.current_num_let_bound,
                        then_branch.iter().copied(),
                        0,
                    );
                    let (else_translated, _) = self.translate_subregion(
                        input_values.clone(),
                        self.current_num_let_bound,
                        else_branch.iter().copied(),
                        0,
                    );

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

                    let mut resulting_values = vec![];
                    let mut branches = vec![];
                    for output_vec in outputs.iter().enumerate() {
                        let (translated, values) = self.translate_subregion(
                            inputs_values.clone(),
                            self.current_num_let_bound,
                            output_vec.1.iter().copied(),
                            0,
                        );
                        resulting_values = values;
                        branches.push(translated);
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

                    let mut input_index = 0;
                    let inner_inputs = input_values.iter().map(|val| match val {
                        StoredValue::StateEdge => StoredValue::StateEdge,
                        _ => {
                            input_index += 1;
                            StoredValue::Arg(input_index - 1)
                        }
                    });

                    // For the sub-region, we need a new region translator
                    // with its own arguments and bindings.
                    // We then put the whole loop in a let binding and move on.
                    let (loop_translated, output_values) = self.translate_subregion(
                        inner_inputs.collect(),
                        input_index,
                        iter::once(pred).chain(outputs.iter()).copied(),
                        1,
                    );

                    let loop_expr = dowhile(
                        parallel_vec(input_values.into_iter().filter_map(|val| val.to_expr())),
                        loop_translated,
                    );

                    self.add_region_binding(loop_expr, id, output_values)
                }
            }
        }
    }

    /// Translate this expression at the given id,
    /// return the newly created index.
    fn translate_basic_expr(&mut self, expr: BasicExpr<Operand>, id: Id) -> StoredNode {
        match expr {
            BasicExpr::Op(ValueOps::Alloc, children, ty) => {
                let [size, state_edge] = children.as_slice() else {
                    panic!("Alloc should have 2 children, found {:?}", children);
                };
                let bril_rs::Type::Pointer(inner) = ty else {
                    panic!("Alloc should return a pointer type, found {:?}", ty);
                };
                let size = self.translate_operand(*size);
                let state_edge = self.translate_operand(*state_edge);
                assert_eq!(
                    state_edge,
                    StoredValue::StateEdge,
                    "Alloc state edge should be state edge. Found {:?}",
                    state_edge
                );
                let expr = alloc(
                    size.to_expr().expect("Alloc size was a state edge"),
                    TreeType::Base(RvsdgType::Bril(*inner).to_tree_type().unwrap()),
                );
                self.add_state_edge_binding(expr, id, true)
            }
            BasicExpr::Op(ValueOps::Load, children, _ty) => {
                let [address, state_edge] = children.as_slice() else {
                    panic!("Load should have 2 children, found {:?}", children);
                };
                let address = self.translate_operand(*address);
                let state_edge_translated = self.translate_operand(*state_edge);
                assert_eq!(
                    state_edge_translated,
                    StoredValue::StateEdge,
                    "Load state edge should be state edge. Found {:?}",
                    state_edge_translated
                );
                let expr = load(address.to_expr().expect("Load address was a state edge"));

                self.add_state_edge_binding(expr, id, true)
            }
            BasicExpr::Op(op, children, _ty) => {
                let children = children
                    .iter()
                    .map(|c| {
                        self.translate_operand(*c).to_expr().unwrap_or_else(|| {
                            panic!("Found state edge as child of operator{} ", op)
                        })
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
                    (ValueOps::Ge, [a, b]) => greater_eq(a.clone(), b.clone()),
                    (ValueOps::Le, [a, b]) => less_eq(b.clone(), a.clone()),
                    (ValueOps::Not, [a]) => not(a.clone()),
                    (ValueOps::PtrAdd, [a, b]) => ptradd(a.clone(), b.clone()),
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
                    ret_values.push(StoredValue::Arg(self.current_num_let_bound));
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
                let translated = self.translate_operand(args[0]);
                let arg1 = translated
                    .to_expr()
                    .expect("Print buffer expr should be a value, not a state edge");
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
                self.add_state_edge_binding(expr, id, false)
            }
            BasicExpr::Effect(EffectOps::Store, args) => {
                assert!(args.len() == 3, "store should have 3 arguments");
                let arg1 = self
                    .translate_operand(args[0])
                    .to_expr()
                    .expect("Store address");
                let arg2 = self
                    .translate_operand(args[1])
                    .to_expr()
                    .expect("Store value");
                assert_eq!(self.translate_operand(args[2]), StoredValue::StateEdge);
                let expr = twrite(arg1, arg2);
                self.add_state_edge_binding(expr, id, false)
            }
            BasicExpr::Effect(EffectOps::Free, args) => {
                assert!(args.len() == 2, "free should have 2 arguments");
                let arg1 = self
                    .translate_operand(args[0])
                    .to_expr()
                    .expect("Free address was a state edge");
                assert_eq!(self.translate_operand(args[1]), StoredValue::StateEdge);
                let expr = free(arg1);
                self.add_state_edge_binding(expr, id, false)
            }
            BasicExpr::Effect(effect_op, _args) => {
                panic!("Unrecognized effect op {:?}", effect_op)
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
                ([single_result], [single_type]) => {
                    (single_result.clone(), TreeType::Base(single_type.clone()))
                }
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

    let (found_val, found_printlog) = interpret_tree_prog(&expected, &input_val);
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
    let (found_val, found_printlog) = interpret_tree_prog(&expected_optimized, &input_val);
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

    let (found_val, found_printlog) = interpret_tree_prog(&result, &input_val);
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

    let (found_val, found_printlog) = interpret_tree_prog(&result_optimized, &input_val);
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
            tlet(
                single(int(1)),
                bind_value(
                    add(getat(0), getat(0)),
                    getat(1), // returns res
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
    let_translation_test(
        PROGRAM,
        program!(function(
            "myfunc",
            emptyt(),
            intt(),
            tlet(
                single(int(1)), // [1]
                bind_value(
                    int(2), // [1, 2]
                    bind_tuple(
                        dowhile(
                            parallel!(getat(0), getat(1)), // [1, 2]
                            bind_value(
                                less_than(getat(1), getat(0)), // [1, 2, 2<1]
                                parallel!(getat(2), getat(0), getat(1))
                            )
                        ), // [1, 2, 1, 2]
                        getat(2) // return 1
                    ),
                )
            )
        ),),
        program!(function(
            "myfunc",
            emptyt(),
            intt(),
            tlet(
                dowhile(
                    parallel!(int(1), int(2)),
                    parallel!(less_than(getat(1), getat(0)), getat(0), getat(1))
                ),
                getat(0)
            ),
        ),),
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(1)),
        vec![],
    );
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

    let_translation_test(
        PROGRAM,
        program!(function(
            "main",
            TreeType::TupleT(vec![]),
            TreeType::TupleT(vec![]),
            tlet(
                single(int(0)), // [0]
                bind_tuple(
                    dowhile(
                        parallel!(getat(0)), // [i]
                        bind_value(
                            int(1), // loop: [i, 1]
                            bind_value(
                                add(getat(0), getat(1)), // [i, 1, i+1]
                                bind_value(
                                    int(10), // [i, 1, i+1, 10]
                                    bind_value(
                                        less_than(getat(2), getat(3)), // [i, 1, i+1, 10, i+1<10]
                                        parallel!(getat(4), getat(2))
                                    )
                                )
                            )
                        )
                    ), // [0, 10]
                    bind_tuple(
                        tprint(getat(1)), // [0, 10]
                        parallel!()
                    )
                )
            ),
        ),),
        program!(function(
            "main",
            TreeType::TupleT(vec![]),
            TreeType::TupleT(vec![]),
            tlet(
                dowhile(
                    parallel!(int(0)),
                    parallel!(
                        less_than(add(getat(0), int(1)), int(10)),
                        add(getat(0), int(1))
                    )
                ),
                bind_tuple(tprint(getat(0)), parallel!())
            ),
        ),),
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec!["10".to_string()],
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

    let_translation_test(
        PROGRAM,
        program!(function(
            "main",
            emptyt(),
            intt(),
            tlet(
                single(int(1)), // [1]
                bind_value(
                    less_than(getat(0), getat(0)), // [1, 1<1]
                    bind_tuple(
                        tif(
                            getat(1),
                            parallel!(getat(0)),
                            bind_value(int(2), parallel!(getat(2)))
                        ), // [1, 1<1, 2]
                        getat(2)
                    ),
                ),
            ),
        ),),
        program!(function(
            "main",
            emptyt(),
            intt(),
            tlet(
                tif(
                    less_than(int(1), int(1)),
                    parallel!(int(1)),
                    parallel!(int(2)),
                ),
                getat(0)
            )
        ),),
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(2)),
        vec![],
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
    let_translation_test(
        PROGRAM,
        program!(function(
            "add",
            TreeType::TupleT(vec![]),
            TreeType::TupleT(vec![]),
            tlet(
                single(int(2)), // [2]
                bind_value(
                    int(1), // [2, 1]
                    bind_value(
                        add(getat(1), getat(0)), // [2, 1, 3]
                        bind_tuple(tprint(getat(2)), bind_tuple(tprint(getat(0)), parallel!()),),
                    ),
                ),
            )
        ),),
        program!(function(
            "add",
            TreeType::TupleT(vec![]),
            TreeType::TupleT(vec![]),
            tlet(
                tprint(add(int(1), int(2))),
                tlet(tprint(int(2)), parallel!(),)
            )
        ),),
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec!["3".to_string(), "2".to_string()],
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

    let_translation_test(
        PROGRAM,
        program!(
            function(
                "main",
                TreeType::TupleT(vec![]),
                TreeType::TupleT(vec![]),
                tlet(
                    single(call("myadd", parallel!())),
                    bind_tuple(tprint(getat(0)), parallel!()),
                ),
            ),
            function(
                "myadd",
                TreeType::TupleT(vec![]),
                intt(),
                tlet(
                    single(int(1)),
                    bind_value(
                        add(getat(0), getat(0)),
                        getat(1), // returns res
                    ),
                ),
            ),
        ),
        program!(
            function(
                "main",
                TreeType::TupleT(vec![]),
                TreeType::TupleT(vec![]),
                tlet(
                    single(call("myadd", parallel!())),
                    bind_tuple(tprint(getat(0)), parallel!()),
                ),
            ),
            function(
                "myadd",
                TreeType::TupleT(vec![]),
                intt(),
                add(int(1), int(1)),
            ),
        ),
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec!["2".to_string()],
    );
}
