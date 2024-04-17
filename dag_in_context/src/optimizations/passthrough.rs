#[cfg(test)]
use crate::egglog_test;
#[test]
fn passthrough_if_arg() -> crate::Result {
    use crate::ast::*;
    // Program is roughly:
    // zero = 0;
    // if (input < 0) {} {}
    // print zero
    let ty = tuplet_vec(vec![intt(), statet()]);
    let pred = less_eq(get(arg_ty(ty.clone()), 0), int_ty(0, ty.clone()));
    let zero = int_ty(0, ty.clone());
    let input = concat(single(get(arg_ty(ty.clone()), 1)), single(zero.clone()));
    let arg = arg_ty(tuplet_vec(vec![statet(), intt()]));
    let then = concat(
        single(get(
            inctx(inif(true, pred.clone(), input.clone()), arg.clone()),
            0,
        )),
        single(get(
            inctx(inif(true, pred.clone(), input.clone()), arg.clone()),
            1,
        )),
    );
    let els = concat(
        single(get(
            inctx(inif(false, pred.clone(), input.clone()), arg.clone()),
            0,
        )),
        single(get(
            inctx(inif(false, pred.clone(), input.clone()), arg.clone()),
            1,
        )),
    );
    let if_e = tif(pred.clone(), input.clone(), then.clone(), els.clone());
    let body = single(tprint(get(if_e.clone(), 1), get(if_e.clone(), 0)));
    egglog_test(
        &format!("{body}"),
        &format!("(check (= (Get {if_e} 1) {zero}))"),
        vec![body.to_program(
            tuplet_vec(vec![intt(), statet()]),
            tuplet_vec(vec![statet()]),
        )],
        tuplev_vec(vec![intv(2), statev()]),
        tuplev_vec(vec![statev()]),
        vec!["0".to_string()],
    )
}

#[test]
fn passthrough_if_predicate() -> crate::Result {
    use crate::ast::*;

    let build = get(
        tif(
            less_than(arg(), int(5)),
            empty(),
            single(ttrue()),
            single(tfalse()),
        ),
        0,
    );
    let check = less_than(arg(), int(5));

    let build = build.to_program(base(intt()), base(boolt()));
    let check = check.to_program(base(intt()), base(boolt()));
    egglog_test(
        &format!("(let b {build})"),
        &format!("(let c {check}) (check (= b c))"),
        vec![build, check],
        intv(3),
        val_bool(true),
        vec![],
    )
}
