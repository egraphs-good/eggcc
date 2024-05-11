use crate::{
    from_egglog::FromEgglog, optimizations::body_contains, prologue, schema::{self, Assumption, BaseType, BinaryOp, Expr, RcExpr, TernaryOp, Type, UnaryOp}
};
use egglog::{util::IndexMap, TermDag};
use indexmap::IndexMap;
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Either3<A, B, C> {
    Left(A),
    Mid(B),
    Right(C),
}

pub struct PrettyPrinter {
    pub expr: RcExpr,
    pub cache: IndexMap<*const schema::Expr, String>,
    pub symbols: IndexMap<Either3<Type, *const Assumption, BaseType>, String>,
    // pub types: IndexMap<Type, String>,
    // pub assums: IndexMap<Assumption, String>,
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
        let mut cache = IndexMap::new();
        let mut symbols = IndexMap::new();
        Self::assign_fresh_var(&expr, &mut cache, &mut symbols);
        PrettyPrinter {
            expr,
            cache,
            symbols,
        }
    }

    pub fn to_egglog_default(&self) -> String {
        self.to_egglog(&|rc, len| (rc > 1 && len > 30) || len > 80)
    }

    pub fn to_egglog(&self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = IndexMap::new();
        let res = self.to_nested_expr(&self.expr, &mut log, fold_when);
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

    pub fn to_rust_default(&self) -> String {
        self.to_rust(&|rc, len| (rc > 1 && len > 30) || rc > 4 || len > 80)
    }

    // turn the Expr to a rust ast macro string.
    // return a rust ast macro
    // fold_when: provide a function that decide when to fold the macro to a let binding
    pub fn to_rust(&self, fold_when: &dyn Fn(usize, usize) -> bool) -> String {
        let mut log = IndexMap::new();
        let res = self.to_nested_expr(&self.expr, &mut log, fold_when);
        let log = self
            .symbols
            .iter()
            .map(|(expr, symbol)| format!("let {symbol} = {expr}; \n"))
            .chain(
                log.iter()
                    .map(|(var, expr)| format!("(let {var} \n{}) \n", expr.pretty()))
                    .collect::<Vec<_>>(),
            )
            .collect::<Vec<_>>()
            .join("");
        log + &format!("(let EXPR___\n{})", res.pretty())
    }

    fn assign_fresh_var(
        expr: &RcExpr,
        cache: &mut IndexMap<*const schema::Expr, String>,
        // types: &mut IndexMap<Type, String>,
        // assums: &mut IndexMap<*const Assumption, String>,
        symbols : &mut IndexMap<Either3<Type, *const Assumption, BaseType>, String>
    ) {
        let len = cache.len();
        let make_fresh = |info: String| format!("{info}_{}", len);
        // let try_insert_fresh =
        //     |var: String, info: String, symbols: IndexMap<String, ?A>| {
        //         if !symbols.contains_key(&var) {
        //             let fresh_var = format!("{info}_{}", symbols.len());
        //             symbols.insert(var, fresh_var);
        //         }
        //     };
        fn try_insert_fresh<T: std::hash::Hash + std::cmp::Eq> (var : T, info: String, symbols: &mut IndexMap<T, String>) {
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
                    try_insert_fresh(ty.to_owned(), ty.abbrev(), types);
                    try_insert_fresh( Rc::as_ptr(&Rc::new(assum.to_owned())) , assum.abbrev(), assums);
                    let c = match c {
                        schema::Constant::Int(i) => format!("int{i}"),
                        schema::Constant::Bool(b) => format!("bool{b}"),
                    };
                    cache.insert(expr_ptr, make_fresh(c));
                }
                Expr::Top(op, lhs, mid, rhs) => {
                    Self::assign_fresh_var(lhs, cache, types, assums);
                    Self::assign_fresh_var(mid, cache, types, assums);
                    Self::assign_fresh_var(rhs, cache, types, assums);
                    cache.insert(expr_ptr, make_fresh(op.to_ast()));
                }
                Expr::Bop(op, lhs, rhs) => {
                    Self::assign_fresh_var(lhs, cache, types, assums);
                    Self::assign_fresh_var(rhs, cache, types, assums);
                    cache.insert(expr_ptr, make_fresh(op.to_ast()));
                }
                Expr::Uop(op, expr) => {
                    Self::assign_fresh_var(expr, cache, types, assums);
                    cache.insert(expr_ptr, make_fresh(op.to_ast()));
                }
                Expr::Get(expr, usize) => {
                    if let Expr::Arg(..) = expr.as_ref() {
                        cache.insert(expr_ptr, make_fresh(format!("get_at_{usize}")));
                    }
                    Self::assign_fresh_var(expr, cache, types, assums);
                }
                Expr::Alloc(id, x, y, ptrty) => {
                    Self::assign_fresh_var(x, cache, types, assums);
                    Self::assign_fresh_var(y, cache, types, assums);
                    try_insert_fresh(ptrty.to_string(), ptrty.abbrev(), types, assums);
                    cache.insert(expr_ptr, expr.as_ref().abbrev() + &id.to_string());
                }
                Expr::Call(name, arg) => {
                    Self::assign_fresh_var(arg, cache, types, assums);
                    cache.insert(expr_ptr, make_fresh("call_".to_owned() + name));
                }
                Expr::Empty(ty, assum) => {
                    try_insert_fresh(ty.pretty(), ty.abbrev(), types, assums);
                    try_insert_fresh(assum.pretty(), assum.abbrev(), types, assums);
                }
                Expr::Single(expr) => {
                    Self::assign_fresh_var(expr, cache, types, assums);
                }
                Expr::Concat(lhs, rhs) => {
                    Self::assign_fresh_var(lhs, cache, types, assums);
                    Self::assign_fresh_var(rhs, cache, types, assums);
                }
                Expr::If(cond, input, then, els) => {
                    Self::assign_fresh_var(cond, cache, types, assums);
                    Self::assign_fresh_var(input, cache, types, assums);
                    Self::assign_fresh_var(then, cache, types, assums);
                    Self::assign_fresh_var(els, cache, types, assums);
                    cache.insert(expr_ptr, make_fresh("if".into()));
                }
                Expr::Switch(cond, input, branch) => {
                    Self::assign_fresh_var(cond, cache, types, assums);
                    Self::assign_fresh_var(input, cache, types, assums);
                    branch
                        .iter()
                        .for_each(|expr| Self::assign_fresh_var(expr, cache, types, assums));
                    cache.insert(expr_ptr, make_fresh("switch".into()));
                }
                Expr::DoWhile(input, body) => {
                    Self::assign_fresh_var(input, cache, types, assums);
                    Self::assign_fresh_var(body, cache, types, assums);
                    cache.insert(expr_ptr, make_fresh("dowhile".into()));
                }
                Expr::Arg(ty, assum) => {
                    try_insert_fresh(ty.pretty(), ty.abbrev(), types, assums);
                    try_insert_fresh(assum.pretty(), assum.abbrev(), types, assums);
                }
                Expr::Function(_, tyin, tyout, body) => {
                    try_insert_fresh(tyin.pretty(), tyin.abbrev(), types, assums);
                    try_insert_fresh(tyout.pretty(), tyout.abbrev(), types, assums);
                    Self::assign_fresh_var(body, cache, types, assums);
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
    ) -> Expr {
        let fold = |egglog: Expr, log: &mut IndexMap<String, Expr>| {
            let fresh_var = self.cache.get(&Rc::as_ptr(&expr)).unwrap();
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
                let inty_str = self.symbols.get(&inty.pretty()).unwrap();
                let outty_str = self.symbols.get(&outty.pretty()).unwrap();
                let body = self.to_nested_expr(body, log, fold_when);
                Expr::Function(
                    name.into(),
                    Type::Symbolic(inty_str.into()),
                    Type::Symbolic(outty_str.into()),
                    Rc::new(body),
                )
            }
            Expr::Const(c, ty, assum) => {
                let ty = self.symbols.get(&ty.pretty()).unwrap();
                let assum = self.symbols.get(&assum.pretty()).unwrap();
                let c = Expr::Const(
                    c.clone(),
                    Type::Symbolic(ty.into()),
                    Assumption::WildCard(assum.into()),
                );
                fold(c, log)
            }
            Expr::Top(op, x, y, z) => {
                let left = self.to_nested_expr(x, log, fold_when);
                let mid = self.to_nested_expr(y, log, fold_when);
                let right = self.to_nested_expr(z, log, fold_when);
                let top = Expr::Top(op.clone(), Rc::new(left), Rc::new(mid), Rc::new(right));
                fold_or_plain(top, log)
            }
            Expr::Bop(op, x, y) => {
                let left = self.to_nested_expr(x, log, fold_when);
                let right = self.to_nested_expr(y, log, fold_when);
                let bop = Expr::Bop(op.clone(), Rc::new(left), Rc::new(right));
                fold_or_plain(bop, log)
            }
            Expr::Uop(op, x) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when);
                let uop = Expr::Uop(op.clone(), Rc::new(sub_expr));
                fold_or_plain(uop, log)
            }
            Expr::Get(x, pos) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when);
                let get = Expr::Get(Rc::new(sub_expr), pos.clone());
                // fold Get Arg i anyway
                if let Expr::Arg(_, _) = x.as_ref() {
                    fold(get, log)
                } else {
                    get
                }
            }
            Expr::Alloc(id, x, y, ty) => {
                let amount = self.to_nested_expr(x, log, fold_when);
                let state_edge = self.to_nested_expr(y, log, fold_when);
                let alloc =
                    Expr::Alloc(id.clone(), Rc::new(amount), Rc::new(state_edge), ty.clone());
                fold_or_plain(alloc, log)
            }
            Expr::Call(name, x) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when);
                let call = Expr::Call(name.into(), Rc::new(sub_expr));
                fold_or_plain(call, log)
            }
            Expr::Empty(ty, assum) => {
                let ty = self.symbols.get(&ty.pretty()).unwrap();
                let assum = self.symbols.get(&assum.pretty()).unwrap();
                Expr::Empty(
                    Type::Symbolic(ty.into()),
                    Assumption::WildCard(assum.into()),
                )
            }
            // doesn't fold Tuple
            Expr::Single(x) => {
                let sub_expr = self.to_nested_expr(x, log, fold_when);
                Expr::Single(Rc::new(sub_expr))
            }
            Expr::Concat(x, y) => {
                let left = self.to_nested_expr(x, log, fold_when);
                let right = self.to_nested_expr(y, log, fold_when);
                Expr::Concat(Rc::new(left), Rc::new(right))
            }
            Expr::Switch(x, inputs, _branches) => {
                let cond = self.to_nested_expr(x, log, fold_when);
                let inputs = self.to_nested_expr(inputs, log, fold_when);
                let branches = _branches
                    .iter()
                    .map(|branch| Rc::new(self.to_nested_expr(branch, log, fold_when)))
                    .collect::<Vec<_>>();
                let switch = Expr::Switch(Rc::new(cond), Rc::new(inputs), branches);
                fold_or_plain(switch, log)
            }
            Expr::If(x, inputs, y, z) => {
                let pred = self.to_nested_expr(x, log, fold_when);
                let inputs = self.to_nested_expr(inputs, log, fold_when);
                let left = self.to_nested_expr(y, log, fold_when);
                let right = self.to_nested_expr(z, log, fold_when);
                let if_expr = Expr::If(
                    Rc::new(pred),
                    Rc::new(inputs),
                    Rc::new(left),
                    Rc::new(right),
                );
                fold_or_plain(if_expr, log)
            }
            Expr::DoWhile(inputs, body) => {
                let inputs = self.to_nested_expr(inputs, log, fold_when);
                let body = self.to_nested_expr(body, log, fold_when);
                let dowhile = Expr::DoWhile(Rc::new(inputs), Rc::new(body));
                fold_or_plain(dowhile, log)
            }
            Expr::Arg(ty, assum) => {
                let ty = self.symbols.get(&ty.pretty()).unwrap();
                let assum = self.symbols.get(&assum.pretty()).unwrap();
                Expr::Arg(
                    Type::Symbolic(ty.into()),
                    Assumption::WildCard(assum.into()),
                )
            }
            Expr::Symbolic(_) => panic!("No symbolic should occur here"),
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
            _ => panic!("Not well formed Concat, expr not in Single"),
        }
    }

    // fn to_ast(
    //     expr: &RcExpr,
    //     cache: &mut HashMap<*const schema::Expr, String>,
    //     symbols: &mut HashMap<String, String>,
    //     log: &mut String,
    //     fold_when: &dyn Fn(usize, usize) -> bool,
    // ) -> String {
        // let find_or_insert = |var: String,
        //                       info: String,
        //                       str_builder: &mut String,
        //                       symbols: &mut HashMap<String, String>| {
        //     let fresh_var = format!("{}_{}", info, symbols.len());
        //     symbols
        //         .entry(var.clone())
        //         .or_insert_with(|| {
        //             str_builder.push_str(&format!("let {} = {}; \n", fresh_var.clone(), var));
        //             fresh_var
        //         })
        //         .clone()
        // };

        // let fold_or_plain =
        //     |ast_str: String,
        //      info: String,
        //      str_builder: &mut String,
        //      cache: &mut HashMap<*const schema::Expr, String>| {
        //         let rc = Rc::strong_count(expr);
        //         if fold_when(rc, ast_str.len()) {
        //             let fresh_var = format!("{}_{}", info, cache.len());
        //             let lookup = cache
        //                 .entry(Rc::as_ptr(expr))
        //                 .or_insert_with(|| {
        //                     str_builder.push_str(&format!(
        //                         "let {} = {}; \n",
        //                         fresh_var.clone(),
        //                         ast_str
        //                     ));
        //                     fresh_var
        //                 })
        //                 .clone();
        //             format!("{lookup}.clone()")
        //         } else {
        //             ast_str
        //         }
        //     };

        // match cache.get(&Rc::as_ptr(expr)) {
        //     Some(str) => format!("{}.clone()", str),
        //     None => {
        //         match expr.as_ref() {
        //             // just don't fold simple things like expr, getat anyway
        //             Expr::Const(c, _, _) => match c {
        //                 schema::Constant::Bool(true) => "ttrue()".into(),
        //                 schema::Constant::Bool(false) => "tfalse()".into(),
        //                 schema::Constant::Int(n) => format!("int({})", n),
        //             },
        //             Expr::Bop(op, lhs, rhs) => {
        //                 let left = Self::to_ast(lhs, cache, symbols, log, fold_when);
        //                 let right = Self::to_ast(rhs, cache, symbols, log, fold_when);
        //                 let ast_str = format!("{}({}, {})", op.to_ast(), left, right);
        //                 fold_or_plain(ast_str, op.to_ast(), log, cache)
        //             }
        //             Expr::Top(op, x, y, z) => {
        //                 let left = Self::to_ast(x, cache, symbols, log, fold_when);
        //                 let mid = Self::to_ast(y, cache, symbols, log, fold_when);
        //                 let right = Self::to_ast(z, cache, symbols, log, fold_when);
        //                 let ast_str = format!("{}({}, {}, {})", op.to_ast(), left, mid, right);
        //                 fold_or_plain(ast_str, op.to_ast(), log, cache)
        //             }
        //             Expr::Uop(op, expr) => {
        //                 let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
        //                 let ast_str = format!("{}({})", op.to_ast(), expr);
        //                 fold_or_plain(ast_str, op.to_ast(), log, cache)
        //             }
        //             Expr::Get(expr, index) => match expr.as_ref() {
        //                 Expr::Arg(_, _) => {
        //                     format!("getat({index})")
        //                 }
        //                 _ => {
        //                     let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
        //                     format!("get({expr}, {index})")
        //                 }
        //             },
        //             Expr::Alloc(id, expr, state, ty) => {
        //                 let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
        //                 let state = Self::to_ast(state, cache, symbols, log, &fold_when);
        //                 let ty_str = ty.to_ast();
        //                 let ty_binding = find_or_insert(ty_str, ty.abbrev(), log, symbols);
        //                 let ast_str = format!("alloc({id}, {expr}, {state}, {ty_binding})");
        //                 fold_or_plain(ast_str, "alloc".into(), log, cache)
        //             }
        //             Expr::Call(name, arg) => {
        //                 let arg = Self::to_ast(arg, cache, symbols, log, fold_when);
        //                 format!("call({name}, {arg})")
        //             }
        //             Expr::Empty(..) => "empty()".into(),
        //             Expr::Single(expr) => {
        //                 let expr = Self::to_ast(expr, cache, symbols, log, fold_when);
        //                 format!("single({expr})")
        //             }
        //             Expr::Concat(..) => {
        //                 let vec = Self::concat_helper(expr, cache, symbols, log, fold_when);
        //                 let inside = vec.join(", ");
        //                 format!("parallel!({inside})")
        //             }
        //             Expr::Switch(cond, inputs, cases) => {
        //                 let cond = Self::to_ast(cond, cache, symbols, log, fold_when);
        //                 let inputs = Self::to_ast(inputs, cache, symbols, log, fold_when);
        //                 let cases = cases
        //                     .iter()
        //                     .map(|expr| Self::to_ast(expr, cache, symbols, log, fold_when))
        //                     .collect::<Vec<_>>()
        //                     .join(", ");
        //                 let ast_str = format!("switch!({cond}, {inputs}; parallel!({cases}))");
        //                 fold_or_plain(ast_str, "switch".into(), log, cache)
        //             }
        //             Expr::If(cond, input, then, els) => {
        //                 let cond = Self::to_ast(cond, cache, symbols, log, fold_when);
        //                 let input = Self::to_ast(input, cache, symbols, log, fold_when);
        //                 let then = Self::to_ast(then, cache, symbols, log, fold_when);
        //                 let els = Self::to_ast(els, cache, symbols, log, fold_when);
        //                 let ast_str = format!("tif({cond}, {input}, {then}, {els})");
        //                 fold_or_plain(ast_str, "if".into(), log, cache)
        //             }
        //             Expr::DoWhile(input, body) => {
        //                 let input = Self::to_ast(input, cache, symbols, log, fold_when);
        //                 let body = Self::to_ast(body, cache, symbols, log, fold_when);
        //                 let ast_str = format!("dowhile({input}, {body})");
        //                 fold_or_plain(ast_str, "dowhile".into(), log, cache)
        //             }
        //             Expr::Arg(..) => "arg()".into(),
        //             Expr::Function(name, ty_in, ty_out, body) => {
        //                 let ty_in_str = ty_in.to_ast();
        //                 let ty_in_binding = find_or_insert(ty_in_str, ty_in.abbrev(), log, symbols);
        //                 let ty_out_str = ty_out.to_ast();
        //                 let ty_out_binding =
        //                     find_or_insert(ty_out_str, ty_out.abbrev(), log, symbols);
        //                 let body = Self::to_ast(body, cache, symbols, log, fold_when);
        //                 format!("function(\"{name}\", {ty_in_binding}, {ty_out_binding}, {body})")
        //             }
        //             Expr::Symbolic(_) => panic!("no symbolic should occur here"),
        //         }
        //     }
        // }
    //}
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

    pub fn to_ast(&self) -> String {
        let e = String::new();
        match self {
            Expr::Const(c, ..) => match c {
                schema::Constant::Bool(true) => "ttrue()".into(),
                schema::Constant::Bool(false) => "tfalse()".into(),
                schema::Constant::Int(n) => format!("int({})", n),
            }
            Expr::Top(op, x, y, z) => {
                let left = x.to_ast();
                let mid = y.to_ast();
                let right = x.to_ast();
                format!("{}({}, {}, {})", op.to_ast(), left, mid, right)
            },
            Expr::Bop(op, x, y) => {
                let left = x.to_ast();
                let right = y.to_ast();
                format!("{}({}, {})", op.to_ast(), left, right)
            },
            Expr::Uop(op, x) => {
                let expr = x.to_ast();
                format!("{}({})", op.to_ast(), expr)
            },
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
            },
            Expr::Call(name, arg) => {
                let arg = arg.to_ast();
                format!("call({name}, {arg})")
            },
            Expr::Empty(..) => "empty()".into(),
            Expr::Single(expr) => {
                let expr = expr.to_ast();
                format!("single({expr})")
            },
            Expr::Concat(..) => {e},
            Expr::If(cond, inputs, x, y) => {
                let cond = cond.to_ast();
                let input = inputs.to_ast();
                let then = x.to_ast();
                let els = y.to_ast();
                format!("tif({cond}, {input}, {then}, {els})")  
            },
            Expr::Switch(cond, inputs, cases) => {
                let cond = cond.to_ast();
                let inputs = inputs.to_ast();
                let cases = cases
                    .iter()
                    .map(|expr| expr.to_ast())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("switch!({cond}, {inputs}; parallel!({cases}))")
            },
            Expr::DoWhile(inputs, body) => {
                let inputs = inputs.to_ast();
                let body = body.to_ast();
                format!("dowhile({inputs}, {body})")
            },
            Expr::Arg(..) => "arg()".into(),
            Expr::Function(name, inty, outty, body) => {
                let inty = inty.to_ast();
                let outty = outty.to_ast();
                let body = body.to_ast();
                format!("function(\"{name}\", {inty}, {outty}, {body})")
            },
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
        .to_egglog_default();

    println!("{res}")
}
