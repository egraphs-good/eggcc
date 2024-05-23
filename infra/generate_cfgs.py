#!/usr/bin/env python3
import glob
import os

def make_cfgs(bench, data_dir):
  cwd = os.getcwd()
  path = f"{data_dir}/{bench}"
  runmodes = os.listdir(path)
  for mode in runmodes:
    os.chdir(f"{path}/{mode}")

    # https://llvm.org/docs/Passes.html#dot-cfg-print-cfg-of-function-to-dot-file
    cmd = "opt -disable-output -passes=dot-cfg llvm.ll > /dev/null 2>&1"
    os.system(cmd)

    # Find all the dot files (can't use glob because it doesn't match hidden files)
    # There are also a bunch of files that start with ._Z that I don't think we care about?
    dots = [f for f in os.listdir(".") if f.endswith(".dot") and not f.startswith("._Z") and not f.startswith("._bril")]
    for dot in dots:
      name = dot.split(".")[1]

      # Convert to png
      cmd = f"dot -Tpng -o {name}.png {dot}"
      os.system(cmd)

    pngs = glob.glob("*.png")
    print(f"Generated {len(pngs)} CFGs for {bench} {mode}")
    with open("png_names.txt", "w") as f:
      f.write("\n".join(pngs))

    # Clean up dot files
    os.system("rm .*.dot")

    # Reset dir
    os.chdir(cwd)


if __name__ == '__main__':
  # expect a single argument
  if len(os.sys.argv) != 2:
      print("Usage: generate_line_counts.py <data directory>")
      exit(1)
  data_dir = os.sys.argv[1]
  benchmarks = os.listdir(data_dir)
  for bench in benchmarks:
    make_cfgs(bench, data_dir)
