use crate::{
    from_egglog::FromEgglog,
    prologue,
    schema::{self, Assumption, BaseType, BinaryOp, Expr, RcExpr, TernaryOp, Type, UnaryOp},
    to_egglog::TreeToEgglog,
};
use egglog::{Term, TermDag};
use indexmap::IndexMap;
use std::{collections::HashMap, rc::Rc};

pub struct PrettyPrinter {
    pub expr: RcExpr,
    pub cache: indexmap::IndexMap<*const schema::Expr, String>,
    // String of Type/Assumption/BaseType -> fresh variable binding
    // no good way of make this polymorphic..
    pub symbols: indexmap::IndexMap<String, String>,
}

impl PrettyPrinter {
    pub fn from_string(str_expr: String) -> std::result::Result<PrettyPrinter, egglog::Error> {
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
        Ok(Self::from_expr(converter.expr_from_egglog(extracted)))
    }

    pub fn from_expr(expr: RcExpr) -> PrettyPrinter {
        let cache = indexmap::IndexMap::new();
        let symbols = indexmap::IndexMap::new();
        PrettyPrinter {
            expr,
            cache,
            symbols,
        }
    }

    pub fn to_egglog_default(&mut self) -> String {
        self.to_egglog(&|rc, len| (rc > 1 && len > 30) || len > 80)
    }

    pub fn to_egglog(&mut self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = indexmap::IndexMap::new();
        Self::assign_fresh_var(&self.expr, &mut self.cache, &mut self.symbols, false);
        let res = self.to_nested_expr(&self.expr, &mut log, fold_when, false);
        let log = self
            .symbols
            .iter()
            .map(|(expr, symbol)| format!("(let {symbol}\n{expr}) \n"))
            .chain(
                log.iter()
                    .map(|(var, expr)| format!("(let {var} \n{}) \n", expr.pretty()))
                    .collect::<Vec<_>>(),
            )
            .collect::<Vec<_>>()
            .join("");
        log + &format!("(let EXPR___\n{})", res.pretty())
    }

    pub fn to_rust_default(&mut self) -> String {
        self.to_rust(&|rc, len| (rc > 1 && len > 30) || rc > 4 || len > 80)
    }

    // turn the Expr to a rust ast macro string.
    // return a rust ast macro
    // fold_when: provide a function that decide when to fold the macro to a let binding
    pub fn to_rust(&mut self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = indexmap::IndexMap::new();
        Self::assign_fresh_var(&self.expr, &mut self.cache, &mut self.symbols, true);
        let res = self.to_nested_expr(&self.expr, &mut log, fold_when, true);
        let log = self
            .symbols
            .iter()
            .map(|(expr, symbol)| format!("let {symbol} = {expr}; \n"))
            .chain(
                log.iter()
                    .map(|(var, expr)| format!("let {var} = {}; \n", expr.to_ast()))
                    .collect::<Vec<_>>(),
            )
            .collect::<Vec<_>>()
            .join("");
        log + &format!("let expr___ = {};", res.to_ast())
    }

