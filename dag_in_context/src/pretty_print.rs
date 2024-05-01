use std::{
    collections::{HashMap},
    rc::Rc,
    str,
};

use egglog::TermDag;

use crate::{
    from_egglog::FromEgglog,
    prologue,
    schema::{self, RcExpr},
};

pub fn pretty_print(str_expr: String) -> std::result::Result<String, egglog::Error> {
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
    let expr = converter.expr_from_egglog(extracted);
    let mut cache: HashMap<*const schema::Expr, String> = HashMap::new();
    let mut symbols: HashMap<String, String> = HashMap::new();
    let mut log = String::new();
    pretty_print_helper(&expr, &mut cache, &mut symbols, &mut log);
    Ok(log)
}

// symbols: Type and Assumption's string -> their binding var
fn pretty_print_helper(
    expr: &RcExpr,
    cache: &mut HashMap<*const schema::Expr, String>,
    symbols: &mut HashMap<String, String>,
    log: &mut String,
) -> String {
    use schema::Expr;
    let find_or_insert =
        |var: String, ty: &str, str_builder: &mut String, symbols: &mut HashMap<String, String>| {
            let fresh_var = format!("{}__{}", ty, symbols.len());
            symbols
                .entry(var.clone())
                .or_insert_with(|| {
                    str_builder.push_str(&format!("(let {} {}) \n", fresh_var.clone(), var));
                    fresh_var
                })
                .clone()
        };
    match cache.get(&Rc::as_ptr(expr)) {
        Some(str) => str.to_string(),
        None => {
            let expr = expr.as_ref();
            match expr {
                Expr::Function(name, inty, outty, body) => {
                    let inty_str = find_or_insert(inty.to_string(), "ty", log, symbols);
                    let outty_str = find_or_insert(outty.to_string(), "ty", log, symbols);
                    let body = pretty_print_helper(body, cache, symbols, log);
                    let fun = format!("(Function {name} {inty_str} {outty_str} {body})");
                    cache.insert(expr, fun.clone());
                    log.push_str(&format!("(let Fun_{name} {}) \n", fun.clone()));
                    fun
                }
                Expr::Const(c, ty, assum) => {
                    let ty = find_or_insert(ty.to_string(), "ty", log, symbols);
                    let assum = find_or_insert(assum.to_string(), "assum", log, symbols);
                    let constant = format!("(Const {c} {ty} {assum})");
                    cache.insert(expr, constant.clone());
                    constant
                }
                Expr::Top(op, x, y, z) => {
                    let left = pretty_print_helper(x, cache, symbols, log);
                    let mid = pretty_print_helper(y, cache, symbols, log);
                    let right = pretty_print_helper(z, cache, symbols, log);
                    let top = format!("(Top ({:?}) {} {} {})", op, left, mid, right); //?
                    let fresh_var = format!("{:?}__{}", op, cache.len());
                    log.push_str(&format!("(let {} {}) \n", fresh_var.clone(), top));
                    cache.insert(expr, fresh_var.clone());
                    fresh_var
                }
                Expr::Bop(op, x, y) => {
                    let left = pretty_print_helper(x, cache, symbols, log);
                    let right = pretty_print_helper(y, cache, symbols, log);
                    let bop = format!("(Bop ({:?}) {} {})", op, left, right); //?
                    let fresh_var = format!("{:?}__{}", op, cache.len());
                    log.push_str(&format!("(let {} {}) \n", fresh_var.clone(), bop));
                    cache.insert(expr, fresh_var.clone());
                    fresh_var
                }
                Expr::Uop(op, x) => {
                    let sub_expr = pretty_print_helper(x, cache, symbols, log);
                    let uop = format!("(Uop ({:?}) {})", op, sub_expr); //?
                    let fresh_var = format!("{:?}__{}", op, cache.len());
                    log.push_str(&format!("(let {} {}) \n", fresh_var, uop));
                    cache.insert(expr, fresh_var.clone());
                    fresh_var
                }
                Expr::Get(x, pos) => {
                    let sub_expr = pretty_print_helper(x, cache, symbols, log);
                    let get = format!("(Get {sub_expr} {pos})");
                    cache.insert(expr, get.clone());
                    get
                }
                Expr::Alloc(id, x, y, pointer_ty) => {
                    let amount = pretty_print_helper(x, cache, symbols, log);
                    let state_edge = pretty_print_helper(y, cache, symbols, log);
                    let ty = find_or_insert(pointer_ty.to_string(), "ty", log, symbols);
                    let alloc = format!("(Alloc {id} {amount} {state_edge} {ty})");
                    log.push_str(&format!(
                        "(let Alloc{id}__{} {}) \n",
                        cache.len(),
                        alloc.clone()
                    ));
                    cache.insert(expr, alloc.clone());
                    alloc
                }
                Expr::Call(name, x) => {
                    let sub_expr = pretty_print_helper(x, cache, symbols, log);
                    let call = format!("(Call {name} {sub_expr})");
                    log.push_str(&format!(
                        "(let CallFun_{name}__{} {}) \n",
                        cache.len(),
                        call.clone()
                    ));
                    cache.insert(expr, call.clone());
                    call
                }
                Expr::Empty(ty, assum) => {
                    let ty = find_or_insert(ty.to_string(), "ty", log, symbols);
                    let assum = find_or_insert(assum.to_string(), "assum", log, symbols);
                    let empty = format!("(Empty {ty} {assum})");
                    cache.insert(expr, empty.clone());
                    empty
                }
                // doesn't fold Tuple
                Expr::Single(x) => {
                    let sub_expr = pretty_print_helper(x, cache, symbols, log);
                    let single = format!("(Single {})", sub_expr.clone());
                    cache.insert(expr, single.clone());
                    single
                }
                Expr::Concat(x, y) => {
                    let left = pretty_print_helper(x, cache, symbols, log);
                    let right = pretty_print_helper(y, cache, symbols, log);
                    let concat = format!("(Concat {left} {right})");
                    cache.insert(expr, concat.clone());
                    concat
                }
                Expr::Switch(x, inputs, _branches) => {
                    let cond = pretty_print_helper(x, cache, symbols, log);
                    let inputs = pretty_print_helper(inputs, cache, symbols, log);

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
                        .map(|branch| pretty_print_helper(branch, cache, symbols, log))
                        .collect::<Vec<_>>();
                    let branch_list = cons_list(branches);

                    let switch = format!("(Switch {cond} {inputs} {branch_list})");
                    let fresh_var = format!("switch__{}", cache.len());
                    log.push_str(&format!("(let {} {}) \n", fresh_var.clone(), switch));
                    cache.insert(expr, fresh_var.clone());
                    fresh_var
                }
                Expr::If(x, inputs, y, z) => {
                    let pred = pretty_print_helper(x, cache, symbols, log);
                    let inputs = pretty_print_helper(inputs, cache, symbols, log);
                    let left = pretty_print_helper(y, cache, symbols, log);
                    let right = pretty_print_helper(z, cache, symbols, log);
                    let if_expr = format!("(If {pred} {inputs} {left} {right})");
                    let fresh_var = format!("if__{}", cache.len());
                    log.push_str(&format!("(let {} {}) \n", fresh_var.clone(), if_expr));
                    cache.insert(expr, fresh_var.clone());
                    fresh_var
                },
                Expr::DoWhile(inputs, body) => {
                    let inputs = pretty_print_helper(inputs, cache, symbols, log);
                    let body = pretty_print_helper(body, cache, symbols, log);
                    let dowhile = format!("(DoWhile {inputs} {body})");
                    let fresh_var = format!("dowhile__{}", cache.len());
                    log.push_str(&format!("(let {} {}) \n", fresh_var.clone(), dowhile));
                    cache.insert(expr, fresh_var.clone());
                    fresh_var
                },
                Expr::Arg(ty, assum) => {
                    let ty = find_or_insert(ty.to_string(), "ty", log, symbols);
                    let assum = find_or_insert(assum.to_string(), "assum", log, symbols);
                    let arg = format!("(Arg {ty} {assum})");
                    cache.insert(expr, arg.clone());
                    arg
                },
            }
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
    let print = tprint(inv.clone(), getat(4)).with_arg_types(output_ty.clone(), base(statet()));

    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4), getat(0)),
        concat(
            parallel!(pred.clone(), not_inv.clone(), getat(1)),
            concat(parallel!(getat(2), getat(3)), single(print.clone())),
        ),
    )
    .with_arg_types(tuplet!(statet()), output_ty.clone());

    let expr_str = my_loop.to_string();
    let expr = pretty_print(expr_str).unwrap();
    print!("{}", expr);
}
