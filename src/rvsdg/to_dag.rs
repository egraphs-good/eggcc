//! Convert RVSDG programs to the dag encoding, adding context nodes as we go.
//! This is a fairly direct translation, with a minor difference being
//! that if nodes do not create a region.
//! When we translate if nodes, we add context nodes at the inputs to the region.
//! Common sub-expressions can still be shared across branches, avoiding blowup from context nodes.
//! We are careful to add context to every leaf node (Empty, Arg, and Const)
use std::iter;

#[cfg(test)]
use crate::{cfg::program_to_cfg, rvsdg::cfg_to_rvsdg, util::parse_from_string};
#[cfg(test)]
use dag_in_context::interpreter::Value;
#[cfg(test)]
use dag_in_context::schema::Constant;
use dag_in_context::{ast::*, schema::Assumption};

use crate::rvsdg::{BasicExpr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};
use bril_rs::{EffectOps, Literal, ValueOps};
use dag_in_context::{
    ast::{add, call, dowhile, function, int, less_than, parallel_vec, program_vec, tfalse, ttrue},
    schema::{RcExpr, TreeProgram, Type},
};
use hashbrown::HashMap;

use super::RvsdgType;

impl RvsdgProgram {
    /// Converts an RVSDG program to the dag encoding.
    /// Common subexpressions are shared by the same Rc<Expr> in the dag encoding.
    /// This invariant is mainted by restore_sharing_invariant.
    pub fn to_dag_encoding(&self, add_context: bool) -> TreeProgram {
        let last_function = self.functions.last().unwrap();
        let rest_functions = self.functions.iter().take(self.functions.len() - 1);
        program_vec(
            last_function.to_dag_encoding(add_context),
            rest_functions
                .map(|f| f.to_dag_encoding(add_context))
                .collect::<Vec<_>>(),
        )
        .restore_sharing_invariant()
    }
}

/// We cache a stored value for each node in the RVSDG, representing the corresponding
/// expression in the dag encoding.
/// These can either be tuple typed or single typed, and we need to track this to properly translate projects.
#[derive(Clone, Debug)]
struct StoredValue {
    is_tuple: bool,
    expr: RcExpr,
}

impl StoredValue {
    fn to_single_expr(&self) -> RcExpr {
        if self.is_tuple {
            panic!("Cannot convert tuple to single expr. Got {:?}", self)
        } else {
            self.expr.clone()
        }
    }

    fn project(&self, index: usize) -> StoredValue {
        if self.is_tuple {
            StoredValue {
                is_tuple: false,
                expr: get(self.expr.clone(), index),
            }
        } else {
            assert_eq!(
                index, 0,
                "Tried to access index {} of non-tuple value",
                index
            );
            self.clone()
        }
    }
}

struct DagTranslator<'a> {
    /// The values of the RVSDG arguments to this region.
    argument_values: Vec<StoredValue>,
    /// `stored_node` is a cache of already translated rvsdg nodes.
    stored_node: HashMap<Id, StoredValue>,
    /// A reference to the nodes in the RVSDG.
    nodes: &'a [RvsdgBody],
    /// The next id to assign to an alloc.
    next_alloc_id: i64,
    /// The current context of the translation. None if we are not adding context.
    current_ctx: Option<Assumption>,
    /// The translator without context, used to reference the translated code for loop bodies.
    without_context: Option<Box<DagTranslator<'a>>>,
}

impl<'a> DagTranslator<'a> {
    /// Adds a pure expression to the cache.
    /// Essentially inlines all references to this expression instead of binding it.
    /// Importantly, on translation back to RVSDG we should ensure that pure
    /// common subexpressions are not duplicated.
    fn cache_single_res(&mut self, expr: RcExpr, id: Id) -> StoredValue {
        let res = StoredValue {
            is_tuple: false,
            expr: expr.clone(),
        };
        self.stored_node.insert(id, res.clone());
        res
    }

    fn cache_tuple_res(&mut self, expr: RcExpr, id: Id) -> StoredValue {
        let res = StoredValue {
            is_tuple: true,
            expr: expr.clone(),
        };
        self.stored_node.insert(id, res.clone());
        res
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

        operands: impl Iterator<Item = Operand> + DoubleEndedIterator,
        new_context: Option<Assumption>,
    ) -> RcExpr {
        let old_argument_values = std::mem::replace(&mut self.argument_values, argument_values);
        let old_context = std::mem::replace(&mut self.current_ctx, new_context);
        let ctx = self.current_ctx.clone();
        let resulting_exprs = operands.map(|operand| {
            let res = self.translate_operand(operand);
            res.to_single_expr()
        });
        let expr = parallel_vec_with_ctx(resulting_exprs, ctx);

        // restore argument values
        self.argument_values = old_argument_values;
        // restore context
        self.current_ctx = old_context;
        expr
    }

