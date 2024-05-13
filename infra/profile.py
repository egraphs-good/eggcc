#!/usr/bin/env python3

import json
import os
import time
from glob import glob
from sys import stdout
import subprocess
from nightly_table import gen_nightly_table
from gen_linecount import gen_linecount_table


modes = [
  # (name, runmode, options)
  ("rvsdg_roundtrip", "rvsdg-round-trip-to-executable", ""),

  ("egglog_noopt_brilift_noopt", "compile-brilift", "--optimize-egglog false --optimize-brilift false"),
  ("egglog_noopt_brilift_opt", "compile-brilift", "--optimize-egglog false --optimize-brilift true"),
  ("egglog_opt_brilift_noopt", "compile-brilift", "--optimize-egglog true --optimize-brilift false"),
  ("egglog_opt_brilift_opt", "compile-brilift", "--optimize-egglog true --optimize-brilift true"),

  ("egglog_noopt_bril_llvm_noopt", "compile-bril-llvm", "--optimize-egglog false --optimize-bril-llvm false"),
  ("egglog_noopt_bril_llvm_opt", "compile-bril-llvm", "--optimize-egglog false --optimize-bril-llvm true"),
  ("egglog_opt_bril_llvm_noopt", "compile-bril-llvm", "--optimize-egglog true --optimize-bril-llvm false"),
  ("egglog_opt_bril_llvm_opt", "compile-bril-llvm", "--optimize-egglog true --optimize-bril-llvm true")
]

def bench(profile):
  # strip the path to just the file name
  profile_file = profile.split("/")[-1]

  # strip off the .bril to get just the profile name
  profile_name = profile_file[:-len(".bril")]

  profile_dir = f'./tmp/bench/{profile_name}'
  try:
    os.mkdir(profile_dir)
  except FileExistsError:
    print(f'{profile_dir} exists, overwriting contents')


  res = []

  for mode in modes:
    (name, runmode, options) = mode

    start = time.time()
    subprocess.call(f'cargo run --release {profile} --run-mode {runmode} {options} -o {profile_dir}/{name}', shell=True)
    end = time.time()

    with open(f'{profile_dir}/{name}-args') as f:
      args = f.read().rstrip()
    
    # TODO for final nightly results, remove `--max-runs 2` and let hyperfine find stable results
    result = subprocess.run([
        'hyperfine',
        '--style', 'none',
        '--warmup', '1',
        '--max-runs', '2',
        '--export-json', '-',
        f'"{profile_dir}/{name}{args if len(args) > 0 else ""}"',
    ], capture_output=True)
    res.append({"runMethod": runmode, "benchmark": name, "compileTime": end-start, "hyperfine": json.loads(result.stdout)})
  return res

if __name__ == '__main__':
  # expect a single argument
  if len(os.sys.argv) != 3:
    print("Usage: profile.py <bril_directory> <output_directory>")
    exit(1)

  profile_path, output_path = os.sys.argv[1:]
  profiles = []
  # if it is a directory get all files
  if os.path.isdir(profile_path):
    print(f'Running all bril files in {profile_path}')
    profiles = glob(f'{profile_path}/**/*.bril', recursive=True)
  else:
    profiles = [profile_path]

  iter = 0

  out = []
  for p in profiles:
    print(f'Benchmark {iter} of {len(profiles)} on all treatments')
    iter += 1
    out.extend(bench(p))

  (overview, detailed) = gen_linecount_table()

  with open(f"{output_path}/data/profile.json", "w") as data:
      data.write(json.dumps(out))

  with open(f"{output_path}/data/linecount.tex", "w") as linecount:
      linecount.write(overview)

  with open(f"{output_path}/data/detailed-linecount.tex", "w") as linecount:
      linecount.write(detailed)

  with open(f"{output_path}/data/nightlytable.tex", "w") as nightly_table:
      nightly_table.write(gen_nightly_table(out))
