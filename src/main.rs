use clap::Parser;
use eggcc::util::{visualize, InterpMode, Run, RunType, TestProgram};
use std::{ffi::OsStr, path::PathBuf};

#[derive(Debug, Parser)]
struct Args {
    /// A directory for debug output, including
    /// svgs for the rvsdg, cfgs, ect.
    #[clap(long)]
    debug_dir: Option<PathBuf>,
    /// Configure the output of the tool, which can be an optimized bril program,
    /// an optimized CFG, or more.
    /// See documentation for [`RunType`] for different options.
    #[clap(long, default_value_t = RunType::Optimize)]
    run_mode: RunType,
    /// Evaluate the resulting program and output
    /// the result.
    #[clap(long)]
    interp: bool,
    #[clap(long)]
    profile_out: Option<PathBuf>,

    /// The bril program to optimize
    file: PathBuf,
    /// The arguments to the bril program
    /// (only used when interpreting)
    bril_args: Vec<String>,

    /// Where to put the executable (only for the CompileBrilfit run mode)
    /// If not provided, the executable will be in a file with the same prefix as the
    /// input file, but with no file extension. That is, if `abc.bril` is passed in,
    /// then the executable will be in `abc`.
    #[clap(short)]
    output_path: Option<String>,
    /// Where to put intermediary files (only for OptimizeBrilLLVM mode)
    #[clap(long)]
    llvm_output_dir: Option<String>,
    /// Run the eggcc optimizer (only for the CompileBrilfit run mode)
    /// Defaults to true.
    #[clap(long)]
    optimize_egglog: Option<bool>,
    /// Run the brilift optimizer (only for the CompileBrilfit run mode)
    /// Defaults to false.
    #[clap(long)]
    optimize_brilift: Option<bool>,
    /// Run the LLVM optimizer (only for the CompileBrilLLVM run mode)
    /// Defaults to false.
    #[clap(long)]
    optimize_bril_llvm: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // enable logging
    env_logger::init();

    if let Some(debug_dir) = args.debug_dir {
        if let Result::Err(error) = visualize(TestProgram::BrilFile(args.file.clone()), debug_dir) {
            eprintln!("{}", error);
            return;
        }
    }

    if args.interp && !args.run_mode.produces_interpretable() {
        eprintln!(
            "Cannot interpret run type {} because it doesn't produce a bril program.",
            args.run_mode
        );
        return;
    }

    let file = match args.file.extension().and_then(OsStr::to_str) {
        Some("rs") => TestProgram::RustFile(args.file.clone()),
        Some("bril") => TestProgram::BrilFile(args.file.clone()),
        Some(x) => panic!("unexpected file extension {x}"),
        None => panic!("could not parse file extension"),
    };

    let run = Run {
        prog_with_args: file.read_program(),
        test_type: args.run_mode,
        interp: if args.interp {
            InterpMode::Interp
        } else {
            InterpMode::None
        },
        profile_out: args.profile_out,
        output_path: args.output_path,
        llvm_output_dir: args.llvm_output_dir,
        optimize_egglog: args.optimize_egglog,
        optimize_brilift: args.optimize_brilift,
        optimize_bril_llvm: args.optimize_bril_llvm,
    };

    let result = match run.run() {
        Ok(result) => result,
        Err(error) => {
            panic!("{}", error);
        }
    };

    if args.interp {
        // just print out the result of interpreting the program
        println!("{}", result.result_interpreted.unwrap());
    } else if let &[visualization] = &result.visualizations.as_slice() {
        // when there is just one visualization, print it out without
        // the "visualization of: {}" header for convenience
        println!("{}", visualization.result);
    } else {
        // otherwise, print out each visualization with a header
        for visualization in result.visualizations {
            println!("visualization of {}:", visualization.name);
            println!("{}", visualization.result);
            println!();
        }
    }
}