    fn translate_operand(&mut self, operand: Operand) -> StoredValue {
        match operand {
            Operand::Arg(index) => self.argument_values[index].clone(),
            Operand::Id(id) => self.translate_node(id).project(0),
            Operand::Project(p_index, id) => self.translate_node(id).project(p_index),
        }
    }

    /// Translate a node or return the index of the already-translated node.
    /// For regions, translates the region and returns the index of the
    /// tuple containing the results.
    /// It's important not to evaluate a node twice, instead using the cached index
    /// in `self.stored_node`
    fn translate_node(&mut self, id: Id) -> StoredValue {
        if let Some(stored) = self.stored_node.get(&id) {
            stored.clone()
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
                    let pred = self.translate_operand(*pred).to_single_expr();
                    let then_context = if self.current_ctx.is_some() {
                        Some(Assumption::InIf(true, pred.clone()))
                    } else {
                        None
                    };
                    let then_translated = self.translate_subregion(
                        input_values.clone(),
                        then_branch.iter().copied(),
                        then_context,
                    );
                    let else_context = if self.current_ctx.is_some() {
                        Some(Assumption::InIf(false, pred.clone()))
                    } else {
                        None
                    };

                    let else_translated = self.translate_subregion(
                        input_values.clone(),
                        else_branch.iter().copied(),
                        else_context,
                    );

                    let expr = tif(pred, then_translated, else_translated);
                    self.cache_tuple_res(expr, id)
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

                    let pred = self.translate_operand(*pred).to_single_expr();

                    let mut branches = vec![];
                    for output_vec in outputs.iter().enumerate() {
                        let translated = self.translate_subregion(
                            inputs_values.clone(),
                            output_vec.1.iter().copied(),
                            self.current_ctx.clone(),
                        );
                        branches.push(translated);
                    }
                    let expr = switch_vec(pred, branches);
                    self.cache_tuple_res(expr, id)
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

                    let inputs_translated = parallel_vec_with_ctx(
                        input_values.iter().map(|val| val.to_single_expr()),
                        self.current_ctx.clone(),
                    );

                    let loop_ctx = if self.current_ctx.is_some() {
                        let already_translated_outputs =
                            parallel_vec(outputs.iter().map(|output| {
                                self.without_context
                                    .as_mut()
                                    .unwrap()
                                    .translate_operand(*output)
                                    .to_single_expr()
                            }));
                        Some(Assumption::InLoop(
                            inputs_translated.clone(),
                            already_translated_outputs,
                        ))
                    } else {
                        None
                    };

                    let mut input_index = 0;

                    let inner_inputs = input_values.iter().map(|_val| {
                        input_index += 1;
                        StoredValue {
                            is_tuple: false,
                            // add context to inner inputs
                            expr: if let Some(inner_ctx) = &loop_ctx {
                                get(in_context(inner_ctx.clone(), arg()), input_index - 1)
                            } else {
                                get(arg(), input_index - 1)
                            },
                        }
                    });

                    // For the sub-region, we need a new region translator
                    // with its own arguments and bindings.
                    // We then put the whole loop in a let binding and move on.
                    let loop_translated = self.translate_subregion(
                        inner_inputs.collect(),
                        iter::once(pred).chain(outputs.iter()).copied(),
                        loop_ctx,
                    );

                    let loop_expr = dowhile(inputs_translated, loop_translated);

                    self.cache_tuple_res(loop_expr, id)
                }
            }
        }
    }

    /// Translate this expression at the given id,
    /// return the newly created index.
    fn translate_basic_expr(&mut self, expr: BasicExpr<Operand>, id: Id) -> StoredValue {
        match expr {
            BasicExpr::Op(op, children, ty) => {
                let children = children
                    .iter()
                    .map(|c| self.translate_operand(*c).to_single_expr())
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
                    (ValueOps::Le, [a, b]) => less_eq(a.clone(), b.clone()),
                    (ValueOps::Not, [a]) => not(a.clone()),
                    (ValueOps::PtrAdd, [a, b]) => ptradd(a.clone(), b.clone()),
                    (ValueOps::Load, [a, b]) => load(a.clone(), b.clone()),
                    (ValueOps::Alloc, [a, b]) => {
                        let bril_rs::Type::Pointer(_inner) = &ty else {
                            panic!("Alloc should return a pointer type, found {:?}", ty);
                        };
                        let alloc_id = self.next_alloc_id;
                        self.next_alloc_id += 1;
                        alloc(
                            alloc_id,
                            a.clone(),
                            b.clone(),
                            RvsdgType::Bril(ty).to_tree_type().unwrap(),
                        )
                    }
                    _ => todo!("handle {} op", op),
                };
                match op {
                    ValueOps::Alloc | ValueOps::Load => self.cache_tuple_res(expr, id),
                    _ => self.cache_single_res(expr, id),
                }
            }
            BasicExpr::Call(name, inputs, _num_ret_values, _output_type) => {
                let mut input_values = vec![];
                for input in inputs {
                    input_values.push(self.translate_operand(input));
                }
                let expr = call(
                    name.as_str(),
                    parallel_vec_with_ctx(
                        input_values.into_iter().map(|val| val.to_single_expr()),
                        self.current_ctx.clone(),
                    ),
                );
                self.cache_tuple_res(expr, id)
            }
            BasicExpr::Const(_op, literal, _ty) => {
                let lit_expr = match literal {
                    Literal::Int(n) => int(n),
                    Literal::Bool(b) => {
                        if b {
                            ttrue()
                        } else {
                            tfalse()
                        }
                    }
                    _ => todo!("handle other literals"),
                };
                let with_ctx = if let Some(ctx) = &self.current_ctx {
                    in_context(ctx.clone(), lit_expr)
                } else {
                    lit_expr
                };
                self.cache_single_res(with_ctx, id)
            }
            BasicExpr::Effect(EffectOps::Print, args) => {
                assert!(args.len() == 2, "print should have 2 arguments");
                let translated = self.translate_operand(args[0]);
                let arg1 = translated.to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();

                // print outputs a new unit value
                let expr = tprint(arg1, arg2);
                self.cache_single_res(expr, id)
            }
            BasicExpr::Effect(EffectOps::Store, args) => {
                assert!(args.len() == 3, "store should have 3 arguments");
                let arg1 = self.translate_operand(args[0]).to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();
                let arg3 = self.translate_operand(args[2]).to_single_expr();
                let expr = twrite(arg1, arg2, arg3);
                self.cache_single_res(expr, id)
            }
            BasicExpr::Effect(EffectOps::Free, args) => {
                assert!(args.len() == 2, "free should have 2 arguments");
                let arg1 = self.translate_operand(args[0]).to_single_expr();
                let arg2 = self.translate_operand(args[1]).to_single_expr();
                let expr = free(arg1, arg2);
                self.cache_single_res(expr, id)
            }
            BasicExpr::Effect(effect_op, _args) => {
                panic!("Unrecognized effect op {:?}", effect_op)
            }
        }
    }
}

