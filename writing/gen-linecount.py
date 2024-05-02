#!/usr/bin/env python3

import subprocess
import json
import glob

def get_generated_egg():
    program = subprocess.run(["cargo", "run", "--quiet"], cwd="../dag_in_context", capture_output=True)
    return len(program.stdout.splitlines())

def get_written_egg():
    total_lines = 0
    file_paths = glob.glob("../../**/*.egg", recursive=True)
    for file_path in file_paths:
        with open(file_path, 'r') as file:
            lines = file.readlines()
            total_lines += len(lines)
    return total_lines

def get_rust_lines():
    rust_lines_output = subprocess.run(["tokei", "--output", "json", "../../"], capture_output=True)
    return json.loads(rust_lines_output.stdout)["Rust"]["code"]

def main():
    rust_lines = str(get_rust_lines())
    written_egg = str(get_written_egg())
    generated_egg = str(get_generated_egg())

    fmt = """\\begin{tabular}{ |s|p{2cm}| }
\hline
\multicolumn{2}{|c|}{Line Counts} \\\
\hline
Language & \# Lines  \\\\
\hline
Rust & %s \\\\
Written Egg & %s \\\\
Generated EGG & %s \\\\
\hline
\end{tabular}""" % (rust_lines, written_egg, generated_egg)
    print(fmt)


main()


