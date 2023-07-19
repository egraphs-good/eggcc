use std::io::{stdin, Read};

use eggcc::*;

fn main() {
    let mut input = String::new();
    stdin().lock().read_to_string(&mut input).unwrap();
    let mut optimizer = Optimizer::default();
    match optimizer.parse_and_optimize(&input) {
        Ok(expr) => println!("{}", expr),
        Err(err) => println!("{}", err),
    }
}
