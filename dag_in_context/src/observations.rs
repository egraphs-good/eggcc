//! Bounded concrete Bool observations for diagnostic translation validation.
//!
//! This is deliberately not a general execution tracer. It follows one call-free,
//! loop-free feasible path, traverses Print/state carriers structurally, and only
//! evaluates the pure grammar below. Expr and EvalBoolAt remain unmarked.
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use clap::ValueEnum;
use egglog::{EGraph, TermId};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    interpreter::{interpret_expr, Value},
    print_with_intermediate_helper,
    schema::{Assumption, BaseType, BinaryOp, Constant, Expr, RcExpr, TreeProgram, Type, UnaryOp},
    to_egglog::TreeToEgglog,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ObservationMode {
    #[default]
    Off,
    CommonSeed,
    Manual,
    Injective,
}

#[derive(Clone, Debug)]
pub struct ObservationConfig {
    pub mode: ObservationMode,
    /// Actual host inputs, including the trailing StateV in the root tuple.
    pub root_inputs: Vec<Value>,
    /// Evidence root for optimize calls; each enabled call allocates one persistent child.
    /// Direct build_program callers supply their own fresh prepared directory instead.
    pub evidence_dir: PathBuf,
}

/// Successful calls expose this locator in normal run data. Failed diagnostics
/// retain the same locator in full stderr before collecting any observations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationEvidence {
    pub directory: PathBuf,
    pub index: PathBuf,
}

