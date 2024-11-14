#!/usr/bin/env python3
import glob
import os

import concurrent.futures
import subprocess

def make_cfgs(bench, data_dir):
  cwd = os.getcwd()
  path = f"{data_dir}/{bench}"
  runmodes = os.listdir(path)
  for mode in runmodes:
    os.chdir(f"{path}/{mode}")

    # HACK: check if opt-18 exists
    # otherwise use opt
    # On Linux, sometimes it's called opt-18, while on mac it seems to be just opt
    # Also, on some machines, just running `opt-18` hangs, so we pass the version flag
    # Catch the output using shell
    opt18_res = subprocess.run("opt-18", shell=True, capture_output=True, text=True)
    if opt18_res.returncode == 0:
      opt = "opt-18"
    else:
      opt = "opt"

    # https://llvm.org/docs/Passes.html#dot-cfg-print-cfg-of-function-to-dot-file
    for filename in glob.glob("*.ll"):
      if "init" in filename:
        # Delete the -init.ll file (We don't need it for nightly,
        # so just reduce the amount of clutter we copy to the nightly machine)
        os.system(f"rm {filename}")
      else:
        os.system(f"{opt} -disable-output -passes=dot-cfg {filename}")

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
