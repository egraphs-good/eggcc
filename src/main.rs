use clap::Parser;
use dag_in_context::{EggccConfig, Schedule};
use eggcc::util::{visualize, InterpMode, LLVMOptLevel, Run, RunMode, TestProgram};
use std::{ffi::OsStr, i64, iter::once, path::PathBuf};

#[derive(Debug, Parser)]
struct Args {
    /// A directory for debug output, including
    /// svgs for the rvsdg, cfgs, ect.
    #[clap(long)]
    debug_dir: Option<PathBuf>,
    /// Configure the output of the tool, which can be an optimized bril program,
    /// an optimized CFG, or more.
    /// See documentation for [`RunType`] for different options.
    #[clap(long, default_value_t = RunMode::Optimize)]
    run_mode: RunMode,
    /// Evaluate the resulting program and output
    /// the result.
    #[clap(long)]
    interp: bool,
    /// Add timing information to the benchmark, measuring cycles before the final print statement.
    #[clap(long)]
    add_timing: bool,
    #[clap(long)]
    profile_out: Option<PathBuf>,

    /// The bril program to optimize
    file: PathBuf,
    /// The arguments to the bril program
    /// (only used when interpreting)
    bril_args: Vec<String>,

    /// Where to put the executable (only for the brillift and llvm modes)
    #[clap(short)]
    output_path: Option<String>,
    /// Output metadata about the run to a file
    #[clap(long)]
    run_data_out: Option<PathBuf>,
    /// Where to put the optimized llvm file (for the llvm mode)
    #[clap(long)]
    llvm_output_dir: Option<PathBuf>,
    /// For the LLVM run mode, choose whether to first run eggcc
    /// to optimize the bril program before going to LLVM.
    #[clap(long)]
    optimize_egglog: Option<bool>,
    /// For the Cranelift run mode, choose between O0 optimization and O3.
    #[clap(long)]
    optimize_brilift: Option<bool>,
    /// For the LLVM run mode, choose between O0 and O3.
    #[clap(long)]
    optimize_bril_llvm: Option<LLVMOptLevel>,
    /// For the eggcc schedule, choose between the sequential and parallel schedules.
    #[clap(long)]
    eggcc_schedule: Option<Schedule>,
    /// Eggcc by default performs several passes.
    /// This argument specifies how many passes to run (all passes by default).
    /// If stop_after_n_passes is negative,
    /// run [0 ... schedule.len() + stop_after_n_passes] passes.
    ///
    /// This flag also works with `--run-mode egglog` mode,
    /// where it prints the egglog program being processed by the last pass
    /// this flag specifies.
    #[clap(long)]
    stop_after_n_passes: Option<i64>,

    /// Turn off enforcement that the output program uses
    /// memory linearly. This can give an idea of what
    /// extraction is doing.
    /// WARNING: Produces unsound results!
    #[clap(long)]
    no_linearity: bool,

    #[clap(long)]
    optimize_function: Option<String>,
}

fn main() {
    let args = Args::parse();

    // enable logging
    env_logger::init();

    let start_time = std::time::Instant::now();

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
        optimized_llvm_out: args.llvm_output_dir,
        optimize_egglog: args.optimize_egglog,
        optimize_brilift: args.optimize_brilift,
        optimize_bril_llvm: args.optimize_bril_llvm,
        add_timing: args.add_timing,
        eggcc_config: EggccConfig {
            schedule: args.eggcc_schedule.unwrap_or(Schedule::default()),
            stop_after_n_passes: args.stop_after_n_passes.unwrap_or(i64::MAX),
            linearity: !args.no_linearity,
            optimize_functions: args.optimize_function.map(|s| once(s.clone()).collect()),
        },
    };

    let mut result = match run.run() {
        Ok(result) => result,
        Err(error) => {
            panic!("{}", error);
        }
    };

    let eggcc_duration = start_time.elapsed();
    result.eggcc_compile_time = eggcc_duration;

    if let Some(run_data_output_path) = args.run_data_out {
        let file = std::fs::File::create(run_data_output_path).unwrap();
        serde_json::to_writer_pretty(file, &result).unwrap();
    }

    if args.interp {
        // just print out the result of interpreting the program
        println!("{}", result.result_interpreted.unwrap());
        if let Some(cycles_taken) = result.cycles_taken {
            eprintln!("{}", cycles_taken);
        }
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
