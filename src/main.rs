use clap::Parser;
use eggcc::*;
use std::{
    io::{stdin, Read},
    path::PathBuf,
};

#[derive(Debug, Parser)]
struct Args {
    /// Don't perform optimization
    #[clap(long)]
    unoptimized: bool,
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
    /// Also evaluate the resulting program and output the results
    #[clap(long)]
    interp: bool,

    /// Path that eggcc will put interp profile results
    #[clap(long)]
    profile_out: Option<PathBuf>,

    /// The bril program to optimize
    file: PathBuf,
    /// The arguments to the bril program
    /// (only used when interpreting)
    bril_args: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let mut input = String::new();
    if args.file.to_str() == Some("-") {
        stdin().read_to_string(&mut input).unwrap();
    } else {
        input = std::fs::read_to_string(args.file).unwrap();
    }

    let bril_args = if args.bril_args.is_empty() {
        Optimizer::parse_bril_args(&input)
    } else {
        args.bril_args
    };

    let program = Optimizer::parse_bril(&input).unwrap();
    let result_program = if args.unoptimized {
        println!("{}", program);
        program
    } else if args.structured {
        let structured = Optimizer::parse_to_structured(&input).unwrap();
        println!("{}", structured);
        structured.to_program()
    } else if args.structured_cfg {
        let prog = Optimizer::parse_to_structured(&input).unwrap().to_program();
        println!("{}", prog);
        prog
    } else if args.egglog_encoding {
        let structured = Optimizer::parse_to_structured(&input).unwrap();
        let mut optimizer = Optimizer::default();
        println!("{}", optimizer.structured_to_optimizer(&structured));
        program
    } else if args.optimized_structured {
        let mut optimizer = Optimizer::default();
        let optimized_structured = optimizer
            .optimized_structured(&Optimizer::parse_bril(&input).unwrap())
            .unwrap();
        println!("{}", optimized_structured);
        optimized_structured.to_program()
    } else {
        let mut optimizer = Optimizer::default();
        let res = optimizer.optimize(&program).unwrap();
        println!("{}", res);
        res
    };

    if args.interp {
        println!(
            "{}",
            Optimizer::interp(&format!("{}", result_program), bril_args, args.profile_out)
        );
    }
}
