use clap::Parser;
use eggcc::*;
use std::{
    io::{stdin, Read},
    path::PathBuf,
};

#[derive(Debug, Parser)]
struct Args {
    #[clap(long)]
    ssa: bool,
    #[clap(long)]
    structured: bool,
    #[clap(long)]
    egglog_encoding: bool,
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let mut input = String::new();
    if args.file.to_str() == Some("-") {
        stdin().read_to_string(&mut input).unwrap();
    } else {
        input = std::fs::read_to_string(args.file).unwrap();
    }

    if args.ssa {
        println!("{}", Optimizer::parse_bril(&input).unwrap());
    } else if args.structured {
        println!("{}", Optimizer::parse_to_structured(&input).unwrap());
    } else if args.egglog_encoding {
        let structured = Optimizer::parse_to_structured(&input).unwrap();
        let mut optimizer = Optimizer::default();
        println!("{}", optimizer.structured_to_optimizer(&structured));
    } else {
        let mut optimizer = Optimizer::default();
        match optimizer.parse_and_optimize(&input) {
            Ok(expr) => println!("{}", expr),
            Err(err) => println!("{}", err),
        }
    }
}
