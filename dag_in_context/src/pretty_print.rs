use crate::{
    from_egglog::FromEgglog,
    prologue,
    schema::{
        self, Assumption, BaseType, BinaryOp, Expr, RcExpr, TernaryOp, TreeProgram, Type, UnaryOp,
    },
    schema_helpers::AssumptionRef,
    to_egglog::TreeToEgglog,
};
use egglog::{Term, TermDag};

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    rc::Rc,
    vec,
};

#[derive(Default)]
pub struct PrettyPrinter {
    // Type/Assum/BaseType -> intermediate variables
    symbols: HashMap<NodeRef, String>,
    // intermediate variable -> Type/Assum/BaseType lookup
    table: BTreeMap<String, AstNode>,
    fresh_count: u64,
}

#[derive(PartialEq, Eq, Hash)]
enum NodeRef {
    Type(schema::Type),
    Assumption(AssumptionRef),
    Expr(*const schema::Expr),
}

#[derive(PartialEq, Eq)]
enum AstNode {
    Type(schema::Type),
    Assumption(schema::Assumption),
    Expr(schema::Expr),
}

impl AstNode {
    pub(crate) fn ast_node_to_str(&self, to_rust: bool) -> String {
        match self {
            AstNode::Assumption(assum) => {
                if to_rust {
                    assum.to_ast()
                } else {
                    assum.pretty()
                }
            }
            AstNode::Type(ty) => {
                if to_rust {
                    ty.to_ast()
                } else {
                    ty.pretty()
                }
            }
            AstNode::Expr(expr) => {
                if to_rust {
                    expr.to_ast()
                } else {
                    expr.pretty()
                }
            }
        }
    }
}

