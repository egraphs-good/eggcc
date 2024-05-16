use crate::{
    from_egglog::FromEgglog,
    prologue,
    schema::{self, Assumption, BaseType, BinaryOp, Expr, RcExpr, TernaryOp, Type, UnaryOp},
    schema_helpers::AssumptionRef,
    to_egglog::TreeToEgglog,
};
use egglog::{Term, TermDag};

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    rc::Rc,
};

pub struct PrettyPrinter {
    pub expr: RcExpr,
    // Type/Assum/BaseType -> intermediate variables
    symbols: indexmap::IndexMap<Symbols, String>,
}

#[derive(PartialEq, Eq, Hash)]
enum Symbols {
    Type(schema::Type),
    Assumption(AssumptionRef),
    Expr(*const schema::Expr),
}

#[derive(PartialEq, Eq)]
enum SymbolLog {
    Type(schema::Type),
    Assumption(schema::Assumption),
    Expr(schema::Expr),
}

impl SymbolLog {
    pub(crate) fn symbol_log_to_str(&self, to_rust: bool) -> String {
        match self {
            SymbolLog::Assumption(assum) => {
                if to_rust {
                    assum.to_ast()
                } else {
                    assum.pretty()
                }
            }
            SymbolLog::Type(ty) => {
                if to_rust {
                    ty.to_ast()
                } else {
                    ty.pretty()
                }
            }
            SymbolLog::Expr(expr) => {
                if to_rust {
                    expr.to_ast()
                } else {
                    expr.pretty()
                }
            }
        }
    }
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
        PrettyPrinter {
            expr,
            symbols: indexmap::IndexMap::new(),
        }
    }

    pub fn to_egglog_default(&mut self) -> String {
        self.to_egglog(&|rc, len| (rc > 1 && len > 30) || len > 80)
    }

    // turn the Expr to a nested egglog with intermediate variables.
    // fold_when: provide a function that decide when to fold the egglog expression to a let binding
    pub fn to_egglog(&mut self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = Vec::new();
        let mut table = std::collections::BTreeMap::<String, SymbolLog>::new();
        self.assign_fresh_var(&self.expr.clone());
        let res = self.fold_expr(&self.expr.clone(), &mut log, &mut table, fold_when, false);
        let log = log
            .iter()
            .map(|fresh_var| {
                let symbol = table.get(fresh_var).unwrap();
                let pretty = symbol.symbol_log_to_str(false);
                format!("(let {fresh_var} \n{pretty})\n")
            })
            .collect::<Vec<_>>()
            .join("");
        log + &format!("\n(let EXPR___\n{})\n", res.pretty())
    }

    pub fn to_rust_default(&mut self) -> String {
        self.to_rust(&|rc, len| (rc > 1 && len > 30) || rc > 4 || len > 80)
    }

    // turn the Expr to a rust ast macro string.
    // fold_when: provide a function that decide when to fold the macro to a let binding
    pub fn to_rust(&mut self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = Vec::new();
        let mut table = std::collections::BTreeMap::<String, SymbolLog>::new();
        self.assign_fresh_var(&self.expr.clone());
        let res = self.fold_expr(&self.expr.clone(), &mut log, &mut table, fold_when, false);
        let log = log
            .iter()
            .map(|fresh_var| {
                let symbol = table.get(fresh_var).unwrap();
                let ast = symbol.symbol_log_to_str(true);
                format!("let {fresh_var} = {ast};")
            })
            .collect::<Vec<_>>()
            .join("\n");
        log + &format!("\nlet expr___ = {};\n", res.to_ast())
    }

    fn assign_fresh_var(&mut self, expr: &RcExpr) {
        fn handle_assum(assum: &Assumption, pp: &mut PrettyPrinter) {
            match assum {
                Assumption::InLoop(inputs, body) => {
                    pp.assign_fresh_var(inputs);
                    pp.assign_fresh_var(body);
                }
                Assumption::InFunc(_) => {}
                Assumption::InIf(_, left, right) => {
                    pp.assign_fresh_var(left);
                    pp.assign_fresh_var(right);
                }
                Assumption::InSwitch(_, inputs, branch) => {
                    pp.assign_fresh_var(inputs);
                    pp.assign_fresh_var(branch);
                }
                Assumption::WildCard(_) => panic!("should not have wildcard here"),
            }
        }

        fn try_insert_fresh(var: Symbols, info: String, pp: &mut PrettyPrinter) {
            if !pp.symbols.contains_key(&var) {
                let fresh_var = format!("{info}_{}", pp.symbols.len());
                pp.symbols.insert(var, fresh_var.clone());
            }
        }

        let expr_symbol = Symbols::Expr(Rc::as_ptr(expr));
        // some expr need fresh var, other do not
        if !self.symbols.contains_key(&expr_symbol) {
            match expr.as_ref() {
                Expr::Const(c, ty, assum) => {
                    try_insert_fresh(Symbols::Type(ty.clone()), ty.abbrev(), self);
                    handle_assum(assum, self);
                    try_insert_fresh(
                        Symbols::Assumption(Assumption::to_ref(assum)),
                        assum.abbrev(),
                        self,
                    );
                    let c = match c {
                        schema::Constant::Int(i) => format!("int{i}"),
                        schema::Constant::Bool(b) => format!("bool{b}"),
                    };
                    try_insert_fresh(expr_symbol, c, self);
                }
                Expr::Top(op, lhs, mid, rhs) => {
                    self.assign_fresh_var(lhs);
                    self.assign_fresh_var(mid);
                    self.assign_fresh_var(rhs);
                    try_insert_fresh(expr_symbol, op.to_ast(), self);
                }
                Expr::Bop(op, lhs, rhs) => {
                    self.assign_fresh_var(lhs);
                    self.assign_fresh_var(rhs);
                    try_insert_fresh(expr_symbol, op.to_ast(), self);
                }
                Expr::Uop(op, expr) => {
                    self.assign_fresh_var(expr);
                    try_insert_fresh(expr_symbol, op.to_ast(), self);
                }
                Expr::Get(expr, usize) => {
                    self.assign_fresh_var(expr);
                    if let Expr::Arg(..) = expr.as_ref() {
                        try_insert_fresh(expr_symbol, format!("get_at_{usize}"), self);
                    }
                }
                Expr::Alloc(id, x, y, _) => {
                    self.assign_fresh_var(x);
                    self.assign_fresh_var(y);
                    try_insert_fresh(expr_symbol, "alloc".to_owned() + &format!("id{id}"), self);
                }
                Expr::Call(name, arg) => {
                    self.assign_fresh_var(arg);
                    try_insert_fresh(expr_symbol, "call_".to_owned() + name, self);
                }
                Expr::Empty(ty, assum) => {
                    try_insert_fresh(Symbols::Type(ty.clone()), ty.abbrev(), self);
                    handle_assum(assum, self);
                    try_insert_fresh(
                        Symbols::Assumption(Assumption::to_ref(assum)),
                        assum.abbrev(),
                        self,
                    );
                }
                Expr::Single(expr) => {
                    self.assign_fresh_var(expr);
                }
                Expr::Concat(lhs, rhs) => {
                    self.assign_fresh_var(lhs);
                    self.assign_fresh_var(rhs);
                }
                Expr::If(cond, input, then, els) => {
                    self.assign_fresh_var(cond);
                    self.assign_fresh_var(input);
                    self.assign_fresh_var(then);
                    self.assign_fresh_var(els);
                    try_insert_fresh(expr_symbol, "if".into(), self);
                }
                Expr::Switch(cond, input, branch) => {
                    self.assign_fresh_var(cond);
                    self.assign_fresh_var(input);
                    branch.iter().for_each(|expr| self.assign_fresh_var(expr));
                    try_insert_fresh(expr_symbol, "switch".into(), self);
                }
                Expr::DoWhile(input, body) => {
                    self.assign_fresh_var(input);
                    self.assign_fresh_var(body);
                    try_insert_fresh(expr_symbol, "dowhile".into(), self);
                }
                Expr::Arg(ty, assum) => {
                    try_insert_fresh(Symbols::Type(ty.clone()), ty.abbrev(), self);
                    handle_assum(assum, self);
                    try_insert_fresh(
                        Symbols::Assumption(Assumption::to_ref(assum)),
                        assum.abbrev(),
                        self,
                    );
                }
                Expr::Function(_, tyin, tyout, body) => {
                    try_insert_fresh(Symbols::Type(tyin.clone()), tyin.abbrev(), self);
                    try_insert_fresh(Symbols::Type(tyout.clone()), tyout.abbrev(), self);
                    self.assign_fresh_var(body);
                }
                Expr::Symbolic(_) => panic!("no symbolic should occur when assigning freshvar"),
            }
        }
    }

    fn fold_expr(
        &mut self,
        expr: &RcExpr,
        log: &mut Vec<String>,
        table: &mut BTreeMap<String, SymbolLog>,
        fold_when: &dyn Fn(usize, usize) -> bool,
        to_rust: bool,
    ) -> Expr {
        let old_expr_addr = Rc::as_ptr(expr);
        let fold = |pp: &mut PrettyPrinter,
                    new_expr: schema::Expr,
                    log: &mut Vec<String>,
                    table: &mut BTreeMap<String, SymbolLog>| {
            let fresh_var = pp.symbols.get(&Symbols::Expr(old_expr_addr)).unwrap();
            if !table.contains_key(fresh_var) {
                log.push(fresh_var.into());
                table.insert(fresh_var.into(), SymbolLog::Expr(new_expr));
            }
            Expr::Symbolic(fresh_var.into())
        };

        let num_shared = Rc::strong_count(expr);
        let fold_or_plain = |pp: &mut PrettyPrinter,
                             new_expr: Expr,
                             log: &mut Vec<String>,
                             table: &mut BTreeMap<String, SymbolLog>| {
            let size = &new_expr
                .to_string()
                .replace(&['(', ')', ' ', ','][..], "") //don't count those char when computing size
                .len();
            if fold_when(num_shared, *size) {
                fold(pp, new_expr, log, table)
            } else {
                new_expr
            }
        };

        fn handle_assum(
            assum: &Assumption,
            pp: &mut PrettyPrinter,
            log: &mut Vec<String>,
            table: &mut BTreeMap<String, SymbolLog>,
            fold_when: &dyn Fn(usize, usize) -> bool,
            to_rust: bool,
        ) -> String {
            let old_assume_binding = pp
                .symbols
                .get(&Symbols::Assumption(assum.to_ref()))
                .unwrap()
                .clone();
            if !table.contains_key(&old_assume_binding) {
                let new_assum = match assum {
                    Assumption::InFunc(_) => assum.clone(),
                    Assumption::InIf(cond, left, right) => {
                        let left = pp.fold_expr(left, log, table, fold_when, to_rust);
                        let right = pp.fold_expr(right, log, table, fold_when, to_rust);
                        Assumption::InIf(*cond, Rc::new(left), Rc::new(right))
                    }
                    Assumption::InLoop(inputs, body) => {
                        let inputs = pp.fold_expr(inputs, log, table, fold_when, to_rust);
                        let body = pp.fold_expr(body, log, table, fold_when, to_rust);
                        Assumption::InLoop(Rc::new(inputs), Rc::new(body))
                    }
                    Assumption::InSwitch(cond, inputs, branch) => {
                        let inputs = pp.fold_expr(inputs, log, table, fold_when, to_rust);
                        let branch = pp.fold_expr(branch, log, table, fold_when, to_rust);
                        Assumption::InSwitch(*cond, Rc::new(inputs), Rc::new(branch))
                    }
                    Assumption::WildCard(_) => assum.clone(),
                };
                log.push(old_assume_binding.clone());
                table.insert(old_assume_binding.clone(), SymbolLog::Assumption(new_assum));
            }

            old_assume_binding
        }

        fn handle_type(
            ty: &Type,
            pp: &mut PrettyPrinter,
            log: &mut Vec<String>,
            table: &mut BTreeMap<String, SymbolLog>,
        ) -> Type {
            let ty_str = pp.symbols.get(&Symbols::Type(ty.clone())).unwrap().clone();
            if !table.contains_key(&ty_str) {
                log.push(ty_str.clone());
                table.insert(ty_str.clone(), SymbolLog::Type(ty.clone()));
            }
            Type::Symbolic(ty_str)
        }

        match expr.as_ref() {
            Expr::Function(name, inty, outty, body) => {
                let inty = handle_type(inty, self, log, table);
                let outty = handle_type(outty, self, log, table);

                let body = self.fold_expr(body, log, table, fold_when, to_rust);
                Expr::Function(name.into(), inty, outty, Rc::new(body))
            }
            Expr::Const(c, ty, assum) => {
                let ty = handle_type(ty, self, log, table);

                let old_assum_binding = handle_assum(assum, self, log, table, fold_when, to_rust);
                let c = Expr::Const(c.clone(), ty, Assumption::WildCard(old_assum_binding));

                if to_rust {
                    c
                } else {
                    fold(self, c, log, table)
                }
            }
            Expr::Top(op, x, y, z) => {
                let left = self.fold_expr(x, log, table, fold_when, to_rust);
                let mid = self.fold_expr(y, log, table, fold_when, to_rust);
                let right = self.fold_expr(z, log, table, fold_when, to_rust);
                let top = Expr::Top(op.clone(), Rc::new(left), Rc::new(mid), Rc::new(right));
                fold_or_plain(self, top, log, table)
            }
            Expr::Bop(op, x, y) => {
                let left = self.fold_expr(x, log, table, fold_when, to_rust);
                let right = self.fold_expr(y, log, table, fold_when, to_rust);
                let bop = Expr::Bop(op.clone(), Rc::new(left), Rc::new(right));
                fold_or_plain(self, bop, log, table)
            }
            Expr::Uop(op, x) => {
                let sub_expr = self.fold_expr(x, log, table, fold_when, to_rust);
                let uop = Expr::Uop(op.clone(), Rc::new(sub_expr));
                fold_or_plain(self, uop, log, table)
            }
            Expr::Get(x, pos) => {
                let sub_expr = self.fold_expr(x, log, table, fold_when, to_rust);
                let get = Expr::Get(Rc::new(sub_expr), *pos);
                // fold Get Arg i anyway
                if let Expr::Arg(_, _) = x.as_ref() {
                    if !to_rust {
                        return fold(self, get, log, table);
                    }
                }
                get
            }
            Expr::Alloc(id, x, y, ty) => {
                let amount = self.fold_expr(x, log, table, fold_when, to_rust);
                let state_edge = self.fold_expr(y, log, table, fold_when, to_rust);
                let alloc = Expr::Alloc(*id, Rc::new(amount), Rc::new(state_edge), ty.clone());
                fold_or_plain(self, alloc, log, table)
            }
            Expr::Call(name, x) => {
                let sub_expr = self.fold_expr(x, log, table, fold_when, to_rust);
                let call = Expr::Call(name.into(), Rc::new(sub_expr));
                fold_or_plain(self, call, log, table)
            }
            Expr::Empty(ty, assum) => {
                let ty = handle_type(ty, self, log, table);
                let assum_str = handle_assum(assum, self, log, table, fold_when, to_rust);

                Expr::Empty(ty, Assumption::WildCard(assum_str))
            }
            // doesn't fold Tuple
            Expr::Single(x) => {
                let sub_expr = self.fold_expr(x, log, table, fold_when, to_rust);
                Expr::Single(Rc::new(sub_expr))
            }
            Expr::Concat(x, y) => {
                let left = self.fold_expr(x, log, table, fold_when, to_rust);
                let right = self.fold_expr(y, log, table, fold_when, to_rust);
                Expr::Concat(Rc::new(left), Rc::new(right))
            }
            Expr::Switch(x, inputs, _branches) => {
                let cond = self.fold_expr(x, log, table, fold_when, to_rust);
                let inputs = self.fold_expr(inputs, log, table, fold_when, to_rust);
                let branches = _branches
                    .iter()
                    .map(|branch| Rc::new(self.fold_expr(branch, log, table, fold_when, to_rust)))
                    .collect::<Vec<_>>();
                let switch = Expr::Switch(Rc::new(cond), Rc::new(inputs), branches);
                fold_or_plain(self, switch, log, table)
            }
            Expr::If(x, inputs, y, z) => {
                let pred = self.fold_expr(x, log, table, fold_when, to_rust);
                let inputs = self.fold_expr(inputs, log, table, fold_when, to_rust);
                let left = self.fold_expr(y, log, table, fold_when, to_rust);
                let right = self.fold_expr(z, log, table, fold_when, to_rust);
                let if_expr = Expr::If(
                    Rc::new(pred),
                    Rc::new(inputs),
                    Rc::new(left),
                    Rc::new(right),
                );
                fold_or_plain(self, if_expr, log, table)
            }
            Expr::DoWhile(inputs, body) => {
                let inputs = self.fold_expr(inputs, log, table, fold_when, to_rust);
                let body = self.fold_expr(body, log, table, fold_when, to_rust);
                let dowhile = Expr::DoWhile(Rc::new(inputs), Rc::new(body));
                fold_or_plain(self, dowhile, log, table)
            }
            Expr::Arg(ty, assum) => {
                let ty = handle_type(ty, self, log, table);
                let assum_str = handle_assum(assum, self, log, table, fold_when, to_rust);

                Expr::Arg(ty, Assumption::WildCard(assum_str))
            }
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
                Expr::Arg(..) => {
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
            Assumption::InFunc(fun_name) => format!("infunc(\"{fun_name}\".into())"),
            Assumption::InIf(is, pred, input) => {
                format!("inif({is}, {}, {})", pred.to_ast(), input.to_ast())
            }
            Assumption::InLoop(input, output) => {
                format!("inloop({}, {})", input.to_ast(), output.to_ast())
            }
            Assumption::InSwitch(is, pred, inputs) => {
                format!("inswitch({is}, {}, {})", pred.to_ast(), inputs.to_ast())
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
    println!("{res}");
}

// #[test]
// fn test_pretty() {
//     use crate::ast::*;
//     let expr = "(Function \"main\" (TupleT (TCons (StateT) (TNil))) (TupleT (TCons (StateT) (TNil))) (Single (Bop (Print) (Const (Int 1) (TupleT (TCons (StateT) (TNil))) (InFunc \"main\")) (Get (DoWhile (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InFunc \"main\")) 0)) (Concat (Single (Bop (LessThan) (Const (Int 3) (TupleT (TCons (StateT) (TNil))) (InLoop (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InFunc \"main\")) 0)) (Concat (Single (Bop (LessThan) (Const (Int 3) (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")) (Const (Int 1) (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")))) (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")) 0))))) (Const (Int 1) (TupleT (TCons (StateT) (TNil))) (InLoop (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InFunc \"main\")) 0)) (Concat (Single (Bop (LessThan) (Const (Int 3) (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")) (Const (Int 1) (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")))) (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")) 0))))))) (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InLoop (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InFunc \"main\")) 0)) (Concat (Single (Bop (LessThan) (Const (Int 3) (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")) (Const (Int 1) (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")))) (Single (Get (Arg (TupleT (TCons (StateT) (TNil))) (InFunc \"dummy\")) 0))))) 0)))) 0))))";

//     let pp = PrettyPrinter::from_string(expr.into()).unwrap().to_egglog_default();

//     println!("{pp}");
// }
