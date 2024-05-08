use std::{collections::HashMap, rc::Rc};

use crate::{
    from_egglog::FromEgglog,
    prologue,
    schema::{self, Assumption, BaseType, BinaryOp, Expr, RcExpr, TernaryOp, Type, UnaryOp},
};
use egglog::TermDag;

pub struct PrettyPrinter {
    pub expr: RcExpr,
}

impl PrettyPrinter {
    pub fn new(str_expr: String) -> std::result::Result<PrettyPrinter, egglog::Error> {
        let bounded_expr = format!("(let EXPR___ {})", str_expr);
        let prog = prologue().to_owned() + &bounded_expr;
        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(&prog).unwrap();
        let mut termdag = TermDag::default();
        let (sort, value) = egraph
            .eval_expr(&egglog::ast::Expr::Var((), "EXPR___".into()))
            .unwrap();
        let (_, extracted) = egraph.extract(value, &mut termdag, &sort);
        let mut converter = FromEgglog {
            termdag: &termdag,
            conversion_cache: HashMap::new(),
        };
        Ok(PrettyPrinter {
            expr: converter.expr_from_egglog(extracted),
        })
    }

    pub fn to_egglog_default(&self) -> String {
        self.to_egglog(&|rc, len| (rc > 1 && len > 80) || rc > 4 || len > 200)
    }

    pub fn to_egglog(&self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = String::new();
        let mut cache: HashMap<*const schema::Expr, String> = HashMap::new();
        let mut symbols: HashMap<String, String> = HashMap::new();
        let res = Self::to_egglog_helper(&self.expr, &mut cache, &mut symbols, &mut log, fold_when);
        log + &format!("(let EXPR___ {res})")
    }

    // turn the Expr to a rust ast macro string.
    // return a rust ast macro
    // fold_when: provide a function that decide when to fold the macro to a let binding
    pub fn to_rust(&self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = String::new();
        let mut cache: HashMap<*const schema::Expr, String> = HashMap::new();
        let mut symbols: HashMap<String, String> = HashMap::new();
        let res = Self::to_ast(&self.expr, &mut cache, &mut symbols, &mut log, fold_when);
        log + &format!("let expr___ = {res}; \n")
    }

    pub fn to_rust_default(&self) -> String {
        self.to_rust(&|rc, len| (rc > 1 && len > 30) || rc > 4 || len > 80)
    }