impl TreeProgram {
    pub fn pretty_print_to_egglog(&self) -> String {
        let mut pp = PrettyPrinter::default();
        let (main_log, main_binding) = pp.to_egglog_default(&self.entry);
        let mut function_bindings = vec![];
        let functions = self
            .functions
            .clone()
            .into_iter()
            .map(|expr| {
                let (log, binding) = pp.to_egglog_default(&expr);
                function_bindings.push(binding.clone());
                log
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        let function_list = function_bindings
            .into_iter()
            .rev()
            .fold("(Nil)".to_string(), |acc, binding| {
                format!("(Cons {binding} {acc})")
            });
        format!(
            "{main_log}\n {functions} \n (let PROG_PP (Program {main_binding} {function_list}))"
        )
    }

    pub fn pretty_print_to_rust(&self) -> String {
        let mut pp = PrettyPrinter::default();
        let (log, _) = pp.to_rust_default(&self.entry);
        std::iter::once(log)
            .chain(self.functions.clone().into_iter().map(|expr| {
                let (log, _) = pp.to_rust_default(&expr);
                log
            }))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl PrettyPrinter {
    pub fn from_string(
        str_expr: String,
        to_rust: bool,
    ) -> std::result::Result<(String, String), egglog::Error> {
        let mut pp = PrettyPrinter::default();
        let binding = pp.mk_fresh("EXPR".into());
        let bounded_expr = format!("(let {} {})", binding.clone(), str_expr);
        let prog = prologue().to_owned() + &bounded_expr;
        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(&prog).unwrap();
        let mut termdag = TermDag::default();
        let (sort, value) = egraph
            .eval_expr(&egglog::ast::Expr::Var((), binding.into()))
            .unwrap();
        let (_, extracted) = egraph.extract(value, &mut termdag, &sort);
        let mut converter = FromEgglog {
            termdag: &termdag,
            conversion_cache: HashMap::default(),
        };
        let expr = converter.expr_from_egglog(extracted);
        if to_rust {
            Ok(pp.to_rust_default(&expr))
        } else {
            Ok(pp.to_egglog_default(&expr))
        }
    }

    pub fn to_egglog_default(&mut self, expr: &RcExpr) -> (String, String) {
        self.to_egglog(expr, &|rc, len| (rc > 1 && len > 30) || len > 80)
    }

    // turn the Expr to a nested egglog with intermediate variables.
    // fold_when: function deciding when to fold the macro to a let binding
    // the function take reference count and length, and return a bool-type
    // expression of when to fold based on reference count and length
    // return a tuple of (log, binding to the passed in expr)
    pub fn to_egglog(
        &mut self,
        expr: &RcExpr,
        fold_when: &dyn Fn(usize, usize) -> bool,
    ) -> (String, String) {
        let mut log = vec![];
        let res = self.refactor_shared_expr(expr, fold_when, false, &mut log);
        let log = log
            .iter()
            .map(|fresh_var| {
                let node = self.table.get(fresh_var).unwrap();
                let pretty = node.ast_node_to_str(false);
                format!("(let {fresh_var} \n{pretty})\n")
            })
            .collect::<Vec<_>>()
            .join("");
        let binding = self.mk_fresh(expr.abbrev());
        (
            log + &format!("\n(let {} \n{})\n", binding.clone(), res.pretty()),
            binding,
        )
    }

    pub fn to_rust_default(&mut self, expr: &RcExpr) -> (String, String) {
        self.to_rust(expr, &|rc, len| (rc > 1 && len > 30) || rc > 4 || len > 80)
    }

    //  turn the Expr to a rust ast macro string.
    // fold_when: function deciding when to fold the macro to a let binding
    // the function take reference count and length, and return a bool-type
    // expression of when to fold based on reference count and length
    // return a tuple of (log, binding to the passed in expr)
    pub fn to_rust(
        &mut self,
        expr: &RcExpr,
        fold_when: &dyn Fn(usize, usize) -> bool,
    ) -> (String, String) {
        let mut log = vec![];
        let res = self.refactor_shared_expr(expr, fold_when, false, &mut log);
        let log = log
            .iter()
            .map(|fresh_var| {
                let node = self.table.get(fresh_var).unwrap();
                let ast = node.ast_node_to_str(true);
                format!("let {fresh_var} = {ast};")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let binding = self.mk_fresh(expr.abbrev());
        (
            log + &format!("\nlet {} = {};\n", binding.clone(), res.to_ast()),
            binding,
        )
    }

    fn mk_fresh(&mut self, info: String) -> String {
        let fresh_var = format!("{info}_v{}", self.fresh_count);
        self.fresh_count += 1;
        fresh_var
    }

    fn try_insert_fresh(&mut self, var: NodeRef, info: String) -> String {
        if self.symbols.get(&var).clone().is_none() {
            let fresh_var = &self.mk_fresh(info);
            self.symbols.insert(var, fresh_var.clone());
            fresh_var.to_owned()
        } else {
            self.symbols.get(&var).unwrap().into()
        }
    }

    fn refactor_shared_assum(
        &mut self,
        assum: &Assumption,
        fold_when: &dyn Fn(usize, usize) -> bool,
        to_rust: bool,
        log: &mut Vec<String>,
    ) -> Assumption {
        let assum_ref = NodeRef::Assumption(assum.to_ref());
        if !self.symbols.contains_key(&assum_ref) {
            let new_assum = match assum {
                Assumption::InFunc(_) => assum.clone(),
                Assumption::InIf(cond, left, right) => {
                    let left = self.refactor_shared_expr(left, fold_when, to_rust, log);
                    let right = self.refactor_shared_expr(right, fold_when, to_rust, log);
                    Assumption::InIf(*cond, Rc::new(left), Rc::new(right))
                }
                Assumption::InLoop(inputs, body) => {
                    let inputs = self.refactor_shared_expr(inputs, fold_when, to_rust, log);
                    let body = self.refactor_shared_expr(body, fold_when, to_rust, log);
                    Assumption::InLoop(Rc::new(inputs), Rc::new(body))
                }
                Assumption::InSwitch(cond, inputs, branch) => {
                    let inputs = self.refactor_shared_expr(inputs, fold_when, to_rust, log);
                    let branch = self.refactor_shared_expr(branch, fold_when, to_rust, log);
                    Assumption::InSwitch(*cond, Rc::new(inputs), Rc::new(branch))
                }
                Assumption::WildCard(_) => assum.clone(),
            };
            let binding = self.try_insert_fresh(assum_ref, assum.abbrev());
            log.push(binding.clone());
            self.table
                .insert(binding.clone(), AstNode::Assumption(new_assum));
            Assumption::WildCard(binding)
        } else {
            Assumption::WildCard(self.symbols.get(&assum_ref).unwrap().into())
        }
    }

    fn refactor_shared_type(&mut self, ty: &Type, log: &mut Vec<String>) -> Type {
        let ty_node = NodeRef::Type(ty.clone());
        let ty_binding = self.try_insert_fresh(ty_node, ty.abbrev());
        if !self.table.contains_key(&ty_binding) {
            log.push(ty_binding.clone());
            self.table
                .insert(ty_binding.clone(), AstNode::Type(ty.clone()));
        }
        Type::Symbolic(ty_binding)
    }

    fn refactor_shared_expr(
        &mut self,
        expr: &RcExpr,
        fold_when: &dyn Fn(usize, usize) -> bool,
        to_rust: bool,
        log: &mut Vec<String>,
    ) -> Expr {
        let old_expr_addr = Rc::as_ptr(expr);
        let fold = |pp: &mut PrettyPrinter, new_expr: schema::Expr, log: &mut Vec<String>| {
            let binding = pp.try_insert_fresh(NodeRef::Expr(old_expr_addr), expr.abbrev());
            if !pp.table.contains_key(&binding) {
                log.push(binding.clone());
                pp.table.insert(binding.clone(), AstNode::Expr(new_expr));
            }
            Expr::Symbolic(binding)
        };

        let num_shared = Rc::strong_count(expr);
        let fold_or_plain = |pp: &mut PrettyPrinter, new_expr: Expr, log: &mut Vec<String>| {
            let size = &new_expr
                .to_string()
                .replace(&['(', ')', ' ', '\n', ','][..], "") //don't count those char when computing size
                .len();
            if fold_when(num_shared, *size) {
                fold(pp, new_expr, log)
            } else {
                new_expr
            }
        };

        let types = expr
            .as_ref()
            .map_types(|ty| self.refactor_shared_type(ty, log));
        let assum = expr
            .as_ref()
            .map_assumptions(|assum| self.refactor_shared_assum(assum, fold_when, to_rust, log));
        let children = expr.map_children(|e| self.refactor_shared_expr(e, fold_when, to_rust, log));

        match expr.as_ref() {
            Expr::Function(name, ..) => Expr::Function(
                name.into(),
                types[0].clone(),
                types[1].clone(),
                Rc::new(children[0].clone()),
            ),
            Expr::Const(c, ..) => {
                let c = Expr::Const(c.clone(), types[0].clone(), assum);
                if to_rust {
                    c
                } else {
                    fold(self, c, log)
                }
            }
            Expr::Top(op, ..) => {
                let top = Expr::Top(
                    op.clone(),
                    Rc::new(children[0].clone()),
                    Rc::new(children[1].clone()),
                    Rc::new(children[2].clone()),
                );
                fold_or_plain(self, top, log)
            }
            Expr::Bop(op, ..) => {
                let bop = Expr::Bop(
                    op.clone(),
                    Rc::new(children[0].clone()),
                    Rc::new(children[1].clone()),
                );
                fold_or_plain(self, bop, log)
            }
            Expr::Uop(op, _) => {
                let uop = Expr::Uop(op.clone(), Rc::new(children[0].clone()));
                fold_or_plain(self, uop, log)
            }
            Expr::Get(_, pos) => {
                let get = Expr::Get(Rc::new(children[0].clone()), *pos);
                // fold Get Arg i anyway
                if let Expr::Arg(..) = expr.as_ref() {
                    if !to_rust {
                        return fold(self, get, log);
                    }
                }
                get
            }
            Expr::Alloc(id, _, _, ty) => {
                let alloc = Expr::Alloc(
                    *id,
                    Rc::new(children[0].clone()),
                    Rc::new(children[1].clone()),
                    ty.clone(),
                );
                fold_or_plain(self, alloc, log)
            }
            Expr::Call(name, ..) => {
                let call = Expr::Call(name.into(), Rc::new(children[0].clone()));
                fold_or_plain(self, call, log)
            }
            Expr::Empty(..) => Expr::Empty(types[0].clone(), assum),
            // doesn't fold Tuple
            Expr::Single(..) => Expr::Single(Rc::new(children[0].clone())),
            Expr::Concat(..) => {
                Expr::Concat(Rc::new(children[0].clone()), Rc::new(children[1].clone()))
            }
            Expr::Switch(..) => {
                let len = children.len();
                let branches = children[2..len]
                    .iter()
                    .map(|branch| Rc::new(branch.clone()))
                    .collect::<Vec<_>>();
                let switch = Expr::Switch(
                    Rc::new(children[0].clone()),
                    Rc::new(children[1].clone()),
                    branches,
                );
                fold_or_plain(self, switch, log)
            }
            Expr::If(..) => {
                let if_expr = Expr::If(
                    Rc::new(children[0].clone()),
                    Rc::new(children[1].clone()),
                    Rc::new(children[2].clone()),
                    Rc::new(children[3].clone()),
                );
                fold_or_plain(self, if_expr, log)
            }
            Expr::DoWhile(..) => {
                let dowhile =
                    Expr::DoWhile(Rc::new(children[0].clone()), Rc::new(children[1].clone()));
                fold_or_plain(self, dowhile, log)
            }
            Expr::Arg(_, _) => Expr::Arg(types[0].clone(), assum),
            Expr::Symbolic(_) => panic!("No symbolic should occur here"),
        }
    }
}

impl Expr {
    pub fn pretty(&self) -> String {
        let (term, termdag) = Rc::new(self.clone()).to_egglog();
        let expr = termdag.term_to_expr(&term);
        expr.to_sexp().pretty()
    }

    fn gather_concat_children(&self) -> Vec<String> {
        match self {
            Expr::Concat(lhs, rhs) => {
                let mut lhs = lhs.as_ref().gather_concat_children();
                let mut rhs = rhs.as_ref().gather_concat_children();
                lhs.append(&mut rhs);
                lhs
            }
            Expr::Single(expr) => {
                let expr = Self::to_ast(expr);
                vec![expr]
            }
            _ => panic!("should be in gather concat"),
        }
    }

    fn check_all_single(&self) -> bool {
        match self {
            Expr::Concat(lhs, rhs) => {
                let lhs = lhs.as_ref().check_all_single();
                let rhs = rhs.as_ref().check_all_single();
                lhs && rhs
            }
            Expr::Single(_) => true,
            _ => false,
        }
    }

    pub fn abbrev(&self) -> String {
        match self {
            Expr::Const(c, ..) => match c {
                schema::Constant::Int(i) => format!("int{i}"),
                schema::Constant::Bool(b) => format!("bool{b}"),
                schema::Constant::Float(f) => {
                    format!("float{}", std::ptr::addr_of!(f) as i64)
                }
            },
            Expr::Top(op, ..) => op.to_ast(),
            Expr::Bop(op, ..) => op.to_ast(),
            Expr::Uop(op, _) => op.to_ast(),
            Expr::Get(_, usize) => {
                format!("get_at_{usize}")
            }
            Expr::Alloc(id, ..) => "alloc".to_owned() + &format!("id{id}"),
            Expr::Call(name, _) => "call_".to_owned() + name,
            Expr::Empty(..) => "empty".into(),
            Expr::Single(_) => "single".into(),
            Expr::Concat(..) => "concat".into(),
            Expr::If(..) => "if".into(),
            Expr::Switch(..) => "switch".into(),
            Expr::DoWhile(..) => "dowhile".into(),
            Expr::Arg(..) => "arg".into(),
            Expr::Function(name, ..) => "fun_".to_owned() + name,
            Expr::Symbolic(var) => "symbolic_".to_owned() + var,
        }
    }

    pub fn to_ast(&self) -> String {
        use schema::Constant::*;
        let children = Rc::new(self.clone()).map_children(|expr| expr.to_ast());
        let types = self.map_types(|ty| ty.to_ast());
        match self {
            Expr::Const(c, ..) => match c {
                Bool(true) => "ttrue()".into(),
                Bool(false) => "tfalse()".into(),
                Int(n) => format!("int({})", n),
                Float(f) => format!("float({})", f),
            },
            Expr::Top(op, ..) => {
                format!(
                    "{}({}, \n{}, \n{})",
                    op.to_ast(),
                    children[0],
                    children[1],
                    children[2]
                )
            }
            Expr::Bop(op, ..) => {
                format!("{}({}, \n{})", op.to_ast(), children[0], children[1])
            }
            Expr::Uop(op, _) => {
                format!("{}({})", op.to_ast(), children[0])
            }
            Expr::Get(expr, index) => match expr.as_ref() {
                Expr::Arg(..) => format!("getat({index})"),
                _ => format!("get({}, {index})", children[0]),
            },
            Expr::Alloc(id, ..) => {
                format!(
                    "alloc({id}, {}, {}, {})",
                    children[0], children[1], types[0]
                )
            }
            Expr::Call(name, _) => {
                format!("call({name}, {})", children[0])
            }
            Expr::Empty(..) => "empty()".into(),
            Expr::Single(_) => {
                format!("single({})", children[0])
            }
            Expr::Concat(..) => {
                if self.check_all_single() {
                    let vec = Self::gather_concat_children(self);
                    let inside = vec.join(", ");
                    format!("parallel!({inside})")
                } else {
                    format!("concat({}, \n{})", children[0], children[1])
                }
            }
            Expr::If(..) => {
                format!(
                    "tif({}, \n{}, \n{}, \n{})",
                    children[0], children[1], children[2], children[3]
                )
            }
            Expr::Switch(..) => {
                let len = children.len();
                let cases = children[2..len].to_vec().join(", ");
                format!("switch!({}, {}; {})", children[0], children[1], cases)
            }
            Expr::DoWhile(..) => {
                format!("dowhile({}, \n{})", children[0], children[1])
            }
            Expr::Arg(..) => "arg()".into(),
            Expr::Function(name, ..) => {
                format!(
                    "function(\"{name}\", \n{}, \n{}, \n{})",
                    types[0], types[1], children[0]
                )
            }
            Expr::Symbolic(str) => format!("{str}.clone()"),
        }
    }
}

impl Assumption {
    pub fn pretty(&self) -> String {
        let (term, termdag) = self.to_egglog();
        let expr = termdag.term_to_expr(&term);
        expr.to_sexp().pretty()
    }

    pub fn to_ast(&self) -> String {
        match self {
            Assumption::InFunc(fun_name) => format!("infunc(\"{fun_name}\")"),
            Assumption::InIf(is, pred, input) => {
                format!("inif({is}, \n{}, \n{})", pred.to_ast(), input.to_ast())
            }
            Assumption::InLoop(input, output) => {
                format!("inloop({}, \n{})", input.to_ast(), output.to_ast())
            }
            Assumption::InSwitch(is, pred, inputs) => {
                format!("inswitch({is}, \n{}, \n{})", pred.to_ast(), inputs.to_ast())
            }
            Assumption::WildCard(str) => format!("{}.clone()", str),
        }
    }

    pub fn abbrev(&self) -> String {
        match self {
            Assumption::InFunc(_) => "in_func",
            Assumption::InIf(..) => "in_if",
            Assumption::InLoop(..) => "in_loop",
            Assumption::InSwitch(..) => "in_switch",
            Assumption::WildCard(_) => "wildcard",
        }
        .into()
    }
}

impl BaseType {
    pub(crate) fn to_egglog(&self) -> (Term, TermDag) {
        let mut state = TreeToEgglog::new();
        let term = self.to_egglog_internal(&mut state);
        (term, state.termdag)
    }

    pub fn pretty(&self) -> String {
        let (term, termdag) = self.to_egglog();
        let expr = termdag.term_to_expr(&term);
        expr.to_sexp().pretty()
    }

    pub fn to_ast(&self) -> String {
        match self {
            BaseType::IntT => "intt()".into(),
            BaseType::BoolT => "boolt()".into(),
            BaseType::StateT => "statet()".into(),
            BaseType::PointerT(ptr) => format!("pointert({})", BaseType::to_ast(ptr)),
            BaseType::FloatT => "floatt()".into(),
        }
    }

    pub fn abbrev(&self) -> String {
        match self {
            BaseType::IntT => "i".into(),
            BaseType::BoolT => "b".into(),
            BaseType::StateT => "s".into(),
            BaseType::PointerT(ptr) => format!("ptr{}", &ptr.abbrev()),
            BaseType::FloatT => "f".into(),
        }
    }
}

impl Type {
    pub fn pretty(&self) -> String {
        let (term, termdag) = self.to_egglog();
        let expr = termdag.term_to_expr(&term);
        expr.to_sexp().pretty()
    }

    pub fn to_ast(&self) -> String {
        match self {
            Type::Base(t) => format!("base({})", BaseType::to_ast(t)),
            Type::TupleT(vec_ty) => {
                if vec_ty.is_empty() {
                    return "emptyt()".into();
                }
                let vec_ty_str = vec_ty
                    .iter()
                    .map(BaseType::to_ast)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuplet!({vec_ty_str})")
            }
            Type::Unknown => panic!("found unknown in to_ast"),
            Type::Symbolic(str) => format!("{}.clone()", str),
        }
    }

    // give a abbreviated name of type, ex: a tuple of Int Int State become tpl_iis
    pub fn abbrev(&self) -> String {
        match self {
            Type::Base(base) => format!("base_{}", base.abbrev()),
            Type::TupleT(vec) => {
                let vec_ty_str = vec
                    .iter()
                    .map(|bt| bt.abbrev())
                    .collect::<Vec<_>>()
                    .join("_");
                format!("tpl_{}", vec_ty_str)
            }
            Type::Unknown => "unknown".into(),
            Type::Symbolic(str) => str.into(),
        }
    }
}

impl BinaryOp {
    pub fn to_ast(&self) -> String {
        use schema::BinaryOp::*;
        // the same as schema_helper's
        match self {
            Add => "add",
            Sub => "sub",
            Mul => "mul",
            Div => "div",
            Eq => "eq",
            LessThan => "less_than",
            GreaterThan => "greater_than",
            LessEq => "less_eq",
            GreaterEq => "greater_eq",
            And => "and",
            Or => "or",
            PtrAdd => "ptradd",
            Load => "load",
            Print => "tprint",
            Free => "free",
            FGreaterEq => "fgreater_eq",
            FGreaterThan => "fgreater_than",
            FLessEq => "fless_eq",
            FLessThan => "fless_than",
            FAdd => "fadd",
            FSub => "fsub",
            FDiv => "fdiv",
            FMul => "fmul",
            FEq => "feq",
        }
        .into()
    }
}

impl TernaryOp {
    pub fn to_ast(&self) -> String {
        match self {
            Self::Write => "twrite",
            Self::Select => "select",
        }
        .into()
    }
}

impl UnaryOp {
    pub fn to_ast(&self) -> String {
        use schema::UnaryOp::Not;
        match self {
            Not => "not".into(),
        }
    }
}

#[test]
fn test_pretty_print() -> crate::Result {
    use crate::ast::*;
    use crate::egglog_test;
    use crate::Value;
    use insta::assert_snapshot;
    let output_ty = tuplet!(intt(), intt(), intt(), intt(), statet());
    let inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), base(intt()));
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), base(boolt()));
    let print = tprint(inv, getat(4)).with_arg_types(output_ty.clone(), base(statet()));
    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4), getat(0)),
        concat(
            parallel!(pred.clone(), getat(0), getat(1)),
            concat(parallel!(getat(2), getat(3)), single(print.clone())),
        ),
    )
    .with_arg_types(tuplet!(statet()), output_ty.clone())
    .add_ctx(schema::Assumption::dummy());

    let pureloop = dowhile(
        single(int(1)),
        parallel!(
            less_than(get(arg(), 0), int(3)),
            get(switch!(int(0), arg(); parallel!(int(4), int(5))), 0)
        ),
    )
    .with_arg_types(emptyt(), tuplet!(intt()))
    .add_ctx(schema::Assumption::dummy());

    let concat_loop = concat(my_loop, pureloop);
    let expr_str = concat_loop.to_string();
    let (egglog, binding) = PrettyPrinter::default().to_egglog_default(&concat_loop);
    let (ast, _) = PrettyPrinter::default().to_rust_default(&concat_loop);
    assert_snapshot!(ast);
    let check = format!("(let unfold {expr_str})\n {egglog} \n(check (= {binding} unfold))\n");

    egglog_test(
        "",
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
