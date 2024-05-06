#!/usr/bin/env python3

import json
import os
from glob import glob
from sys import stdout
import subprocess


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
    
  for mode in modes:
    (name, runmode, options) = mode
    subprocess.call(f'cargo run --release {profile} --run-mode {runmode} {options} -o {profile_dir}/{name}', shell=True)

    with open(f'{profile_dir}/{name}-args') as f:
      args = f.read().rstrip()
    
    # TODO for final nightly results, remove `--max-runs 2` and let hyperfine find stable results
    subprocess.call(f'hyperfine --warmup 1 --max-runs 2 --export-json {profile_dir}/{name}.json "{profile_dir}/{name} {args}"', shell=True)

# aggregate all profile info into a single json array.
# It walks a file that looks like:
# tmp
# - bench
# -- <benchmark name>
# ---- run_method.json
# ---- run_method.profile
def aggregate():
    res = []
    jsons = glob("./tmp/bench/*/*.json")
    for file_path in jsons:
        if os.stat(file_path).st_size == 0:
            continue
        name = file_path.split("/")[-2]
        runMethod = file_path.split("/")[-1][:-len(".json")]
        result = {"runMethod": runMethod, "benchmark": name}
        with open(file_path) as f:
            result["hyperfine"] = json.load(f)
        res.append(result)
    with open("nightly/data/profile.json", "w") as f:
      json.dump(res, f, indent=2)


if __name__ == '__main__':
  # expect a single argument
  if len(os.sys.argv) != 2:
    print("Usage: profile.py <bril_directory>")
    exit(1)

  arg = os.sys.argv[1]
  profiles = []
  # if it is a directory get all files
  if os.path.isdir(arg):
    print(f'Running all bril files in {arg}')
    profiles = glob(f'{arg}/**/*.bril', recursive=True)
  else:
    profiles = [arg]

  iter = 0
  for p in profiles:
    print(f'Benchmark {iter} of {len(profiles)} on all treatments')
    iter += 1
    bench(p)

  aggregate()