    // symbols: Type and Assumption's string -> their binding var
    fn to_egglog_helper(
        expr: &RcExpr,
        cache: &mut HashMap<*const schema::Expr, String>,
        symbols: &mut HashMap<String, String>,
        log: &mut String,
        fold_when: &dyn Fn(usize, usize) -> bool,
    ) -> String {
        use self::*;
        let find_or_insert = |var: String,
                              info: String,
                              str_builder: &mut String,
                              symbols: &mut HashMap<String, String>| {
            let fresh_var = format!("{}_{}", info, symbols.len());
            symbols
                .entry(var.clone())
                .or_insert_with(|| {
                    str_builder.push_str(&format!("(let {} \n   {}) \n", fresh_var.clone(), var));
                    fresh_var
                })
                .clone()
        };

        let fold_or_plain =
            |egglog_str: String,
             info: String,
             str_builder: &mut String,
             cache: &mut HashMap<*const schema::Expr, String>| {
                let rc = Rc::strong_count(expr);
                if fold_when(rc, egglog_str.len()) {
                    let fresh_var = format!("{}_{}", info, cache.len());
                    cache
                        .entry(Rc::as_ptr(expr))
                        .or_insert_with(|| {
                            str_builder.push_str(&format!(
                                "(let {} \n   {}) \n",
                                fresh_var.clone(),
                                egglog_str
                            ));
                            fresh_var
                        })
                        .clone()
                } else {
                    egglog_str
                }
            };
        match cache.get(&Rc::as_ptr(expr)) {
            Some(str) => str.to_owned(),
            None => {
                let expr = expr.as_ref();
                match expr {
                    Expr::Function(name, inty, outty, body) => {
                        let inty_str =
                            find_or_insert(inty.to_string(), inty.abbrev(), log, symbols);
                        let outty_str =
                            find_or_insert(outty.to_string(), outty.abbrev(), log, symbols);
                        let body = Self::to_egglog_helper(body, cache, symbols, log, fold_when);
                        let fun = format!("(Function {name} {inty_str} {outty_str} \n   {body})");
                        fold_or_plain(fun, format!("Fun_{name}"), log, cache)
                    }
                    Expr::Const(c, ty, assum) => {
                        let ty = find_or_insert(ty.to_string(), ty.abbrev(), log, symbols);
                        let assum = find_or_insert(assum.to_string(), assum.abbrev(), log, symbols);
                        let constant = format!("(Const {c} {ty} {assum})");
                        cache.insert(expr, constant.clone());
                        constant
                    }
                    Expr::Top(op, x, y, z) => {
                        let left = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let mid = Self::to_egglog_helper(y, cache, symbols, log, fold_when);
                        let right = Self::to_egglog_helper(z, cache, symbols, log, fold_when);
                        let top =
                            format!("(Top ({:?}) \n   {} \n   {} \n   {})", op, left, mid, right);
                        fold_or_plain(top, format!("{:?}", op), log, cache)
                    }
                    Expr::Bop(op, x, y) => {
                        let left = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let right = Self::to_egglog_helper(y, cache, symbols, log, fold_when);
                        let bop = format!("(Bop ({:?}) \n   {} \n   {})", op, left, right);

                        fold_or_plain(bop, format!("{:?}", op), log, cache)
                    }
                    Expr::Uop(op, x) => {
                        let sub_expr = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let uop = format!("(Uop ({:?}) {})", op, sub_expr);

                        fold_or_plain(uop, format!("{:?}", op), log, cache)
                    }
                    Expr::Get(x, pos) => {
                        let sub_expr = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let get = format!("(Get {sub_expr} {pos})");
                        cache.insert(expr, get.clone());
                        get
                    }
                    Expr::Alloc(id, x, y, pointer_ty) => {
                        let amount = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let state_edge = Self::to_egglog_helper(y, cache, symbols, log, fold_when);
                        let ty = find_or_insert(
                            pointer_ty.to_string(),
                            pointer_ty.abbrev(),
                            log,
                            symbols,
                        );
                        let alloc =
                            format!("(Alloc {id} \n    {amount} \n    {state_edge} \n    {ty})");
                        fold_or_plain(alloc, format!("Alloc{id}"), log, cache)
                    }
                    Expr::Call(name, x) => {
                        let sub_expr = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let call = format!("(Call {name} {sub_expr})");
                        fold_or_plain(call, format!("CallFun_{name}"), log, cache)
                    }
                    Expr::Empty(ty, assum) => {
                        let ty = find_or_insert(ty.to_string(), ty.abbrev(), log, symbols);
                        let assum = find_or_insert(assum.to_string(), assum.abbrev(), log, symbols);
                        let empty = format!("(Empty {ty} {assum})");
                        cache.insert(expr, empty.clone());
                        empty
                    }
                    // doesn't fold Tuple
                    Expr::Single(x) => {
                        let sub_expr = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let single = format!("(Single {})", sub_expr.clone());
                        cache.insert(expr, single.clone());
                        single
                    }
                    Expr::Concat(x, y) => {
                        let left = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let right = Self::to_egglog_helper(y, cache, symbols, log, fold_when);
                        let concat = format!("(Concat {left} {right})");
                        cache.insert(expr, concat.clone());
                        concat
                    }
                    Expr::Switch(x, inputs, _branches) => {
                        let cond = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let inputs = Self::to_egglog_helper(inputs, cache, symbols, log, fold_when);

                        fn cons_list(vec: Vec<String>) -> String {
                            match vec.get(0) {
                                Some(str) => {
                                    format!("(Cons {} {})", str, cons_list(vec[1..].to_vec()))
                                }
                                None => "(Nil)".to_string(),
                            }
                        }
                        let branches = _branches
                            .iter()
                            .map(|branch| {
                                Self::to_egglog_helper(branch, cache, symbols, log, fold_when)
                            })
                            .collect::<Vec<_>>();
                        let branch_list = cons_list(branches);
                        let switch = format!("(Switch \n   {cond}\n   {inputs}\n   {branch_list})");
                        fold_or_plain(switch, "switch".into(), log, cache)
                    }
                    Expr::If(x, inputs, y, z) => {
                        let pred = Self::to_egglog_helper(x, cache, symbols, log, fold_when);
                        let inputs = Self::to_egglog_helper(inputs, cache, symbols, log, fold_when);
                        let left = Self::to_egglog_helper(y, cache, symbols, log, fold_when);
                        let right = Self::to_egglog_helper(z, cache, symbols, log, fold_when);
                        let if_expr =
                            format!("(If \n   {pred}\n    {inputs}\n    {left}\n    {right})");
                        fold_or_plain(if_expr, "if".into(), log, cache)
                    }
                    Expr::DoWhile(inputs, body) => {
                        let inputs = Self::to_egglog_helper(inputs, cache, symbols, log, fold_when);
                        let body = Self::to_egglog_helper(body, cache, symbols, log, fold_when);
                        let dowhile = format!("(DoWhile\n   {inputs}\n   {body})");
                        fold_or_plain(dowhile, "dowhile".into(), log, cache)
                    }
                    Expr::Arg(ty, assum) => {
                        let ty = find_or_insert(ty.to_string(), ty.abbrev(), log, symbols);
                        let assum = find_or_insert(assum.to_string(), assum.abbrev(), log, symbols);
                        let arg = format!("(Arg {ty} {assum})");
                        cache.insert(expr, arg.clone());
                        arg
                    }
                }
            }
        }
    }

