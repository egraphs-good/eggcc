#!/usr/bin/env python3
import glob
import os

import concurrent.futures
import subprocess

def make_cfgs(bench, data_dir, allowed_modes=None):
  """Generate CFG PNGs for a benchmark.
  If allowed_modes is not None, only process those run mode directory names.
  """
  print(f"Generating CFGs for {bench}", flush=True)
  bench_path = f"{data_dir}/{bench}"
  try:
    runmodes = os.listdir(bench_path)
  except FileNotFoundError:
    print(f"Bench path not found: {bench_path}")
    return
  if allowed_modes is not None:
    runmodes = [m for m in runmodes if m in allowed_modes]
  
  for mode in runmodes:
    path = f"{bench_path}/{mode}"
    # HACK: check if opt-18 exists
    # otherwise use opt
    # On Linux, sometimes it's called opt-18, while on mac it seems to be just opt
    # Also, on some machines, just running `opt-18` hangs, so we pass the version flag
    # Catch the output using shell
    opt18_res = subprocess.run("opt-18 --version", shell=True, capture_output=True)
    if opt18_res.returncode == 0:
      opt = "opt-18"
    else:
      opt = "opt"

    # https://llvm.org/docs/Passes.html#dot-cfg-print-cfg-of-function-to-dot-file
    # spawn a shell in the path and run opt
    opt_res = subprocess.run(f"{opt} -disable-output -passes=dot-cfg optimized.ll", shell=True, cwd=path, capture_output=True)
    if opt_res.returncode != 0:
      print(f"Error running opt on {path}/optimized.ll")
      continue
      

    # Find all the dot files (can't use glob because it doesn't match hidden files)
    # There are also a bunch of files that start with ._Z that I don't think we care about?
    dots = [f for f in os.listdir(f"{path}") if f.endswith(".dot") and not f.startswith("._Z") and not f.startswith("._bril")]
    for dot in dots:
      parts = dot.split(".")
      if len(parts) < 2:
        continue
      name = parts[1]

      # Convert to svg
      cmd = f"dot -Tsvg -o {path}/{name}.svg {path}/{dot}"
      dot_res = subprocess.run(cmd, shell=True, capture_output=True).returncode
      if dot_res != 0:
        print(f"Error converting {dot} to svg")
        exit(1)

    svgs = glob.glob(f"{path}/*.svg")
    svgs_names = [os.path.basename(svg) for svg in svgs]
    print(f"Generated {len(svgs)} CFGs for {bench} {mode}")
    with open(f"{path}/svg_names.txt", "w") as f:
      f.write("\n".join(svgs_names))

    # Clean up dot files
    os.system(f"rm {path}/.*.dot")

