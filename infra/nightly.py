#!/usr/bin/env python3
"""
Runs profile.py, graphs.py, and generate_line_counts.py in sequence to produce the data and plots for the nightly paper.
Moves the html over to the output folder, and gzips all JSON and SVG files for upload to the nightly-results server.

Use the --update flag to only update the front end (output folder) without re-running the benchmarking. This is useful for quickly iterating on the HTML/CSS/JS without waiting benchmarking.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def run_cmd(cmd, cwd=None):
    """Run a command and exit on failure."""
    print(f"+ {cmd}")
    result = subprocess.run(cmd, shell=True, cwd=cwd)
    if result.returncode != 0:
        print(f"Command failed with exit code {result.returncode}")
        sys.exit(result.returncode)

def main():
    print("Beginning eggcc nightly script...")

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

    # Check environment
    is_local = os.environ.get("LOCAL", "") != ""
    args = sys.argv[1:]
    update_only = "--update" in args

    # Make sure we're in the right place
    os.chdir(script_dir)
    print(f"Switching to nightly script directory: {script_dir}")

    # Setup rustup and tokei if not local
    if not is_local:
        run_cmd("rustup update")
        run_cmd("cargo install tokei")

    # Clean previous nightly run
    if update_only:
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

    # Run profiler
    if update_only:
        print("Skipping profile.py, updating front end")
    elif is_local:
        # In local mode, pass all arguments to profile.py
        args_str = " ".join(f'"{arg}"' for arg in args)
        run_cmd(f'./infra/profile.py "{data_dir}" {args_str} 2>&1 | tee "{log_file}"')
    else:
        os.environ["LLVM_SYS_180_PREFIX"] = "/usr/lib/llvm-18/"
        run_cmd("make runtime")
        # Run on all benchmarks in nightly
        run_cmd(f'./infra/profile.py "{data_dir}" benchmarks/passing 2>&1 | tee "{log_file}"')

    # Generate the plots
    run_cmd(f'./infra/graphs.py "{output_dir}" "{paper_dir}" "{profile_json}" benchmarks/passing 2>&1 | tee "{log_file}"')

    # Generate latex after running the profiler (depends on profile.json)
    run_cmd(f'./infra/generate_line_counts.py "{data_dir}" 2>&1 | tee "{log_file}"')

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
        shutil.copytree(data_dir, output_data_dir, dirs_exist_ok=True)

    # Gzip all JSON and SVGs in the nightly dir (only in non-local mode)
    if not is_local:
        if profile_json.exists():
            run_cmd(f'gzip "{profile_json}"')
        run_cmd(f'find "{output_dir}" -name "*.svg" -exec gzip {{}} +')
        run_cmd(f'find "{output_dir}" -name "*.ll" -exec gzip {{}} +')

    print("Nightly script completed successfully!")

if __name__ == "__main__":
    main()