    fn concat_helper(
        expr: &RcExpr,
        cache: &mut HashMap<*const schema::Expr, String>,
        symbols: &mut HashMap<String, String>,
        log: &mut String,
        fold_when: &dyn Fn(usize, usize) -> bool,
    ) -> Vec<String> {
        match expr.as_ref() {
            Expr::Concat(lhs, rhs) => {
                let mut lhs = Self::concat_helper(lhs, cache, symbols, log, fold_when);
                let mut rhs = Self::concat_helper(rhs, cache, symbols, log, fold_when);
                lhs.append(&mut rhs);
                lhs
            }
            Expr::Single(expr) => {
                let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
                vec![expr]
            }
            _ => panic!("not well formed Concat, expr not wrapped with Single"),
        }
    }

    fn to_ast(
        expr: &RcExpr,
        cache: &mut HashMap<*const schema::Expr, String>,
        symbols: &mut HashMap<String, String>,
        log: &mut String,
        fold_when: &dyn Fn(usize, usize) -> bool,
    ) -> String {
        let find_or_insert = |var: String,
                              info: String,
                              str_builder: &mut String,
                              symbols: &mut HashMap<String, String>| {
            let fresh_var = format!("{}_{}", info, symbols.len());
            symbols
                .entry(var.clone())
                .or_insert_with(|| {
                    str_builder.push_str(&format!("let {} = {}; \n", fresh_var.clone(), var));
                    fresh_var
                })
                .clone()
        };

        let fold_or_plain =
            |ast_str: String,
             info: String,
             str_builder: &mut String,
             cache: &mut HashMap<*const schema::Expr, String>| {
                let rc = Rc::strong_count(expr);
                if fold_when(rc, ast_str.len()) {
                    let fresh_var = format!("{}_{}", info, cache.len());
                    let lookup = cache
                        .entry(Rc::as_ptr(expr))
                        .or_insert_with(|| {
                            str_builder.push_str(&format!(
                                "let {} = {}; \n",
                                fresh_var.clone(),
                                ast_str
                            ));
                            fresh_var
                        })
                        .clone();
                    format!("{lookup}.clone()")
                } else {
                    ast_str
                }
            };

        match cache.get(&Rc::as_ptr(expr)) {
            Some(str) => format!("{}.clone()", str),
            None => {
                match expr.as_ref() {
                    // just don't fold simple things like expr, getat anyway
                    Expr::Const(c, _, _) => match c {
                        schema::Constant::Bool(true) => "ttrue()".into(),
                        schema::Constant::Bool(false) => "tfalse()".into(),
                        schema::Constant::Int(n) => format!("int({})", n),
                    },
                    Expr::Bop(op, lhs, rhs) => {
                        let left = Self::to_ast(lhs, cache, symbols, log, fold_when);
                        let right = Self::to_ast(rhs, cache, symbols, log, fold_when);
                        let ast_str = format!("{}({}, {})", op.to_ast(), left, right);
                        fold_or_plain(ast_str, op.to_ast(), log, cache)
                    }
                    Expr::Top(op, x, y, z) => {
                        let left = Self::to_ast(x, cache, symbols, log, fold_when);
                        let mid = Self::to_ast(y, cache, symbols, log, fold_when);
                        let right = Self::to_ast(z, cache, symbols, log, fold_when);
                        let ast_str = format!("{}({}, {}, {})", op.to_ast(), left, mid, right);
                        fold_or_plain(ast_str, op.to_ast(), log, cache)
                    }
                    Expr::Uop(op, expr) => {
                        let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
                        let ast_str = format!("{}({})", op.to_ast(), expr);
                        fold_or_plain(ast_str, op.to_ast(), log, cache)
                    }
                    Expr::Get(expr, index) => match expr.as_ref() {
                        Expr::Arg(_, _) => {
                            format!("getat({index})")
                        }
                        _ => {
                            let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
                            format!("get({expr}, {index})")
                        }
                    },
                    Expr::Alloc(id, expr, state, ty) => {
                        let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
                        let state = Self::to_ast(state, cache, symbols, log, &fold_when);
                        let ty_str = ty.to_ast();
                        let ty_binding = find_or_insert(ty_str, ty.abbrev(), log, symbols);
                        let ast_str = format!("alloc({id}, {expr}, {state}, {ty_binding})");
                        fold_or_plain(ast_str, "alloc".into(), log, cache)
                    }
                    Expr::Call(name, arg) => {
                        let arg = Self::to_ast(arg, cache, symbols, log, fold_when);
                        format!("call({name}, {arg})")
                    }
                    Expr::Empty(..) => "empty()".into(),
                    Expr::Single(expr) => {
                        let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
                        format!("single({expr})")
                    }
                    Expr::Concat(..) => {
                        let vec = Self::concat_helper(expr, cache, symbols, log, fold_when);
                        let inside = vec.join(", ");
                        format!("parallel!({inside})")
                    }
                    Expr::Switch(cond, inputs, cases) => {
                        let cond = Self::to_ast(cond, cache, symbols, log, fold_when);
                        let inputs = Self::to_ast(inputs, cache, symbols, log, fold_when);
                        let cases = cases
                            .iter()
                            .map(|expr| Self::to_ast(expr, cache, symbols, log, fold_when))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let ast_str = format!("switch!({cond}, {inputs}; parallel!({cases}))");
                        fold_or_plain(ast_str, "switch".into(), log, cache)
                    }
                    Expr::If(cond, input, then, els) => {
                        let cond = Self::to_ast(cond, cache, symbols, log, fold_when);
                        let input = Self::to_ast(input, cache, symbols, log, fold_when);
                        let then = Self::to_ast(then, cache, symbols, log, fold_when);
                        let els = Self::to_ast(els, cache, symbols, log, fold_when);
                        let ast_str = format!("tif({cond}, {input}, {then}, {els})");
                        fold_or_plain(ast_str, "if".into(), log, cache)
                    }
                    Expr::DoWhile(input, body) => {
                        let input = Self::to_ast(input, cache, symbols, log, fold_when);
                        let body = Self::to_ast(body, cache, symbols, log, fold_when);
                        let ast_str = format!("dowhile({input}, {body})");
                        fold_or_plain(ast_str, "dowhile".into(), log, cache)
                    }
                    Expr::Arg(..) => "arg()".into(),
                    Expr::Function(name, ty_in, ty_out, body) => {
                        let ty_in_str = ty_in.to_ast();
                        let ty_in_binding = find_or_insert(ty_in_str, ty_in.abbrev(), log, symbols);
                        let ty_out_str = ty_out.to_ast();
                        let ty_out_binding =
                            find_or_insert(ty_out_str, ty_out.abbrev(), log, symbols);
                        let body = Self::to_ast(body, cache, symbols, log, fold_when);
                        format!("function(\"{name}\", {ty_in_binding}, {ty_out_binding}, {body})")
                    }
                }
            }
        }
    }
}

