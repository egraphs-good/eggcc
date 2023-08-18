use clap::Parser;
use eggcc::*;
use std::{
    io::{stdin, Read},
    path::PathBuf,
};

#[derive(Debug, Parser)]
struct Args {
    /// Output the SSA form of the bril program
    #[clap(long)]
    ssa: bool,
    /// Output the structured form of the bril program,
    /// which uses blocks, loops, and break
    #[clap(long)]
    structured: bool,
    /// Convert the program to a structured cfg, and
    /// output it as a bril program
    #[clap(long)]
    structured_cfg: bool,
    /// Output the egglog encoding of the program.
    /// The egglog program can be run to optimize the program.
    #[clap(long, verbatim_doc_comment)]
    egglog_encoding: bool,
    /// After optimization, output the structured form of the program
    /// using blocks, loops, and break
    #[clap(long)]
    optimized_structured: bool,
    /// The bril program to optimize
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
    } else if args.structured_cfg {
        println!(
            "{}",
            Optimizer::parse_to_structured(&input).unwrap().to_program()
        );
    } else if args.egglog_encoding {
        let structured = Optimizer::parse_to_structured(&input).unwrap();
        let mut optimizer = Optimizer::default();
        println!("{}", optimizer.structured_to_optimizer(&structured));
    } else if args.optimized_structured {
        let mut optimizer = Optimizer::default();
        let optimized_structured =
            optimizer.optimized_structured(&Optimizer::parse_bril(&input).unwrap());
        println!("{}", optimized_structured.unwrap());
    } else {
        let mut optimizer = Optimizer::default();
        match optimizer.parse_and_optimize(&input) {
            Ok(expr) => println!("{}", expr),
            Err(err) => println!("{}", err),
        }
    }
}
