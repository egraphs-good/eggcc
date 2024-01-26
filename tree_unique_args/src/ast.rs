use crate::{Expr, Expr::*, Id, Order};

pub fn give_fresh_ids(expr: &mut Expr) {
    let mut id = 1;
    give_fresh_ids_helper(expr, 0, &mut id);
}

fn give_fresh_ids_helper(expr: &mut Expr, current_id: i64, fresh_id: &mut i64) {
    match expr {
        Loop(id, input, body) => {
            let new_id = *fresh_id;
            *fresh_id += 1;
            *id = Id(new_id);
            give_fresh_ids_helper(input, current_id, fresh_id);
            give_fresh_ids_helper(body, new_id, fresh_id);
        }
        Let(id, arg, body) => {
            let new_id = *fresh_id;
            *fresh_id += 1;
            *id = Id(new_id);
            give_fresh_ids_helper(arg, current_id, fresh_id);
            give_fresh_ids_helper(body, new_id, fresh_id);
        }
        Arg(id) => {
            *id = Id(current_id);
        }
        Function(id, body) => {
            let new_id = *fresh_id;
            *fresh_id += 1;
            *id = Id(new_id);
            give_fresh_ids_helper(body, new_id, fresh_id);
        }
        Call(id, arg) => {
            *id = Id(current_id);
            give_fresh_ids_helper(arg, current_id, fresh_id);
        }
        _ => expr.for_each_child(move |child| give_fresh_ids_helper(child, current_id, fresh_id)),
    }
}

pub fn program(args: Vec<Expr>) -> Expr {
    let mut prog = Program(args);
    give_fresh_ids(&mut prog);
    prog
}

pub fn num(n: i64) -> Expr {
    Num(n)
}

pub fn ttrue() -> Expr {
    Boolean(true)
}
pub fn tfalse() -> Expr {
    Boolean(false)
}

pub fn unit() -> Expr {
    Unit
}

pub fn add(a: Expr, b: Expr) -> Expr {
    Add(Box::new(a), Box::new(b))
}

pub fn sub(a: Expr, b: Expr) -> Expr {
    Sub(Box::new(a), Box::new(b))
}

pub fn mul(a: Expr, b: Expr) -> Expr {
    Mul(Box::new(a), Box::new(b))
}

pub fn lessthan(a: Expr, b: Expr) -> Expr {
    LessThan(Box::new(a), Box::new(b))
}

pub fn and(a: Expr, b: Expr) -> Expr {
    And(Box::new(a), Box::new(b))
}

pub fn or(a: Expr, b: Expr) -> Expr {
    Or(Box::new(a), Box::new(b))
}

pub fn not(a: Expr) -> Expr {
    Not(Box::new(a))
}

pub fn get(a: Expr, i: usize) -> Expr {
    Get(Box::new(a), i)
}

pub fn concat(a: Expr, b: Expr) -> Expr {
    Concat(Box::new(a), Box::new(b))
}

pub fn print(a: Expr) -> Expr {
    Print(Box::new(a))
}

pub fn sequence(args: Vec<Expr>) -> Expr {
    All(Order::Sequential, args)
}

pub fn parallel(args: Vec<Expr>) -> Expr {
    All(Order::Parallel, args)
}

pub fn switch(arg: Expr, cases: Vec<Expr>) -> Expr {
    Switch(Box::new(arg), cases)
}

pub fn tloop(input: Expr, body: Expr) -> Expr {
    Loop(Id(0), Box::new(input), Box::new(body))
}

pub fn tlet(arg: Expr, body: Expr) -> Expr {
    Let(Id(0), Box::new(arg), Box::new(body))
}

pub fn arg() -> Expr {
    Arg(Id(0))
}

pub fn function(arg: Expr) -> Expr {
    Function(Id(0), Box::new(arg))
}

pub fn call(arg: Expr) -> Expr {
    Call(Id(0), Box::new(arg))
}

#[test]
fn test_gives_nested_ids() {
    let mut prog = tlet(num(0), tlet(num(1), num(2)));
    give_fresh_ids(&mut prog);
    assert_eq!(
        prog,
        Let(
            Id(1),
            Box::new(Num(0)),
            Box::new(Let(Id(2), Box::new(Num(1)), Box::new(Num(2))))
        )
    );
}

#[test]
fn test_gives_loop_ids() {
    let mut prog = tlet(num(0), tloop(num(1), num(2)));
    give_fresh_ids(&mut prog);
    assert_eq!(
        prog,
        Let(
            Id(1),
            Box::new(Num(0)),
            Box::new(Loop(Id(2), Box::new(Num(1)), Box::new(Num(2))))
        )
    );
}

#[test]
fn test_complex_program_ids() {
    // test a program that includes
    // a let, a loop, a switch, and a call
    let prog = program(vec![function(tlet(
        num(0),
        tloop(
            num(1),
            switch(
                arg(),
                vec![
                    num(2),
                    call(num(3)),
                    tlet(num(4), num(5)),
                    tloop(num(6), num(7)),
                ],
            ),
        ),
    ))]);
    assert_eq!(
        prog,
        Program(vec![Function(
            Id(1),
            Box::new(Let(
                Id(2),
                Box::new(Num(0)),
                Box::new(Loop(
                    Id(3),
                    Box::new(Num(1)),
                    Box::new(Switch(
                        Box::new(Arg(Id(3))),
                        vec![
                            Num(2),
                            Call(Id(3), Box::new(Num(3))),
                            Let(Id(4), Box::new(Num(4)), Box::new(Num(5))),
                            Loop(Id(5), Box::new(Num(6)), Box::new(Num(7))),
                        ]
                    ))
                ))
            ))
        )])
    );
}
