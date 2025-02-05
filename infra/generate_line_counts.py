#!/usr/bin/env python3

import glob
import json
import subprocess
import os
import statistics 

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

def round_fmt(v):
    return "{:.3f}".format(round(v, 3))

# given a list of numbers, compute the mean
# numbers may be floating-point or integers (for cycles)
def mean_cycles(cycles):
    return sum(cycles) / len(cycles)

# given a list of integers, return the max
def max_cycles(cycles):
    return max(cycles)

def min_cycles(cycles):
    return min(cycles)


# given a list of integers, return the standard deviation
def stddev_cycles(cycles):
    return statistics.stdev(cycles)

def get_rows_for_benchmark(bench, profile_data):
    data_for_bench = [x for x in profile_data if x["benchmark"] == bench]
    rows = []
    for (idx, entry) in enumerate(data_for_bench):
        fst_col = r'\multirow{' + str(len(data_for_bench)) + r'}{*}{' + bench.replace("_", r'\_') + r'}' if idx == 0 else ''
        cycles = entry["cycles"]
        row = " ".join([
            r'\multicolumn{1}{|l|}{' + fst_col + r'} &',
            r'\multicolumn{1}{l|}{' + entry["runMethod"] + r'}  &',
            r'\multicolumn{1}{l|}{' + round_fmt(mean_cycles(cycles)) + r'} &',
            r'\multicolumn{1}{l|}{' + round_fmt(max_cycles(cycles)) + r'} &',
            r'\multicolumn{1}{l|}{' + round_fmt(min_cycles(cycles)) + r'} &',
            round_fmt(stddev_cycles(cycles)) + r' \\',
        ])
        rows.append(row)
    rows.append(r' \hline')
    return rows


def generate_latex(output_path):
    with open(f'{output_path}/linecount.tex', "w") as f:
        f.write(linecount_table())
    with open(f'{output_path}/detailed-linecount.tex', "w") as f:
        f.write(detailed_linecount_table()) 
    tex_files = glob.glob(f"{output_path}/*.tex")
    for tex in tex_files:
        cmd = " ".join(["pdflatex", f"-output-directory {output_path}/", tex, "> /dev/null 2>&1"])
        os.system(cmd)


if __name__ == '__main__':
    # expect a single argument
    if len(os.sys.argv) != 2:
        print("Usage: generate_line_counts.py <output_directory>")
        exit(1)

    output_path = os.sys.argv[1]
    generate_latex(output_path)