    fn assign_fresh_var(
        expr: &RcExpr,
        cache: &mut IndexMap<*const schema::Expr, String>,
        symbols: &mut IndexMap<String, String>,
        to_rust: bool,
    ) {
        let len = cache.len();
        let make_fresh = |info: String| format!("{info}_{}", len);

        fn try_insert_fresh(var: String, info: String, symbols: &mut IndexMap<String, String>) {
            if !symbols.contains_key(&var) {
                let fresh_var = format!("{info}_{}", symbols.len());
                symbols.insert(var, fresh_var);
            }
        }

        let expr_ptr = Rc::as_ptr(expr);

        // some expr need fresh var, other do not
        if !cache.contains_key(&expr_ptr) {
            match expr.as_ref() {
                Expr::Const(c, ty, assum) => {
                    // try_insert_fresh(ty.to_owned(), ty.abbrev(), types);
                    // try_insert_fresh( Rc::as_ptr(&Rc::new(assum.to_owned())) , assum.abbrev(), assums);
                    if to_rust {
                        try_insert_fresh(ty.to_ast(), ty.abbrev(), symbols);
                        try_insert_fresh(assum.to_ast(), assum.abbrev(), symbols);
                    } else {
                        try_insert_fresh(ty.pretty(), ty.abbrev(), symbols);
                        try_insert_fresh(assum.pretty(), assum.abbrev(), symbols);
                    }
                    let c = match c {
                        schema::Constant::Int(i) => format!("int{i}"),
                        schema::Constant::Bool(b) => format!("bool{b}"),
                    };
                    cache.insert(expr_ptr, make_fresh(c));
                }
                Expr::Top(op, lhs, mid, rhs) => {
                    Self::assign_fresh_var(lhs, cache, symbols, to_rust);
                    Self::assign_fresh_var(mid, cache, symbols, to_rust);
                    Self::assign_fresh_var(rhs, cache, symbols, to_rust);
                    cache.insert(expr_ptr, make_fresh(op.to_ast()));
                }
                Expr::Bop(op, lhs, rhs) => {
                    Self::assign_fresh_var(lhs, cache, symbols, to_rust);
                    Self::assign_fresh_var(rhs, cache, symbols, to_rust);
                    cache.insert(expr_ptr, make_fresh(op.to_ast()));
                }
                Expr::Uop(op, expr) => {
                    Self::assign_fresh_var(expr, cache, symbols, to_rust);
                    cache.insert(expr_ptr, make_fresh(op.to_ast()));
                }
                Expr::Get(expr, usize) => {
                    if let Expr::Arg(..) = expr.as_ref() {
                        cache.insert(expr_ptr, make_fresh(format!("get_at_{usize}")));
                    }
                    Self::assign_fresh_var(expr, cache, symbols, to_rust);
                }
                Expr::Alloc(id, x, y, ty) => {
                    Self::assign_fresh_var(x, cache, symbols, to_rust);
                    Self::assign_fresh_var(y, cache, symbols, to_rust);
                    if to_rust {
                        try_insert_fresh(ty.to_ast(), ty.abbrev(), symbols);
                    } else {
                        try_insert_fresh(ty.pretty(), ty.abbrev(), symbols);
                    }
                    cache.insert(expr_ptr, expr.as_ref().abbrev() + &id.to_string());
                }
                Expr::Call(name, arg) => {
                    Self::assign_fresh_var(arg, cache, symbols, to_rust);
                    cache.insert(expr_ptr, make_fresh("call_".to_owned() + name));
                }
                Expr::Empty(ty, assum) => {
                    if to_rust {
                        try_insert_fresh(ty.to_ast(), ty.abbrev(), symbols);
                        try_insert_fresh(assum.to_ast(), assum.abbrev(), symbols);
                    } else {
                        try_insert_fresh(ty.pretty(), ty.abbrev(), symbols);
                        try_insert_fresh(assum.pretty(), assum.abbrev(), symbols);
                    }
                }
                Expr::Single(expr) => {
                    Self::assign_fresh_var(expr, cache, symbols, to_rust);
                }
                Expr::Concat(lhs, rhs) => {
                    Self::assign_fresh_var(lhs, cache, symbols, to_rust);
                    Self::assign_fresh_var(rhs, cache, symbols, to_rust);
                }
                Expr::If(cond, input, then, els) => {
                    Self::assign_fresh_var(cond, cache, symbols, to_rust);
                    Self::assign_fresh_var(input, cache, symbols, to_rust);
                    Self::assign_fresh_var(then, cache, symbols, to_rust);
                    Self::assign_fresh_var(els, cache, symbols, to_rust);
                    cache.insert(expr_ptr, make_fresh("if".into()));
                }
                Expr::Switch(cond, input, branch) => {
                    Self::assign_fresh_var(cond, cache, symbols, to_rust);
                    Self::assign_fresh_var(input, cache, symbols, to_rust);
                    branch
                        .iter()
                        .for_each(|expr| Self::assign_fresh_var(expr, cache, symbols, to_rust));
                    cache.insert(expr_ptr, make_fresh("switch".into()));
                }
                Expr::DoWhile(input, body) => {
                    Self::assign_fresh_var(input, cache, symbols, to_rust);
                    Self::assign_fresh_var(body, cache, symbols, to_rust);
                    cache.insert(expr_ptr, make_fresh("dowhile".into()));
                }
                Expr::Arg(ty, assum) => {
                    if to_rust {
                        try_insert_fresh(ty.to_ast(), ty.abbrev(), symbols);
                        try_insert_fresh(assum.to_ast(), assum.abbrev(), symbols);
                    } else {
                        try_insert_fresh(ty.pretty(), ty.abbrev(), symbols);
                        try_insert_fresh(assum.pretty(), assum.abbrev(), symbols);
                    }
                }
                Expr::Function(_, tyin, tyout, body) => {
                    if to_rust {
                        try_insert_fresh(tyin.to_ast(), tyin.abbrev(), symbols);
                        try_insert_fresh(tyout.to_ast(), tyout.abbrev(), symbols);
                    } else {
                        try_insert_fresh(tyin.pretty(), tyin.abbrev(), symbols);
                        try_insert_fresh(tyout.pretty(), tyout.abbrev(), symbols);
                    }
                    Self::assign_fresh_var(body, cache, symbols, to_rust);
                }
                Expr::Symbolic(_) => panic!("no symbolic should occur here"),
            }
        }
    }

