use crate::{Expr, Expr::*, Id, Order};

impl Expr {
    /// Check that two expressions are the same ignoring their ids.
    /// To do this, simply assign them all new ids.
    /// If they are the same expression, they will get the same ids
    /// since `give_fresh_ids` is deterministic.
    pub fn eq_ignoring_ids(&self, other: &Expr) -> bool {
        let mut copy = other.clone();
        give_fresh_ids(&mut copy);
        self == &copy
    }

    /// Like [`Expr::eq_ignoring_ids`] but asserts
    /// that they are equal with a good error message.
    pub fn assert_eq_ignoring_ids(&self, other: &Expr) {
        let mut copy = other.clone();
        give_fresh_ids(&mut copy);
        if self != &copy {
            panic!(
                "assertion failed: `(left == right)`\n\
                 left:  `{:?}`\n\
                 right: `{:?}`\n",
                self, copy
            );
        }
    }
}

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

/// a macro that wraps the children in
/// a vec for program
#[macro_export]
macro_rules! program {
    ($($x:expr),*) => ($crate::ast::program_vec(vec![$($x),*]))
}
pub use program;

pub fn program_vec(args: Vec<Expr>) -> Expr {
    let mut res = Program(args);
    give_fresh_ids(&mut res);
    res
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

pub fn getarg(i: usize) -> Expr {
    get(arg(), i)
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

pub fn sequence_vec(args: Vec<Expr>) -> Expr {
    All(Id(0), Order::Sequential, args)
}

#[macro_export]
macro_rules! sequence {
    // use crate::ast::sequence_vec to resolve import errors
    ($($x:expr),*) => ($crate::ast::sequence_vec(vec![$($x),*]))
}
pub use sequence;

pub fn parallel_vec(args: Vec<Expr>) -> Expr {
    All(Id(0), Order::Parallel, args)
}

#[macro_export]
macro_rules! parallel {
    ($($x:expr),*) => ($crate::ast::parallel_vec(vec![$($x),*]))
}
pub use parallel;

#[macro_export]
macro_rules! switch {
    ($arg:expr, $($x:expr),*) => ($crate::ast::switch_vec($arg, vec![$($x),*]))
}
pub use switch;

pub fn switch_vec(arg: Expr, cases: Vec<Expr>) -> Expr {
    Switch(Box::new(arg), cases)
}

pub fn tloop(input: Expr, body: Expr) -> Expr {
    Loop(Id(0), Box::new(input), Box::new(body))
}

/// Let is reserved by rust, so we call it
/// tlet for tree-let
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
    let prog = program!(function(tlet(
        num(0),
        tloop(
            num(1),
            switch!(
                arg(),
                num(2),
                call(num(3)),
                tlet(num(4), num(5)),
                tloop(num(6), num(7))
            ),
        ),
    )));
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
