#!/usr/bin/env python3

import json
import os
from glob import glob
from sys import stdout
import subprocess

modes = [
  "rvsdg_roundtrip",

  "egglog_noopt_brilift_noopt",
  "egglog_noopt_brilift_opt",
  "egglog_opt_brilift_noopt",
  "egglog_opt_brilift_opt",

  "egglog_noopt_llvm_noopt",
  "egglog_noopt_llvm_opt",
  "egglog_opt_llvm_noopt",
  "egglog_opt_llvm_opt",
]

def get_eggcc_options(name, profile_dir):
  match name:
    case "rvsdg_roundtrip":
      return '--run-mode rvsdg-round-trip-to-executable'
    case "egglog_noopt_brilift_noopt":
      return '--run-mode compile-brilift --optimize-egglog false --optimize-brilift false'
    case "egglog_noopt_brilift_opt":
      return '--run-mode compile-brilift --optimize-egglog false --optimize-brilift true'
    case "egglog_opt_brilift_noopt":
      return '--run-mode compile-brilift --optimize-egglog true --optimize-brilift false'
    case "egglog_opt_brilift_opt":
      return '--run-mode compile-brilift --optimize-egglog true --optimize-brilift true'
    case "egglog_noopt_llvm_noopt":
      return f'--run-mode compile-bril-llvm --optimize-egglog false --optimize-bril-llvm false --llvm-output-dir {profile_dir}/llvm-{name}'
    case "egglog_noopt_llvm_opt":
      return f'--run-mode compile-bril-llvm --optimize-egglog false --optimize-bril-llvm true --llvm-output-dir {profile_dir}/llvm-{name}'
    case "egglog_opt_llvm_noopt":
      return f'--run-mode compile-bril-llvm --optimize-egglog true --optimize-bril-llvm false --llvm-output-dir {profile_dir}/llvm-{name}'
    case "egglog_opt_llvm_opt":
      return f'--run-mode compile-bril-llvm --optimize-egglog true --optimize-bril-llvm true --llvm-output-dir {profile_dir}/llvm-{name}'
    case _:
      raise Exception("Unexpected run mode: " + name)
    

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
    subprocess.call(f'cargo run --release {profile} {get_eggcc_options(mode, profile_dir)} -o {profile_dir}/{mode}', shell=True)

    with open(f'{profile_dir}/{mode}-args') as f:
      args = f.read().rstrip()
    
    # TODO for final nightly results, remove `--max-runs 2` and let hyperfine find stable results
    subprocess.call(f'hyperfine --warmup 1 --max-runs 2 --export-json {profile_dir}/{mode}.json "{profile_dir}/{mode} {args}"', shell=True)

def get_llvm(runMethod, benchmark):
  path = f'./tmp/bench/{benchmark}/llvm-{runMethod}/{benchmark}_{runMethod}.ll'

  with open(path) as f:
    return f.read()

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
        if "llvm" in runMethod:
          result["llvm"] = get_llvm(runMethod, name)
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
