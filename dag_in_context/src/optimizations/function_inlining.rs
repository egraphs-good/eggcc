use std::rc::Rc;

use indexmap::IndexMap;

use crate::schema::{Expr, RcExpr, TreeProgram};

fn subst_expr(arg: &RcExpr, within: &RcExpr) -> RcExpr {
    let mut cache = IndexMap::new();
    subst_expr_with_cache(arg, within, &mut cache)
}

fn subst_expr_with_cache(
    arg: &RcExpr,
    within: &RcExpr,
    cache: &mut IndexMap<*const Expr, RcExpr>,
) -> RcExpr {
    if let Some(cached) = cache.get(&Rc::as_ptr(within)) {
        return cached.clone();
    }

    let ptr = Rc::as_ptr(within);

    use Expr::*;
    let res = match within.as_ref() {
        Arg(_, _) => arg.clone(),
        Top(op, x, y, z) => Rc::new(Top(
            op.clone(),
            subst_expr_with_cache(arg, x, cache),
            subst_expr_with_cache(arg, y, cache),
            subst_expr_with_cache(arg, z, cache),
        )),
        Bop(op, x, y) => Rc::new(Bop(
            op.clone(),
            subst_expr_with_cache(arg, x, cache),
            subst_expr_with_cache(arg, y, cache),
        )),
        Uop(op, x) => Rc::new(Uop(op.clone(), subst_expr_with_cache(arg, x, cache))),
        Get(x, i) => Rc::new(Get(subst_expr_with_cache(arg, x, cache), *i)),
        Alloc(n, a, s, bt) => Rc::new(Alloc(
            *n,
            subst_expr_with_cache(arg, a, cache),
            subst_expr_with_cache(arg, s, cache),
            bt.clone(),
        )),
        Call(name, x) => Rc::new(Call(name.clone(), subst_expr_with_cache(arg, x, cache))),
        Empty(ty, ctx) => Rc::new(Empty(ty.clone(), ctx.clone())),
        Const(c, ty, ctx) => Rc::new(Const(c.clone(), ty.clone(), ctx.clone())),
        Single(x) => Rc::new(Single(subst_expr_with_cache(arg, x, cache))),
        Concat(x, y) => Rc::new(Concat(
            subst_expr_with_cache(arg, x, cache),
            subst_expr_with_cache(arg, y, cache),
        )),
        If(p, i, t, e) => Rc::new(If(
            subst_expr_with_cache(arg, p, cache),
            subst_expr_with_cache(arg, i, cache),
            t.clone(),
            e.clone(),
        )),
        Switch(p, i, bs) => Rc::new(Switch(
            subst_expr_with_cache(arg, p, cache),
            subst_expr_with_cache(arg, i, cache),
            bs.clone(),
        )),
        DoWhile(i, pb) => Rc::new(DoWhile(subst_expr_with_cache(arg, i, cache), pb.clone())),
        Function(n, tin, tout, b) => {
            Rc::new(Function(n.clone(), tin.clone(), tout.clone(), b.clone()))
        }
        Symbolic(s, ty) => Rc::new(Symbolic(s.clone(), ty.clone())),
    };

    cache.insert(ptr, res.clone());
    res
}