impl Expr {
    pub fn abbrev(&self) -> String {
        format!("{:?}", self)
    }
}

impl Assumption {
    pub fn to_ast(
        &self,
        cache: &mut HashMap<*const schema::Expr, String>,
        symbols: &mut HashMap<String, String>,
        log: &mut String,
        fold_when: &dyn Fn(usize, usize) -> bool,
    ) -> String {
        match self {
            Assumption::InFunc(fun_name) => {
                format!("infunc(\"{fun_name}\".into())")
            }
            Assumption::InIf(is, pred, input) => {
                format!(
                    "inif({is}, {}, {})",
                    PrettyPrinter::to_ast(pred, cache, symbols, log, fold_when),
                    PrettyPrinter::to_ast(input, cache, symbols, log, fold_when)
                )
            }
            Assumption::InLoop(input, output) => {
                format!(
                    "inloop({}, {})",
                    PrettyPrinter::to_ast(input, cache, symbols, log, fold_when),
                    PrettyPrinter::to_ast(output, cache, symbols, log, fold_when)
                )
            }
            Assumption::InSwitch(is, pred, inputs) => {
                format!(
                    "inswitch({is}, {}, {})",
                    PrettyPrinter::to_ast(pred, cache, symbols, log, fold_when),
                    PrettyPrinter::to_ast(inputs, cache, symbols, log, fold_when)
                )
            }
            Assumption::WildCard(_) => panic!("found wildcard"),
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
    pub fn to_ast(&self) -> String {
        match self {
            BaseType::IntT => "intt()".into(),
            BaseType::BoolT => "boolt()".into(),
            BaseType::StateT => "statet()".into(),
            BaseType::PointerT(ptr) => format!("pointert({})", BaseType::to_ast(ptr)),
        }
    }

    pub fn abbrev(&self) -> String {
        match self {
            BaseType::IntT => "i".into(),
            BaseType::BoolT => "b".into(),
            BaseType::StateT => "s".into(),
            BaseType::PointerT(ptr) => format!("ptr{}", &ptr.abbrev()),
        }
    }
}

impl Type {
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
        }
    }
}

