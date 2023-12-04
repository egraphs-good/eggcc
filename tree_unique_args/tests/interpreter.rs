// This file is a reference for the semantics of tree_unique_args
// WIP
#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

pub enum Order {
    Parallel,
    Sequential,
}
pub struct Id {
    id: i64,
}
pub enum Expr {
    Num(i32),
    Boolean(bool),
    Unit,
    Badd(Box<Expr>, Box<Expr>),
    // TODO: other pure ops
    Get(Box<Expr>, usize),
    Print(Box<Expr>),
    Read(Box<Expr>),
    Write(Box<Expr>, Box<Expr>),
    All(Order, Vec<Expr>),
    Switch(Box<Expr>, Vec<Expr>),
    Loop(Id, Box<Expr>, Box<Expr>),
    Body(Id, Box<Expr>, Box<Expr>),
    Arg(Id),
    // TODO: call and functions
}

pub enum Value {
    Num(i32),
    Boolean(bool),
    Unit,
    Tuple(Vec<Value>),
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Num,
    Boolean,
    Unit,
    Tuple(Vec<Type>),
}

pub fn typecheck(e: &Expr) -> Result<Type, &'static str> {
    match e {
        Expr::Num(_) => Ok(Type::Num),
        Expr::Boolean(_) => Ok(Type::Boolean),
        Expr::Unit => Ok(Type::Unit),
        Expr::Badd(e1, e2) => {
            let ty1 = typecheck(&*e1)?;
            let ty2 = typecheck(&*e2)?;
            if ty1 == Type::Num && ty2 == Type::Num {
                Ok(Type::Num)
            } else {
                Err("badd")
            }
        }
        Expr::Get(tuple, i) => {
            let ty_tuple = typecheck(&*tuple)?;
            match ty_tuple {
                Type::Tuple(tys) => Ok(tys[*i].clone()),
                _ => Err("get"),
            }
        }
        Expr::Print(e) => {
            // right now, any value can be printed
            typecheck(&*e)?;
            Ok(Type::Unit)
        }
        Expr::Read(addr) => {
            let ty_addr = typecheck(&*addr)?;
            if ty_addr == Type::Num {
                // right now, all memory holds nums.
                // read could also take a static type to interpret
                // the memory as.
                Ok(Type::Num)
            } else {
                Err("read")
            }
        }
        Expr::Write(addr, data) => {
            let ty_addr = typecheck(&*addr)?;
            let ty_data = typecheck(&*data)?;
            if ty_addr == Type::Num && ty_data == Type::Num {
                Ok(Type::Unit)
            } else {
                Err("write")
            }
        }
        Expr::All(_, exprs) => {
            let mut tys: Vec<Type> = vec![];
            for expr in exprs {
                let ty = typecheck(&*expr)?;
                tys.push(ty)
            }
            Ok(Type::Tuple(tys))
        }
        _ => Err("unimplemented"),
    }
}

pub fn interpret(e: Expr, mem: &HashMap<i64, Value>) -> Value {
    Value::Num(0)
}

#[test]
fn test_interpreter() {}
