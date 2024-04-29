#! /bin/python3

import subprocess
import json
import glob
import shutil

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
    rust_lines_output = subprocess.run(["scc", "--format", "json", "../../"], capture_output=True)
    return next(filter(lambda lang: lang["Name"] == "Rust", json.loads(rust_lines_output.stdout)))["Code"]

def replace_in_text(file_path, replacements):
    with open(file_path, 'r') as file:
        text = file.read()

    for key, value in replacements.items():
        text = text.replace(key, value)

    with open(file_path, 'w') as file:
        file.write(text)


def main():
    rust_lines = str(get_rust_lines())
    written_egg = str(get_written_egg())
    generated_egg = str(get_generated_egg())

    shutil.copy('tmpls/linecount.tex', 'linecount.tex')
    replace_in_text('linecount.tex', {'RUSTLINES': rust_lines, 'WRITTEN_EGG': written_egg, 'GENERATED_EGG': generated_egg})


main()


