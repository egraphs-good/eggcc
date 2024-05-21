#!/usr/bin/env python3

import glob
import json
import subprocess
import os

def get_generated_egg():
    program = subprocess.run(["cargo", "run", "--quiet"], cwd="./dag_in_context", capture_output=True)
    return len(program.stdout.splitlines())

def get_written_egg():
    file_paths = glob.glob("./**/*.egg", recursive=True)

    counts = {}
    for file_path in file_paths:
        with open(file_path, 'r') as file:
            counts[file_path] = len(file.readlines())
    return counts


def get_rust_lines():
    rust_lines_output = subprocess.run(["tokei", "--output", "json", "./"], capture_output=True)
    return json.loads(rust_lines_output.stdout)["Rust"]["code"]

if __name__ == '__main__':
    # expect a single argument
    if len(os.sys.argv) != 2:
        print("Usage: generate_line_counts.py <output_directory>")
        exit(1)
    
    output_path = os.sys.argv[1]
    data = {
        "rust": get_rust_lines(),
        "gen_egg": get_generated_egg(),
        "written_egg": get_written_egg()
    }
    with open(f"{output_path}/data/latex.json", "w") as f:
        json.dump(data, f, indent=2)