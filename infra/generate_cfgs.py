#!/usr/bin/env python3
import glob
import os

import concurrent.futures
import subprocess

def make_cfgs(bench, data_dir):
  print(f"Generating CFGs for {bench}", flush=True)
  bench_path = f"{data_dir}/{bench}"
  runmodes = os.listdir(bench_path)
  
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
      exit(1)
      

    # Find all the dot files (can't use glob because it doesn't match hidden files)
    # There are also a bunch of files that start with ._Z that I don't think we care about?
    dots = [f for f in os.listdir(f"{path}") if f.endswith(".dot") and not f.startswith("._Z") and not f.startswith("._bril")]
    for dot in dots:
      name = dot.split(".")[1]

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



if __name__ == '__main__':
  # expect a single argument
  if len(os.sys.argv) != 2:
      print("Usage: generate_line_counts.py <data directory>")
      exit(1)
  data_dir = os.sys.argv[1]
  benchmarks = os.listdir(data_dir)

  # get the number of cores on this machine 
  parallelism = os.cpu_count()
  with concurrent.futures.ThreadPoolExecutor(max_workers = parallelism) as executor:
    futures = {executor.submit(make_cfgs, bench, data_dir) for bench in benchmarks}

    for future in concurrent.futures.as_completed(futures):
      try:
        future.result()
      except Exception as e:
        print(f"Shutting down executor due to error: {e}")
        executor.shutdown(wait=False, cancel_futures=True)
        raise e
