from glob import glob
import os
import subprocess

"""
Generates line counts for bril/rs files. Depends on having rs2bril and bril2txt installed
(see https://github.com/uwplse/bril/tree/main/bril-rs/rs2bril)
"""


def count_code_lines(contents):
    """Line count with comments and empty lines removed."""
    lines = [
        line
        for line in contents.split("\n")
        if line.strip() and not line.strip().startswith("#")
    ]
    return len(lines)


def line_count(name, path):
    output = ""
    if path.endswith(".rs"):
        # if Rust, first compile to bril
        bril_json = subprocess.check_output(["rs2bril", "-f", path])
        output = subprocess.check_output(["bril2txt"], input=bril_json).decode("utf-8")
    elif path.endswith(".bril"):
        with open(path, "r") as f:
            output = f.read()
    else:
        raise Exception(f"Invalid program type : {path}")

    return count_code_lines(output)


def main(bril_dir):
    programs = []
    # if it is a directory get all files
    if os.path.isdir(bril_dir):
        programs = glob(f"{bril_dir}/**/*.bril", recursive=True) + glob(
            f"{bril_dir}/**/*.rs", recursive=True
        )
    else:
        programs = [bril_dir]

    paths = {}
    for program in programs:
        bench_name_parts = program.split("/")[-1].split(".")
        if len(bench_name_parts) != 2:
            raise Exception(f"Invalid benchmark name: {program}")
        name = bench_name_parts[0]
        paths[name] = program

    line_counts = {}
    for name, path in paths.items():
        line_counts[name] = line_count(name, path)

    return line_counts


if __name__ == "__main__":
    line_counts = main("tests/passing/small")
    print(line_counts)