    fn to_nested_expr(
        &self,
        expr: &RcExpr,
        log: &mut IndexMap<String, Expr>,
        fold_when: &dyn Fn(usize, usize) -> bool,
        to_rust: bool,
    ) -> Expr {
        let fold = |egglog: Expr, log: &mut IndexMap<String, Expr>| {
            let fresh_var = self.cache.get(&Rc::as_ptr(expr)).unwrap();
            if !log.contains_key(fresh_var) {
                log.insert(fresh_var.into(), egglog);
            }
            Expr::Symbolic(fresh_var.into())
        };
        let fold_or_plain = |egglog: Expr, log: &mut IndexMap<String, Expr>| {
            let rc = Rc::strong_count(expr);
            let size = egglog
                .clone()
                .to_string()
                .replace(&['(', ')', ' '][..], "")
                .len();
            if fold_when(rc, size) {
                fold(egglog, log)
            } else {
                egglog
            }
        };

        match expr.as_ref() {
            Expr::Function(name, inty, outty, body) => {
                let inty_str: &String;
                let outty_str: &String;
                if to_rust {
                    inty_str = self.symbols.get(&inty.to_ast()).unwrap();
                    outty_str = self.symbols.get(&outty.to_ast()).unwrap();
                } else {
                    inty_str = self.symbols.get(&inty.pretty()).unwrap();
                    outty_str = self.symbols.get(&outty.pretty()).unwrap();
                }

                let body = self.to_nested_expr(body, log, fold_when, to_rust);
                Expr::Function(
                    name.into(),
                    Type::Symbolic(inty_str.into()),
                    Type::Symbolic(outty_str.into()),
                    Rc::new(body),
                )
            }
            Expr::Const(c, ty, assum) => {
                let ty_str: &String;
                let assum_str: &String;
                if to_rust {
                    ty_str = self.symbols.get(&ty.to_ast()).unwrap();
                    assum_str = self.symbols.get(&assum.to_ast()).unwrap();
                } else {
                    ty_str = self.symbols.get(&ty.pretty()).unwrap();
                    assum_str = self.symbols.get(&assum.pretty()).unwrap();
                }
                let c = Expr::Const(
                    c.clone(),
                    Type::Symbolic(ty_str.into()),
                    Assumption::WildCard(assum_str.into()),
                );

                if to_rust {
                    c
                } else {
                    fold(c, log)
                }
            }
            Expr::Top(op, x, y, z) => {
                let left = self.to_nested_expr(x, log, fold_when, to_rust);
                let mid = self.to_nested_expr(y, log, fold_when, to_rust);
                let right = self.to_nested_expr(z, log, fold_when, to_rust);
                let top = Expr::Top(op.clone(), Rc::new(left), Rc::new(mid), Rc::new(right));
                fold_or_plain(top, log)
            }
            Expr::Bop(op, x, y) => {
                let left = self.to_nested_expr(x, log, fold_when, to_rust);
                let right = self.to_nested_expr(y, log, fold_when, to_rust);
                let bop = Expr::Bop(op.clone(), Rc::new(left), Rc::new(right));
                fold_or_plain(bop, log)
            }
            Expr::Uop(op, x) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when, to_rust);
                let uop = Expr::Uop(op.clone(), Rc::new(sub_expr));
                fold_or_plain(uop, log)
            }
            Expr::Get(x, pos) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when, to_rust);
                let get = Expr::Get(Rc::new(sub_expr), *pos);
                // fold Get Arg i anyway
                if let Expr::Arg(_, _) = x.as_ref() {
                    if !to_rust {
                        return fold(get, log);
                    }
                }
                get
            }
            Expr::Alloc(id, x, y, ty) => {
                let amount = self.to_nested_expr(x, log, fold_when, to_rust);
                let state_edge = self.to_nested_expr(y, log, fold_when, to_rust);
                let alloc = Expr::Alloc(*id, Rc::new(amount), Rc::new(state_edge), ty.clone());
                fold_or_plain(alloc, log)
            }
            Expr::Call(name, x) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when, to_rust);
                let call = Expr::Call(name.into(), Rc::new(sub_expr));
                fold_or_plain(call, log)
            }
            Expr::Empty(ty, assum) => {
                let ty_str: &String;
                let assum_str: &String;
                if to_rust {
                    ty_str = self.symbols.get(&ty.to_ast()).unwrap();
                    assum_str = self.symbols.get(&assum.to_ast()).unwrap();
                } else {
                    ty_str = self.symbols.get(&ty.pretty()).unwrap();
                    assum_str = self.symbols.get(&assum.pretty()).unwrap();
                }

                Expr::Empty(
                    Type::Symbolic(ty_str.into()),
                    Assumption::WildCard(assum_str.into()),
                )
            }
            // doesn't fold Tuple
            Expr::Single(x) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when, to_rust);
                Expr::Single(Rc::new(sub_expr))
            }
            Expr::Concat(x, y) => {
                let left = self.to_nested_expr(x, log, fold_when, to_rust);
                let right = self.to_nested_expr(y, log, fold_when, to_rust);
                Expr::Concat(Rc::new(left), Rc::new(right))
            }
            Expr::Switch(x, inputs, _branches) => {
                let cond = self.to_nested_expr(x, log, fold_when, to_rust);
                let inputs = self.to_nested_expr(inputs, log, fold_when, to_rust);
                let branches = _branches
                    .iter()
                    .map(|branch| Rc::new(self.to_nested_expr(branch, log, fold_when, to_rust)))
                    .collect::<Vec<_>>();
                let switch = Expr::Switch(Rc::new(cond), Rc::new(inputs), branches);
                fold_or_plain(switch, log)
            }
            Expr::If(x, inputs, y, z) => {
                let pred = self.to_nested_expr(x, log, fold_when, to_rust);
                let inputs = self.to_nested_expr(inputs, log, fold_when, to_rust);
                let left = self.to_nested_expr(y, log, fold_when, to_rust);
                let right = self.to_nested_expr(z, log, fold_when, to_rust);
                let if_expr = Expr::If(
                    Rc::new(pred),
                    Rc::new(inputs),
                    Rc::new(left),
                    Rc::new(right),
                );
                fold_or_plain(if_expr, log)
            }
            Expr::DoWhile(inputs, body) => {
                let inputs = self.to_nested_expr(inputs, log, fold_when, to_rust);
                let body = self.to_nested_expr(body, log, fold_when, to_rust);
                let dowhile = Expr::DoWhile(Rc::new(inputs), Rc::new(body));
                fold_or_plain(dowhile, log)
            }
            Expr::Arg(ty, assum) => {
                let ty_str: &String;
                let assum_str: &String;
                if to_rust {
                    ty_str = self.symbols.get(&ty.to_ast()).unwrap();
                    assum_str = self.symbols.get(&assum.to_ast()).unwrap();
                } else {
                    ty_str = self.symbols.get(&ty.pretty()).unwrap();
                    assum_str = self.symbols.get(&assum.pretty()).unwrap();
                }
                Expr::Arg(
                    Type::Symbolic(ty_str.into()),
                    Assumption::WildCard(assum_str.into()),
                )
            }
            Expr::Symbolic(_) => panic!("No symbolic should occur here"),
        }
    }
}