impl RvsdgFunction {
    fn to_dag_encoding_helper<'a>(
        &'a self,
        ctx: Option<Assumption>,
        without_context: Option<Box<DagTranslator<'a>>>,
    ) -> (DagTranslator<'a>, RcExpr) {
        let argument_values: Vec<StoredValue> = self
            .args
            .iter()
            .enumerate()
            .map(|(i, _ty)| StoredValue {
                is_tuple: false,
                expr: if let Some(ctx) = &ctx {
                    get(in_context(ctx.clone(), arg()), i)
                } else {
                    get(arg(), i)
                },
            })
            .collect();
        let mut translator = DagTranslator {
            argument_values: argument_values.clone(),
            stored_node: HashMap::new(),
            nodes: &self.nodes,
            next_alloc_id: 0,
            current_ctx: ctx.clone(),
            without_context,
        };

        let translated_results = self
            .results
            .iter()
            .map(|r| translator.translate_operand(r.1).to_single_expr())
            .collect::<Vec<_>>();
        let result_types = self
            .results
            .iter()
            .filter_map(|r| r.0.to_tree_type())
            .collect::<Vec<_>>();

        (
            translator,
            function(
                self.name.as_str(),
                Type::TupleT(
                    self.args
                        .iter()
                        .filter_map(|ty| ty.to_tree_type())
                        .collect(),
                ),
                tuplet_vec(result_types),
                parallel_vec_with_ctx(translated_results, ctx),
            ),
        )
    }

    /// Translates an RVSDG function to the
    /// tree encoding.
    /// It generates one let binding per
    /// node in the RVSDG, adding the value
    /// for that node to the end of the argument
    /// using the `concat` constructor.
    /// In the inner-most scope, the value of
    /// all nodes is available.
    pub fn to_dag_encoding(&self, add_context: bool) -> RcExpr {
        let (without_ctx, expr) = self.to_dag_encoding_helper(None, None);
        if add_context {
            let (_with_ctx, expr_with_ctx) = self.to_dag_encoding_helper(
                Some(Assumption::InFunc(self.name.clone())),
                Some(Box::new(without_ctx)),
            );
            expr_with_ctx
        } else {
            expr
        }
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
fn dag_translation_test(
    program: &str,
    expected: TreeProgram,
    input_val: Value,
    expected_val: Value,
    expected_printlog: Vec<String>,
    add_context: bool,
) {
    use dag_in_context::interpreter::interpret_dag_prog;

    let prog = parse_from_string(program);
    let cfg = program_to_cfg(&prog);
    let rvsdg = cfg_to_rvsdg(&cfg).unwrap();
    let result = rvsdg.to_dag_encoding(add_context);

    let (found_val, found_printlog) = interpret_dag_prog(&expected, &input_val);
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
    assert_eq!(
        expected_printlog, found_printlog,
        "Reference optimized program produced incorrect print log. Expected {:?}, found {:?}",
        expected_printlog, found_printlog
    );

    let (found_val, found_printlog) = interpret_dag_prog(&result, &input_val);
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
}

#[test]
fn simple_translation_dag() {
    let simple_add = r#"
    @add(): int {
      v0: int = const 1;
      res: int = add v0 v0;
      ret res;
    }
    "#;
    dag_translation_test(
        simple_add,
        program!(function(
            "add",
            Type::TupleT(vec![statet()]),
            tuplet!(intt(), statet()),
            parallel!(add(int(1), int(1)), getat(0)),
        ),),
        tuplev!(statev()),
        tuplev!(intv(2), statev()),
        vec![],
        false,
    );

    dag_translation_test(
        simple_add,
        program!(function(
            "add",
            Type::TupleT(vec![statet()]),
            tuplet!(intt(), statet()),
            parallel!(
                add(
                    in_context(infunc("add"), int(1)),
                    in_context(infunc("add"), int(1))
                ),
                get(in_context(infunc("add"), arg()), 0)
            ),
        ),),
        tuplev!(statev()),
        tuplev!(intv(2), statev()),
        vec![],
        true,
    );
}

#[test]
fn dag_translate_simple_loop() {
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
    let doloop = dowhile(
        parallel!(getat(0), int(1), int(2)),
        parallel!(less_than(getat(2), getat(1)), getat(0), getat(1), getat(2)),
    );
    dag_translation_test(
        PROGRAM,
        program!(function(
            "myfunc",
            tuplet!(statet()),
            tuplet!(intt(), statet()),
            parallel!(get(doloop.clone(), 1), get(doloop, 0))
        ),),
        Value::Tuple(vec![]),
        Value::Const(Constant::Int(1)),
        vec![],
        false,
    );
}

#[test]
fn dag_translate_loop() {
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

    let myloop = dowhile(
        parallel!(getat(0), int(0)),
        parallel!(
            less_than(add(getat(1), int(1)), int(10)),
            getat(0),
            add(getat(1), int(1))
        ),
    );
    let myprint = tprint(get(myloop.clone(), 1), get(myloop, 0));
    dag_translation_test(
        PROGRAM,
        program!(function(
            "main",
            tuplet!(statet()),
            tuplet!(statet()),
            parallel!(myprint),
        ),),
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec!["10".to_string()],
        false,
    );
}

#[test]
fn dag_if_translation() {
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

    let myif = tif(
        less_than(int(1), int(1)),
        parallel!(getat(0), int(1)),
        parallel!(getat(0), int(2)),
    );
    dag_translation_test(
        PROGRAM,
        program!(function(
            "main",
            tuplet!(statet()),
            tuplet!(intt(), statet()),
            parallel!(get(myif.clone(), 1), get(myif, 0)),
        ),),
        Value::Tuple(vec![statev()]),
        tuplev!(intv(2), statev()),
        vec![],
        false,
    );
}

#[test]
fn dag_print_translation() {
    const PROGRAM: &str = r#"
    @add() {
        v0: int = const 1;
        v1: int = const 2;
        v2: int = add v0 v1;
        print v2;
        print v1;
    }
    "#;
    let first_print = tprint(add(int(1), int(2)), getat(0));
    let second_print = tprint(int(2), first_print);
    dag_translation_test(
        PROGRAM,
        program!(function(
            "add",
            tuplet!(statet()),
            tuplet!(statet()),
            parallel!(second_print),
        ),),
        tuplev!(statev()),
        tuplev!(statev()),
        vec!["3".to_string(), "2".to_string()],
        false,
    );
}

#[test]
fn dag_multi_function_translation() {
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

    let mycall = call("myadd", parallel!(getat(0)));
    dag_translation_test(
        PROGRAM,
        program!(
            function(
                "main",
                tuplet!(statet()),
                tuplet!(statet()),
                parallel!(tprint(get(mycall.clone(), 0), get(mycall, 1))),
            ),
            function(
                "myadd",
                tuplet!(statet()),
                tuplet!(intt(), statet()),
                parallel!(add(int(1), int(1)), getat(0)),
            ),
        ),
        tuplev!(statev()),
        tuplev!(statev()),
        vec!["2".to_string()],
        false,
    );
}
