#!/usr/bin/env python3

import glob
import json
import subprocess
import os

def header():
    return [
        r"\documentclass{article}",
        r"\usepackage{graphicx} % Required for inserting images",
        r"\usepackage{amsmath}",
        r"\usepackage{amsfonts}",
        r"\usepackage{setspace} \setstretch{1}",
        r"\usepackage{multirow}",
        r"\usepackage{semantic}",
        r"\begin{document}",
    ]

def footer():
    return [r"\end{document}"]

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

def linecount_table():
    rows = header()
    rows += [
        r'\begin{tabular}{ |l|l| }',
        r'\hline',
        r'\multicolumn{2}{|c|}{Line Counts} \\',
        r'\hline',
        r'Language & \# Lines \\',
         r'\hline',
        fr'Rust & {get_rust_lines()} \\',
        fr'Written Egg & {sum(get_written_egg().values())} \\',
        fr'Generated EGG & {get_generated_egg()} \\',
        r'\hline',
        r'\end{tabular}',
    ]
    rows += footer()
    return "\n".join(rows)

def detailed_linecount_table():
    rows = header()
    rows += [
        r'\begin{tabular}{ |l|l| }',
        r'\hline',
        r'\multicolumn{2}{|c|}{Egglog Line Counts} \\',
        r'\hline',
        r'File & \# Lines \\',
    ]
    counts = get_written_egg()
    rows += [r'\hline {file} & {lines} \\'.format(file=file.split("/")[-1].replace("_", r'\_'),lines=lines)
            for (file, lines) in counts.items()]
    rows += [
        r'\hline',
        r'\end{tabular}',
    ]
    rows += footer()
    return "\n".join(rows)

def generate_latex(output_path):
    with open(f'{output_path}/data/linecount.tex', "w") as f:
        f.write(linecount_table())
    with open(f'{output_path}/data/detailed-linecount.tex', "w") as f:
        f.write(detailed_linecount_table()) 
    tex_files = glob.glob(f"{output_path}/data/*.tex")
    for tex in tex_files:
        cmd = " ".join(["pdflatex", f"-output-directory {output_path}/data/", tex])
        os.system(cmd)


if __name__ == '__main__':
    # expect a single argument
    if len(os.sys.argv) != 2:
        print("Usage: generate_line_counts.py <output_directory>")
        exit(1)

    output_path = os.sys.argv[1]
    generate_latex(output_path)