impl BinaryOp {
    pub fn to_ast(&self) -> String {
        use schema::BinaryOp::*;
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
        }
        .into()
    }
}

impl TernaryOp {
    pub fn to_ast(&self) -> String {
        use schema::TernaryOp::Write;
        match self {
            Write => "twrite".into(),
        }
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
fn test_pretty_print() {
    use crate::ast::*;
    let output_ty = tuplet!(intt(), intt(), intt(), intt(), statet());
    let inner_inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), base(intt()));
    let inv = add(inner_inv.clone(), int(0)).with_arg_types(output_ty.clone(), base(intt()));
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), base(boolt()));
    let not_inv = add(getat(0), inv.clone()).with_arg_types(output_ty.clone(), base(intt()));
    let inv_in_print = add(inv.clone(), int_ty(4, output_ty.clone()));
    let print =
        tprint(inv_in_print.clone(), getat(4)).with_arg_types(output_ty.clone(), base(statet()));

    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4), getat(0)),
        concat(
            parallel!(pred.clone(), not_inv.clone(), getat(1)),
            concat(parallel!(getat(2), getat(3)), single(print.clone())),
        ),
    )
    .with_arg_types(tuplet!(statet()), output_ty.clone())
    .add_ctx(schema::Assumption::dummy());

    let expr_str = my_loop.to_string();

    PrettyPrinter::new(expr_str.clone())
        .unwrap()
        .to_rust_default();
}