fn inline_once_in_expr(
    expr: &RcExpr,
    func_name_to_body: &IndexMap<String, RcExpr>,
    inlined_cache: &mut IndexMap<*const Expr, RcExpr>,
) -> RcExpr {
    use Expr::*;
    let ptr = Rc::as_ptr(expr);
    if let Some(cached) = inlined_cache.get(&ptr) {
        return cached.clone();
    }
    let result = match expr.as_ref() {
        Call(name, args) => {
            let args_inlined = inline_once_in_expr(args, func_name_to_body, inlined_cache);

            if let Some(body) = func_name_to_body.get(name) {
                subst_expr(&args_inlined, body)
            } else {
                panic!("Function {name} not found for inlining");
            }
        }
        Top(op, x, y, z) => Rc::new(Top(
            op.clone(),
            inline_once_in_expr(x, func_name_to_body, inlined_cache),
            inline_once_in_expr(y, func_name_to_body, inlined_cache),
            inline_once_in_expr(z, func_name_to_body, inlined_cache),
        )),
        Bop(op, x, y) => Rc::new(Bop(
            op.clone(),
            inline_once_in_expr(x, func_name_to_body, inlined_cache),
            inline_once_in_expr(y, func_name_to_body, inlined_cache),
        )),
        Uop(op, x) => Rc::new(Uop(
            op.clone(),
            inline_once_in_expr(x, func_name_to_body, inlined_cache),
        )),
        Get(x, i) => Rc::new(Get(
            inline_once_in_expr(x, func_name_to_body, inlined_cache),
            *i,
        )),
        Alloc(n, a, s, bt) => Rc::new(Alloc(
            *n,
            inline_once_in_expr(a, func_name_to_body, inlined_cache),
            inline_once_in_expr(s, func_name_to_body, inlined_cache),
            bt.clone(),
        )),
        Single(x) => Rc::new(Single(inline_once_in_expr(
            x,
            func_name_to_body,
            inlined_cache,
        ))),
        Concat(x, y) => Rc::new(Concat(
            inline_once_in_expr(x, func_name_to_body, inlined_cache),
            inline_once_in_expr(y, func_name_to_body, inlined_cache),
        )),
        If(p, i, t, e) => Rc::new(If(
            inline_once_in_expr(p, func_name_to_body, inlined_cache),
            inline_once_in_expr(i, func_name_to_body, inlined_cache),
            inline_once_in_expr(t, func_name_to_body, inlined_cache),
            inline_once_in_expr(e, func_name_to_body, inlined_cache),
        )),
        Switch(p, i, bs) => Rc::new(Switch(
            inline_once_in_expr(p, func_name_to_body, inlined_cache),
            inline_once_in_expr(i, func_name_to_body, inlined_cache),
            bs.iter()
                .map(|b| inline_once_in_expr(b, func_name_to_body, inlined_cache))
                .collect(),
        )),
        DoWhile(i, pb) => Rc::new(DoWhile(
            inline_once_in_expr(i, func_name_to_body, inlined_cache),
            inline_once_in_expr(pb, func_name_to_body, inlined_cache),
        )),
        Function(n, tin, tout, b) => Rc::new(Function(
            n.clone(),
            tin.clone(),
            tout.clone(),
            inline_once_in_expr(b, func_name_to_body, inlined_cache),
        )),
        Const(_, _, _) | Empty(_, _) | Arg(_, _) | Symbolic(_, _) => expr.clone(),
    };
    inlined_cache.insert(ptr, result.clone());
    result
}

#[allow(dead_code)]
fn build_func_body_map(program: &TreeProgram) -> IndexMap<String, RcExpr> {
    let mut map: IndexMap<String, RcExpr> = IndexMap::new();
    let mut push_fn = |f: &RcExpr| {
        let name = f
            .func_name()
            .expect("Function should have name for inlining");
        let body = f.func_body().expect("Function should have body").clone();
        map.insert(name, body);
    };
    push_fn(&program.entry);
    for f in &program.functions {
        push_fn(f);
    }
    map
}

pub fn perform_inlining(program: &TreeProgram, fns: Vec<String>, iterations: usize) -> TreeProgram {
    if iterations == 0 || fns.is_empty() {
        return program.clone();
    }
    let func_name_to_body = build_func_body_map(program);
    let rewrite_fn = |func: &RcExpr| {
        let name = func.func_name().unwrap();
        let mut body = func.func_body().unwrap().clone();
        if fns.contains(&name) {
            for _ in 0..iterations {
                let mut cache = IndexMap::new();
                body = inline_once_in_expr(&body, &func_name_to_body, &mut cache);
            }
        }
        Rc::new(Expr::Function(
            name,
            func.func_input_ty().unwrap(),
            func.func_output_ty().unwrap(),
            body,
        ))
    };

    let entry = rewrite_fn(&program.entry);
    let functions = program.functions.iter().map(rewrite_fn).collect();

    let res_untyped = TreeProgram { entry, functions };
    res_untyped.override_arg_types()
}