fn atomic_json(path: &Path, value: &serde_json::Value) -> io::Result<()> {
    let mut temporary = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    temporary.write_all(&serde_json::to_vec_pretty(value)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

pub(crate) fn allocate_invocation(
    config: &ObservationConfig,
) -> Result<(ObservationConfig, ObservationEvidence), egglog::Error> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fs::create_dir_all(&config.evidence_dir).map_err(|error| {
        egglog::Error::BackendError(format!(
            "observation evidence root {}: {error}",
            config.evidence_dir.display()
        ))
    })?;
    let root = config.evidence_dir.canonicalize().map_err(|error| {
        egglog::Error::BackendError(format!(
            "observation evidence root {}: {error}",
            config.evidence_dir.display()
        ))
    })?;
    for _ in 0..1024 {
        let counter = NEXT.fetch_add(1, Ordering::Relaxed);
        let directory = root.join(format!("invocation-{}-{counter}", std::process::id()));
        match fs::create_dir(&directory) {
            Ok(()) => {
                let evidence = ObservationEvidence {
                    index: directory.join("index.json"),
                    directory,
                };
                // Print before writing the initial index as well: an I/O failure
                // leaves a locator to an incomplete child, never a success claim.
                eprintln!(
                    "observation-evidence: {}",
                    serde_json::to_string(&evidence).unwrap()
                );
                let initial = json!({"schema_version":1,"pid":std::process::id(),
                    "requested_root":config.evidence_dir,"root":root,"directory":evidence.directory,"index":evidence.index,
                    "mode":format!("{:?}",config.mode),"stage":"invocation-started","status":"in-progress",
                    "passes":{},"events":[]});
                atomic_json(&evidence.index, &initial).map_err(|error| {
                    egglog::Error::BackendError(format!(
                        "observation evidence index {}: {error}",
                        evidence.index.display()
                    ))
                })?;
                let mut prepared = config.clone();
                prepared.evidence_dir = evidence.directory.clone();
                return Ok((prepared, evidence));
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(egglog::Error::BackendError(format!(
                    "observation invocation allocation {}: {error}",
                    directory.display()
                )))
            }
        }
    }
    Err(egglog::Error::BackendError(format!(
        "observation invocation allocation exhausted 1024 collision retries under {}",
        root.display()
    )))
}

impl ObservationEvidence {
    pub(crate) fn update(
        &self,
        pass: Option<usize>,
        stage: &str,
        terminal: Option<&str>,
    ) -> Result<(), egglog::Error> {
        let update = || -> io::Result<()> {
            let mut index: serde_json::Value = serde_json::from_slice(&fs::read(&self.index)?)?;
            if index["directory"] != json!(self.directory)
                || index["index"] != json!(self.index)
                || index["pid"] != std::process::id()
            {
                return Err(io::Error::other("invocation index identity mismatch"));
            }
            if let Some(pass) = pass {
                let directory = self.directory.join(format!("pass-{pass:02}"));
                let mut files: Vec<_> = fs::read_dir(&directory)?
                    .map(|entry| {
                        let entry = entry?;
                        Ok(entry.path())
                    })
                    .collect::<io::Result<_>>()?;
                files.sort();
                index["passes"][format!("{pass:02}")] =
                    json!({"stage":stage,"directory":directory,"evidence":files});
            }
            index["events"]
                .as_array_mut()
                .ok_or_else(|| io::Error::other("invalid invocation events"))?
                .push(json!({"pass":pass,"stage":stage}));
            index["stage"] = json!(stage);
            if let Some(status) = terminal {
                index["status"] = json!(status);
            }
            atomic_json(&self.index, &index)
        };
        update().map_err(|error| {
            egglog::Error::BackendError(format!(
                "observation index update {} at {stage}: {error}",
                self.index.display()
            ))
        })
    }
}

#[derive(Debug)]
struct Point {
    token: i64,
    input_index: usize,
    parent: Option<i64>,
    branch: Option<bool>,
    context: Assumption,
    input: Value,
    input_type: Type,
}

#[derive(Debug)]
struct Observation {
    expr: RcExpr,
    point: i64,
    value: bool,
}

#[derive(Debug)]
struct SkippedRegion {
    parent: i64,
    branch: bool,
    /// No evaluation or observation is performed in this region.
    observations: usize,
}

#[derive(Debug, Default)]
struct Collected {
    points: Vec<Point>,
    observations: Vec<Observation>,
    skipped_regions: Vec<SkippedRegion>,
    structural_print_visits: usize,
}

#[derive(Clone)]
struct CheckedValue {
    value: Value,
    /// Contains no Print or region transition; safe for detached interpretation
    /// after this collector has checked all leaves, types and operations.
    pure: bool,
}

fn value_type(value: &Value) -> Result<Type, String> {
    Ok(match value {
        Value::Const(Constant::Int(_)) => Type::Base(BaseType::IntT),
        Value::Const(Constant::Bool(_)) => Type::Base(BaseType::BoolT),
        Value::StateV => Type::Base(BaseType::StateT),
        Value::Tuple(values) => Type::TupleT(
            values
                .iter()
                .map(|v| match value_type(v)? {
                    Type::Base(base) => Ok(base),
                    _ => Err("observation domain: nested tuple input/type is unsupported".into()),
                })
                .collect::<Result<_, String>>()?,
        ),
        _ => return Err("observation domain: floats and pointers are unsupported".into()),
    })
}

fn collect(
    program: &TreeProgram,
    fns: &[String],
    inputs: &[Value],
    pass: usize,
) -> Result<Collected, String> {
    let Expr::Function(name, arg_type, output_type, body) = program.entry.as_ref() else {
        return Err("observation domain: entry is not a Function".into());
    };
    if !program.functions.is_empty() || fns != [name.clone()] || inputs.is_empty() {
        return Err(
            "observation domain: requires one entry function/batch and at least one actual input"
                .into(),
        );
    }
    // A pass gets 2^32 opaque tokens. Root inputs and selected nested regions
    // allocate monotonically within that range, never by semantic value.
    let base = i64::try_from(pass)
        .ok()
        .and_then(|p| p.checked_mul(1_i64 << 32))
        .ok_or("observation domain: pass token range overflow")?;
    let mut collected = Collected::default();
    for (input_index, input) in inputs.iter().enumerate() {
        if &value_type(input)? != arg_type {
            return Err(format!(
                "observation domain: root input type mismatch, expected {arg_type:?}"
            ));
        }
        let Value::Tuple(values) = input else {
            return Err("observation domain: root input must be a tuple ending in StateV".into());
        };
        if values.last() != Some(&Value::StateV)
            || values[..values.len() - 1].contains(&Value::StateV)
        {
            return Err(
                "observation domain: root input must have exactly one trailing StateV".into(),
            );
        }
        let result = collected.region(
            body,
            input.clone(),
            Assumption::InFunc(name.clone()),
            input_index,
            None,
            None,
            base,
        )?;
        if &value_type(&result.value)? != output_type {
            return Err("observation domain: function output type mismatch".into());
        }
    }
    Ok(collected)
}

impl Collected {
    #[allow(clippy::too_many_arguments)]
    fn region(
        &mut self,
        body: &RcExpr,
        input: Value,
        context: Assumption,
        input_index: usize,
        parent: Option<i64>,
        branch: Option<bool>,
        base: i64,
    ) -> Result<CheckedValue, String> {
        let offset =
            i64::try_from(self.points.len()).map_err(|_| "observation domain: too many regions")?;
        if offset >= 1_i64 << 32 {
            return Err("observation domain: too many region tokens".into());
        }
        let token = base
            .checked_add(offset)
            .ok_or("observation domain: token overflow")?;
        let input_type = value_type(&input)?;
        self.points.push(Point {
            token,
            input_index,
            parent,
            branch,
            context: context.clone(),
            input: input.clone(),
            input_type: input_type.clone(),
        });
        self.visit(
            body,
            &input,
            &input_type,
            &context,
            token,
            input_index,
            base,
            &mut HashMap::new(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn visit(
        &mut self,
        expr: &RcExpr,
        input: &Value,
        input_type: &Type,
        context: &Assumption,
        point: i64,
        input_index: usize,
        base: i64,
        cache: &mut HashMap<*const Expr, CheckedValue>,
    ) -> Result<CheckedValue, String> {
        if let Some(value) = cache.get(&Rc::as_ptr(expr)) {
            return Ok(value.clone());
        }
        // Every recursive call below retains this exact region environment.
        macro_rules! child {
            ($expr:expr) => {
                self.visit(
                    $expr,
                    input,
                    input_type,
                    context,
                    point,
                    input_index,
                    base,
                    cache,
                )?
            };
        }
        let checked = match expr.as_ref() {
            Expr::Const(constant, ty, ctx) => {
                if ty != input_type || ctx != context {
                    return Err("observation domain: Const argument type/context mismatch".into());
                }
                let value = Value::Const(constant.clone());
                value_type(&value)?;
                CheckedValue { value, pure: true }
            }
            Expr::Arg(ty, ctx) => {
                if ty != input_type || ctx != context {
                    return Err("observation domain: Arg type/context mismatch".into());
                }
                CheckedValue {
                    value: input.clone(),
                    pure: true,
                }
            }
            Expr::Empty(ty, ctx) => {
                if ty != input_type || ctx != context {
                    return Err("observation domain: Empty argument type/context mismatch".into());
                }
                CheckedValue {
                    value: Value::Tuple(vec![]),
                    pure: true,
                }
            }
            Expr::Single(inner) => {
                let inner = child!(inner);
                if !matches!(value_type(&inner.value)?, Type::Base(_)) {
                    return Err("observation domain: Single cannot nest tuples".into());
                }
                CheckedValue {
                    value: Value::Tuple(vec![inner.value]),
                    pure: inner.pure,
                }
            }
            Expr::Concat(left, right) => {
                let left = child!(left);
                let right = child!(right);
                let (Value::Tuple(mut lhs), Value::Tuple(rhs)) = (left.value, right.value) else {
                    return Err("observation domain: Concat needs tuples".into());
                };
                lhs.extend(rhs);
                CheckedValue {
                    value: Value::Tuple(lhs),
                    pure: left.pure && right.pure,
                }
            }
            Expr::Get(tuple, index) => {
                let tuple = child!(tuple);
                let Value::Tuple(values) = tuple.value else {
                    return Err("observation domain: Get needs a tuple".into());
                };
                CheckedValue {
                    value: values
                        .get(*index)
                        .ok_or("observation domain: Get index out of range")?
                        .clone(),
                    pure: tuple.pure,
                }
            }
            Expr::Bop(BinaryOp::Print, value, state) => {
                let value = child!(value);
                let state = child!(state);
                if !matches!(
                    value_type(&value.value)?,
                    Type::Base(BaseType::IntT | BaseType::BoolT)
                ) || state.value != Value::StateV
                {
                    return Err(
                        "observation domain: Print requires a scalar and StateV carrier".into(),
                    );
                }
                // No interpreter call, log, memory or detached effect evaluation.
                self.structural_print_visits += 1;
                CheckedValue {
                    value: Value::StateV,
                    pure: false,
                }
            }
            Expr::Bop(
                op @ (BinaryOp::Add
                | BinaryOp::Sub
                | BinaryOp::Mul
                | BinaryOp::LessThan
                | BinaryOp::And
                | BinaryOp::Or),
                left,
                right,
            ) => {
                let left = child!(left);
                let right = child!(right);
                if !left.pure || !right.pure {
                    return Err(
                        "observation domain: scalar operation depends on an effect/region carrier"
                            .into(),
                    );
                }
                let value = match (op, left.value, right.value) {
                    (
                        BinaryOp::Add,
                        Value::Const(Constant::Int(a)),
                        Value::Const(Constant::Int(b)),
                    ) => Value::Const(Constant::Int(a.wrapping_add(b))),
                    (
                        BinaryOp::Sub,
                        Value::Const(Constant::Int(a)),
                        Value::Const(Constant::Int(b)),
                    ) => Value::Const(Constant::Int(a.wrapping_sub(b))),
                    (
                        BinaryOp::Mul,
                        Value::Const(Constant::Int(a)),
                        Value::Const(Constant::Int(b)),
                    ) => Value::Const(Constant::Int(a.wrapping_mul(b))),
                    (
                        BinaryOp::LessThan,
                        Value::Const(Constant::Int(a)),
                        Value::Const(Constant::Int(b)),
                    ) => Value::Const(Constant::Bool(a < b)),
                    (
                        BinaryOp::And,
                        Value::Const(Constant::Bool(a)),
                        Value::Const(Constant::Bool(b)),
                    ) => Value::Const(Constant::Bool(a && b)),
                    (
                        BinaryOp::Or,
                        Value::Const(Constant::Bool(a)),
                        Value::Const(Constant::Bool(b)),
                    ) => Value::Const(Constant::Bool(a || b)),
                    _ => return Err("observation domain: binary operand type mismatch".into()),
                };
                CheckedValue { value, pure: true }
            }
            Expr::Uop(UnaryOp::Not, inner) => {
                let inner = child!(inner);
                let Value::Const(Constant::Bool(value)) = inner.value else {
                    return Err("observation domain: Not needs Bool".into());
                };
                if !inner.pure {
                    return Err("observation domain: Not depends on a region/effect carrier".into());
                }
                CheckedValue {
                    value: Value::Const(Constant::Bool(!value)),
                    pure: true,
                }
            }
            Expr::If(predicate, inputs, then, otherwise) => {
                let predicate_value = child!(predicate);
                let Value::Const(Constant::Bool(selected)) = predicate_value.value else {
                    return Err("observation domain: If predicate must be Bool".into());
                };
                if !predicate_value.pure {
                    return Err(
                        "observation domain: If predicate must be pure in its region".into(),
                    );
                }
                let region_input = child!(inputs);
                value_type(&region_input.value)?;
                self.skipped_regions.push(SkippedRegion {
                    parent: point,
                    branch: !selected,
                    observations: 0,
                });
                let selected_body = if selected { then } else { otherwise };
                let mut result = self.region(
                    selected_body,
                    region_input.value,
                    Assumption::InIf(selected, predicate.clone(), inputs.clone()),
                    input_index,
                    Some(point),
                    Some(selected),
                    base,
                )?;
                result.pure = false; // If is traversal, never a detached pure observation.
                result
            }
            _ => {
                return Err(format!(
                    "observation domain: unsupported reachable grammar: {}",
                    match expr.as_ref() {
                        Expr::Call(..) => "Call",
                        Expr::Alloc(..) => "Alloc",
                        Expr::DoWhile(..) => "DoWhile",
                        Expr::Switch(..) => "Switch",
                        Expr::Top(..) => "Top",
                        Expr::Bop(..) => "binary operator",
                        Expr::Uop(..) => "unary operator",
                        _ => "expression",
                    }
                ))
            }
        };
        if checked.pure {
            if let Value::Const(Constant::Bool(value)) = checked.value {
                // The public helper is used only after the whole pure fragment has
                // passed the positive grammar, region, context and type checks.
                let independently_evaluated = interpret_expr(expr, input);
                if independently_evaluated.value != checked.value
                    || !independently_evaluated.log.is_empty()
                    || !independently_evaluated.mem.is_empty()
                {
                    return Err(
                        "observation domain: pure interpreter/collector disagreement".into(),
                    );
                }
                self.observations.push(Observation {
                    expr: expr.clone(),
                    point,
                    value,
                });
            }
        }
        cache.insert(Rc::as_ptr(expr), checked.clone());
        Ok(checked)
    }
}

fn declarations(mode: ObservationMode) -> String {
    let annotation = if mode == ObservationMode::Injective {
        " :injective"
    } else {
        ""
    };
    format!(
        r#"
(sort ObservationValue)
(constructor ObservedBool (bool) ObservationValue{annotation})
(constructor EvalBoolAt (Expr i64) ObservationValue)
(relation ObservationSeed (i64 Expr i64 bool))
(relation ObservationConflict (i64 i64 i64 bool bool))
(ruleset observation-witness)
(rule ((ObservationSeed a e p true) (ObservationSeed b e p false))
      ((ObservationConflict a b p true false)) :ruleset observation-witness)
(ruleset observation-manual-audit)
(rule ((ObservationConflict a b p true false))
      ((panic "observation-manual-audit: conflicting feasible Bool observations"))
      :ruleset observation-manual-audit)
"#
    )
}

fn audit_schedule(mode: ObservationMode) -> &'static str {
    match mode {
        ObservationMode::Off | ObservationMode::CommonSeed => "",
        ObservationMode::Manual => "(run observation-manual-audit 1)",
        ObservationMode::Injective => "(run-schedule (saturate injectivity))",
    }
}

fn conflicts(egraph: &EGraph) -> Result<Vec<String>, egglog::Error> {
    let (rows, _, dag) = egraph.function_to_dag("ObservationConflict", usize::MAX, false)?;
    Ok(rows.into_iter().map(|row| dag.to_string(row)).collect())
}

pub(crate) struct PreparedObservations {
    pub declarations: String,
    pub initialization: String,
    config: ObservationConfig,
    directory: PathBuf,
    evidence: Option<ObservationEvidence>,
    pass: usize,
}

fn write_json(path: &Path, value: &serde_json::Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap())
        .unwrap_or_else(|error| panic!("observation receipt {}: {error}", path.display()));
}

impl PreparedObservations {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        program: &TreeProgram,
        fns: &[String],
        config: &ObservationConfig,
        pass: usize,
        evidence: Option<&ObservationEvidence>,
        tree: &mut TreeToEgglog,
        cache: &mut IndexMap<TermId, String>,
        printed: &mut String,
    ) -> Self {
        fs::create_dir_all(&config.evidence_dir).unwrap();
        let directory = config.evidence_dir.join(format!("pass-{pass:02}"));
        fs::create_dir(&directory).unwrap_or_else(|error| {
            panic!(
                "observation receipt directory must be fresh: {}: {error}",
                directory.display()
            )
        });
        if let Some(evidence) = evidence {
            assert_eq!(
                evidence.directory, config.evidence_dir,
                "prepared observation directory mismatch"
            );
            evidence.update(Some(pass), "collecting", None).unwrap();
        }
        let collected = collect(program, fns, &config.root_inputs, pass).unwrap_or_else(|error| {
            write_json(
                &directory.join("domain-error.json"),
                &json!({"stage":"before-seeding", "error":error, "emitted_observation_seeds":0}),
            );
            if let Some(evidence) = evidence {
                evidence
                    .update(Some(pass), "unsupported-domain", Some("unsupported"))
                    .unwrap();
            }
            panic!("{error}");
        });
        let mut initialization = String::new();
        let mut seeds = Vec::new();
        // Keep Rc addresses alive while TreeToEgglog's pointer cache is in use.
        let mut literal_nodes = Vec::new();
        for point in &collected.points {
            if !collected
                .observations
                .iter()
                .any(|o| o.point == point.token)
            {
                continue;
            }
            for value in [false, true] {
                let literal = Rc::new(Expr::Const(
                    Constant::Bool(value),
                    point.input_type.clone(),
                    point.context.clone(),
                ));
                let term = literal.to_egglog_with(tree);
                let var = print_with_intermediate_helper(&tree.termdag, term, cache, printed);
                literal_nodes.push(literal);
                let id = seeds.len();
                initialization.push_str(&format!("(ObservationSeed {id} {var} {} {value})\n(union (EvalBoolAt {var} {}) (ObservedBool {value}))\n", point.token, point.token));
                seeds.push(json!({"id":id, "expr":var, "term":tree.termdag.to_string(term), "point":point.token, "value":value, "origin":"contextual-literal"}));
            }
        }
        for observation in &collected.observations {
            let term = observation.expr.to_egglog_with(tree);
            let var = print_with_intermediate_helper(&tree.termdag, term, cache, printed);
            let id = seeds.len();
            initialization.push_str(&format!("(ObservationSeed {id} {var} {} {})\n(union (EvalBoolAt {var} {}) (ObservedBool {}))\n", observation.point, observation.value, observation.point, observation.value));
            seeds.push(json!({"id":id,"expr":var,"term":tree.termdag.to_string(term),"point":observation.point,"value":observation.value,"origin":"feasible-pure-expression"}));
        }
        let points: Vec<_> = collected.points.iter().map(|p| json!({"token":p.token,"input_index":p.input_index,"parent":p.parent,"branch":p.branch,"context":format!("{:?}",p.context),"input_type":format!("{:?}",p.input_type),"region_input":format!("{:?}",p.input)})).collect();
        let skipped: Vec<_> = collected.skipped_regions.iter().map(|p| json!({"parent":p.parent,"branch":p.branch,"observations":p.observations,"reason":"unselected-region-not-evaluated"})).collect();
        write_json(
            &directory.join("observations.json"),
            &json!({"pass":pass,"mode":format!("{:?}",config.mode),"points":points,"seeds":seeds,"skipped_regions":skipped,"supported_bool_observations":collected.observations.len(),"detached_pure_bool_evaluations":collected.observations.len(),"structural_print_visits":collected.structural_print_visits,"literal_injective":config.mode==ObservationMode::Injective,"eval_bool_at_injective":false,"expr_annotations":false}),
        );
        if let Some(evidence) = evidence {
            evidence.update(Some(pass), "collected", None).unwrap();
        }
        Self {
            declarations: declarations(config.mode),
            initialization,
            config: config.clone(),
            directory,
            evidence: evidence.cloned(),
            pass,
        }
    }

    pub(crate) fn initialize(
        &self,
        graph: &mut EGraph,
        initialization: &str,
        schedule: &str,
    ) -> Result<(), egglog::Error> {
        fs::write(self.directory.join("initialization.egg"), initialization).unwrap();
        fs::write(self.directory.join("source-schedule.egg"), schedule).unwrap();
        if let Err(error) = graph.parse_and_run_program(None, initialization) {
            write_json(
                &self.directory.join("initialization-error.json"),
                &json!({"stage":"initialization","error":error.to_string()}),
            );
            if let Some(evidence) = &self.evidence {
                evidence.update(Some(self.pass), "initialization-error", Some("error"))?;
            }
            return Err(error);
        }
        if let Some(evidence) = &self.evidence {
            evidence.update(Some(self.pass), "initialized", None)?;
        }

        Ok(())
    }

    pub(crate) fn run_source(
        &self,
        graph: &mut EGraph,
        schedule: &str,
    ) -> Result<(), egglog::Error> {
        if let Some(evidence) = &self.evidence {
            evidence.update(Some(self.pass), "source-schedule", None)?;
        }
        let result = graph.parse_and_run_program(None, schedule);
        if let Err(error) = &result {
            write_json(
                &self.directory.join("source-schedule-error.json"),
                &json!({"stage":"source-schedule-before-observer-audit","error":error.to_string(),"native_extraction_entered":false}),
            );
        }
        if let Some(evidence) = &self.evidence {
            evidence.update(
                Some(self.pass),
                if result.is_ok() {
                    "source-committed"
                } else {
                    "source-schedule-error"
                },
                result.is_err().then_some("error"),
            )?;
        }
        result.map(|_| ())
    }

    pub(crate) fn audit(&self, graph: &mut EGraph) -> Result<(), egglog::Error> {
        // Witness materialization is identical in all enabled modes. It stores
        // source seed IDs and primitive payloads before cancellation can abort.
        let witness_result = graph
            .parse_and_run_program(None, "(run observation-witness 1)")
            .and_then(|_| conflicts(graph));
        let witnesses = match witness_result {
            Ok(witnesses) => witnesses,
            Err(error) => {
                write_json(
                    &self.directory.join("witness-error.json"),
                    &json!({"stage":"observation-witness","error":error.to_string()}),
                );
                if let Some(evidence) = &self.evidence {
                    evidence.update(Some(self.pass), "observation-witness-error", Some("error"))?;
                }
                return Err(error);
            }
        };
        write_json(
            &self.directory.join("committed-before-audit.json"),
            &json!({"stage":"source-schedule-committed-before-serialization","conflicts":witnesses,"native_extraction_entered":false,"audit":audit_schedule(self.config.mode)}),
        );
        if let Some(evidence) = &self.evidence {
            evidence.update(Some(self.pass), "pre-audit", None)?;
        }
        let result = graph.parse_and_run_program(None, audit_schedule(self.config.mode));
        write_json(
            &self.directory.join("audit.json"),
            &json!({"stage":"pre-serialization-observation-audit","accepted":result.is_ok(),"mode":format!("{:?}",self.config.mode),"error":result.as_ref().err().map(ToString::to_string),"native_extraction_entered":false}),
        );
        if let Some(evidence) = &self.evidence {
            evidence.update(
                Some(self.pass),
                if result.is_ok() {
                    "audit-accepted"
                } else {
                    "observation-audit-rejected"
                },
                result.is_err().then_some("rejected"),
            )?;
        }
        result.map(|_| ())
    }

    pub(crate) fn serialization_entry(&self) -> Result<(), egglog::Error> {
        write_json(
            &self.directory.join("serialization-entry.json"),
            &json!({"stage":"observer-accepted-entering-serialization"}),
        );
        if let Some(evidence) = &self.evidence {
            evidence.update(Some(self.pass), "entering-serialization", None)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
