#!/usr/bin/env python3
"""
Runs profile.py, graphs.py, and generate_line_counts.py in sequence to produce the data and plots for the nightly paper.
Moves the html over to the output folder, and gzips all JSON and SVG files for upload to the nightly-results server.
"""

import argparse
import os
import sys
import subprocess
import shutil
from pathlib import Path

from profile import NightlyConfig, run_profile
from graphs import make_graphs
from generate_line_counts import generate_latex

class TeeWriter:
    """Write to both a file and the original stream."""
    def __init__(self, file, stream):
        self.file = file
        self.stream = stream
    
    def write(self, data):
        self.stream.write(data)
        self.stream.flush()
        self.file.write(data)
        self.file.flush()
    
    def flush(self):
        self.stream.flush()
        self.file.flush()

def run_cmd(cmd, cwd=None):
    """Run a command and exit on failure."""
    print(f"+ {cmd}")
    result = subprocess.run(cmd, shell=True, cwd=cwd)
    if result.returncode != 0:
        print(f"Command failed with exit code {result.returncode}")
        sys.exit(result.returncode)

def run_nightly(args, config, top_dir, script_dir, resource_dir, nightly_dir, output_dir, 
                paper_dir, data_dir, output_data_dir, profile_json, is_local):
    """Main nightly workflow - run profiler, generate graphs, and package output."""
    # Run profiler
    if args.update:
        print("Skipping profile.py, updating front end")
    else:
        if not is_local:
            os.environ["LLVM_SYS_180_PREFIX"] = "/usr/lib/llvm-18/"
            run_cmd("make runtime")
        
        print(f"Running profile with data_dir={data_dir}, bril_dir={args.benchmark_dir}, parallel={args.parallel}")
        run_profile(str(data_dir), args.benchmark_dir, config, parallel=args.parallel)

    # Generate the plots
    print("Generating graphs...")
    make_graphs(str(output_dir), str(paper_dir), str(profile_json), "benchmarks/passing", config)

    # Generate latex after running the profiler (depends on profile.json)
    print("Generating line counts...")
    generate_latex(str(data_dir))

    os.chdir(script_dir)

    # Update HTML index page
    if resource_dir.exists():
        for item in resource_dir.iterdir():
            dest = output_dir / item.name
            if item.is_dir():
                shutil.copytree(item, dest, dirs_exist_ok=True)
            else:
                shutil.copy2(item, dest)

    # Copy data over to output
    if data_dir.exists():
        # in local mode copy over
        if is_local:
            shutil.copytree(data_dir, output_data_dir, dirs_exist_ok=True)
        else:
            # otherwise move
            shutil.move(str(data_dir), str(output_data_dir))

    # Gzip all JSON and SVGs in the nightly dir (only in non-local mode)
    if not is_local:
        if profile_json.exists():
            run_cmd(f'gzip "{profile_json}"')
        run_cmd(f'find "{output_dir}" -name "*.svg" -exec gzip {{}} +')
        run_cmd(f'find "{output_dir}" -name "*.ll" -exec gzip {{}} +')

    print("Nightly script completed successfully!")

def main():
    parser = argparse.ArgumentParser(
        description="Run eggcc nightly benchmarks and generate reports."
    )
    parser.add_argument(
        "benchmark_dir",
        nargs="?",
        default="benchmarks/passing",
        help="Directory or file containing benchmarks to run (default: benchmarks/passing)"
    )
    parser.add_argument(
        "--update",
        action="store_true",
        help="Only update the front end (output folder) without re-running benchmarks"
    )
    parser.add_argument(
        "--paper",
        action="store_true",
        help="Production mode: 100%% regions, Gurobi enabled, many samples"
    )
    parser.add_argument(
        "--use-gurobi",
        action="store_true",
        help="Enable Gurobi treatments without full paper mode"
    )
    parser.add_argument(
        "--parallel",
        action="store_true",
        help="Run benchmarks in parallel (results may have higher variance)"
    )
    parser.add_argument(
        "--local",
        action="store_true",
        help="Local mode: skip rustup update and tokei install"
    )
    
    args = parser.parse_args()

    print("Beginning eggcc nightly script...")

    # Use the flag instead of environment variable
    is_local = args.local

    # Set up PATH for cargo/rustup (needed before running rustup/cargo commands)
    home_dir = os.path.expanduser("~")
    cargo_bin = os.path.join(home_dir, ".cargo", "bin")
    if cargo_bin not in os.environ.get("PATH", ""):
        os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"

    # Install/update rustup and tokei (skip in local mode)
    if not is_local:
        print("Updating rustup...")
        run_cmd("rustup update")
        print("Installing tokei...")
        # Install tokei v13.0.0 which works with Rust 1.87
        run_cmd("cargo install tokei --version 13.0.0 --locked")

    # Build config - paper mode implies use_gurobi
    use_gurobi = args.paper or args.use_gurobi
    config = NightlyConfig(paper_mode=args.paper, use_gurobi=use_gurobi)

    # Determine directories
    script_dir = Path(__file__).resolve().parent
    top_dir = script_dir.parent
    resource_dir = script_dir / "nightly-resources"

    # Nightly run directories
    nightly_dir = top_dir / "nightly"
    output_dir = nightly_dir / "output"
    paper_dir = output_dir / "paper"
    data_dir = nightly_dir / "data"
    output_data_dir = output_dir / "data"
    llvm_dir = data_dir / "llvm"
    log_file = output_dir / "log.txt"
    profile_json = data_dir / "profile.json"

    # Make sure we're in the right place
    os.chdir(script_dir)
    print(f"Switching to nightly script directory: {script_dir}")

    # Clean previous nightly run
    if args.update:
        print("Updating front end only (output folder) due to --update flag")
        if output_dir.exists():
            shutil.rmtree(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        paper_dir.mkdir(parents=True, exist_ok=True)
    else:
        if nightly_dir.exists():
            shutil.rmtree(nightly_dir)
        nightly_dir.mkdir(parents=True, exist_ok=True)
        output_dir.mkdir(parents=True, exist_ok=True)
        data_dir.mkdir(parents=True, exist_ok=True)
        llvm_dir.mkdir(parents=True, exist_ok=True)
        paper_dir.mkdir(parents=True, exist_ok=True)

    os.chdir(top_dir)

    # Set up logging to both console and file
    log_file_handle = open(log_file, 'w')
    original_stdout = sys.stdout
    original_stderr = sys.stderr
    sys.stdout = TeeWriter(log_file_handle, original_stdout)
    sys.stderr = TeeWriter(log_file_handle, original_stderr)

    try:
        run_nightly(args, config, top_dir, script_dir, resource_dir, nightly_dir, 
                    output_dir, paper_dir, data_dir, output_data_dir, profile_json, is_local)
    finally:
        # Restore original stdout/stderr and close log file
        sys.stdout = original_stdout
        sys.stderr = original_stderr
        log_file_handle.close()

if __name__ == "__main__":
    main()