impl Expr {
    pub fn abbrev(&self) -> String {
        format!("{:?}", self)
    }

    pub fn pretty(&self) -> String {
        let (term, termdag) = Rc::new(self.clone()).to_egglog();
        let expr = termdag.term_to_expr(&term);
        expr.to_sexp().pretty()
    }

    fn concat_helper(&self) -> Vec<String> {
        match self {
            Expr::Concat(lhs, rhs) => {
                let mut lhs = lhs.as_ref().concat_helper();
                let mut rhs = rhs.as_ref().concat_helper();
                lhs.append(&mut rhs);
                lhs
            }
            Expr::Single(expr) => {
                let expr = Self::to_ast(expr.as_ref());
                vec![expr]
            }
            _ => panic!("Not well formed Concat, expr not in Single"),
        }
    }
    pub fn to_ast(&self) -> String {
        match self {
            Expr::Const(c, ..) => match c {
                schema::Constant::Bool(true) => "ttrue()".into(),
                schema::Constant::Bool(false) => "tfalse()".into(),
                schema::Constant::Int(n) => format!("int({})", n),
            },
            Expr::Top(op, x, y, z) => {
                let left = x.to_ast();
                let mid = y.to_ast();
                let right = z.to_ast();
                format!("{}({}, {}, {})", op.to_ast(), left, mid, right)
            }
            Expr::Bop(op, x, y) => {
                let left = x.to_ast();
                let right = y.to_ast();
                format!("{}({}, {})", op.to_ast(), left, right)
            }
            Expr::Uop(op, x) => {
                let expr = x.to_ast();
                format!("{}({})", op.to_ast(), expr)
            }
            Expr::Get(expr, index) => match expr.as_ref() {
                Expr::Arg(_, _) => {
                    format!("getat({index})")
                }
                _ => {
                    let expr = expr.to_ast();
                    format!("get({expr}, {index})")
                }
            },
            Expr::Alloc(id, expr, state, ty) => {
                let expr = expr.to_ast();
                let state = state.to_ast();
                let ty_str = ty.to_ast();
                format!("alloc({id}, {expr}, {state}, {ty_str})")
            }
            Expr::Call(name, arg) => {
                let arg = arg.to_ast();
                format!("call({name}, {arg})")
            }
            Expr::Empty(..) => "empty()".into(),
            Expr::Single(expr) => {
                let expr = expr.to_ast();
                format!("single({expr})")
            }
            Expr::Concat(..) => {
                let vec = Self::concat_helper(self);
                let inside = vec.join(", ");
                format!("parallel!({inside})")
            }
            Expr::If(cond, inputs, x, y) => {
                let cond = cond.to_ast();
                let input = inputs.to_ast();
                let then = x.to_ast();
                let els = y.to_ast();
                format!("tif({cond}, {input}, {then}, {els})")
            }
            Expr::Switch(cond, inputs, cases) => {
                let cond = cond.to_ast();
                let inputs = inputs.to_ast();
                let cases = cases
                    .iter()
                    .map(|expr| expr.to_ast())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("switch!({cond}, {inputs}; parallel!({cases}))")
            }
            Expr::DoWhile(inputs, body) => {
                let inputs = inputs.to_ast();
                let body = body.to_ast();
                format!("dowhile({inputs}, {body})")
            }
            Expr::Arg(..) => "arg()".into(),
            Expr::Function(name, inty, outty, body) => {
                let inty = inty.to_ast();
                let outty = outty.to_ast();
                let body = body.to_ast();
                format!("function(\"{name}\", {inty}, {outty}, {body})")
            }
            Expr::Symbolic(str) => str.into(),
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
            Assumption::InFunc(fun_name) => {
                format!("infunc(\"{fun_name}\".into())")
            }
            Assumption::InIf(is, pred, input) => {
                format!("inif({is}, {}, {})", pred.to_ast(), input.to_ast())
            }
            Assumption::InLoop(input, output) => {
                format!("inloop({}, {})", input.to_ast(), output.to_ast(),)
            }
            Assumption::InSwitch(is, pred, inputs) => {
                format!("inswitch({is}, {}, {})", pred.to_ast(), inputs.to_ast(),)
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
            Type::Symbolic(_) => panic!("found symbolic in to_ast"),
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

    let res = PrettyPrinter::from_string(expr_str.clone())
        .unwrap()
        .to_rust_default();
    println!("{res}")
}
