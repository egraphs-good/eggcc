#!/usr/bin/env python3

import json
import os
import time
from glob import glob
from sys import stdout
import subprocess

from nightly_table import gen_nightly_table
from gen_linecount import gen_linecount_table
import concurrent.futures

treatments = [
  "rvsdg_roundtrip",
  "cranelift-O3",
  "llvm-peep",
  "llvm-peep-eggcc",
  "llvm-O3",
  "llvm-O3-eggcc",
]

def get_eggcc_options(name, profile_dir):
  match name:
    case "rvsdg_roundtrip":
      return '--run-mode rvsdg-round-trip-to-executable'
    case "cranelift-O3":
      return f'--run-mode cranelift --optimize-egglog false --optimize-brilift true'
    case "llvm-peep":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm false --llvm-output-dir {profile_dir}/llvm-{name}'
    case "llvm-O3":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm true --llvm-output-dir {profile_dir}/llvm-{name}'
    case "llvm-peep-eggcc":
      return f'--run-mode llvm --optimize-egglog true --optimize-bril-llvm false --llvm-output-dir {profile_dir}/llvm-{name}'
    case "llvm-O3-eggcc":
      return f'--run-mode llvm --optimize-egglog true --optimize-bril-llvm true --llvm-output-dir {profile_dir}/llvm-{name}'
    case _:
      raise Exception("Unexpected run mode: " + name)
    

class Benchmark:
  def __init__(self, path, treatment, index, total):
    self.path = path
    self.treatment = treatment
    # index of this benchmark (for printing)
    self.index = index
    # total number of benchmarks being run
    self.total = total

def benchmark_name(benchmark_path):
  return benchmark_path.split("/")[-1][:-len(".bril")]

def benchmark_profile_dir(benchmark_path):
  return f'./tmp/bench/{benchmark_name(benchmark_path)}'

def setup_benchmark(benchmark_path):
  # strip off the .bril to get just the profile name
  profile_dir = benchmark_profile_dir(benchmark_path)
  try:
    os.mkdir(profile_dir)
  except FileExistsError:
    print(f'{profile_dir} exists, overwriting contents')

def optimize(benchmark):
  print(f'[{benchmark.index}/{benchmark.total}] Optimizing {benchmark.path} with {benchmark.treatment}')
  profile_dir = benchmark_profile_dir(benchmark.path)
  cmd = f'cargo run --release {benchmark.path} {get_eggcc_options(benchmark.treatment, profile_dir)} -o {profile_dir}/{benchmark.treatment}'
  print(f'Running: {cmd}')
  subprocess.call(cmd, shell=True)



def bench(benchmark):
  print(f'[{benchmark.index}/{benchmark.total}] Benchmarking {benchmark.path} with {benchmark.treatment}')
  profile_dir = benchmark_profile_dir(benchmark.path)

  with open(f'{profile_dir}/{benchmark.treatment}-args') as f:
    args = f.read().rstrip()

    # check that we have a file for the benchmark
    if not os.path.isfile(f'{profile_dir}/{benchmark.treatment}'):
      # TODO add an error to the errors file
      #with open('nightly/data/errors.txt', 'a') as f:
        #f.write(f'ERROR: No executable found for {name} in {benchmark.path}\n')
      pass
    else:
      # TODO for final nightly results, remove `--max-runs 2` and let hyperfine find stable results
      cmd = f'hyperfine --warmup 1 --max-runs 2 --export-json {profile_dir}/{benchmark.treatment}.json "{profile_dir}/{benchmark.treatment} {args}"'
      print(f'Running: {cmd}')
      subprocess.call(cmd, shell=True)

def get_llvm(runMethod, benchmark):
  path = f'./tmp/bench/{benchmark}/llvm-{runMethod}/{benchmark}-{runMethod}.ll'

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
          result["llvm_ir"] = get_llvm(runMethod, name)
        with open(file_path) as f:
            result["hyperfine"] = json.load(f)
        res.append(result)
    with open("nightly/data/profile.json", "w") as f:
      json.dump(res, f, indent=2)


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

  for benchmark_path in profiles:
    setup_benchmark(benchmark_path)

  to_run = []
  index = 0
  total = len(profiles) * len(treatments)
  for benchmark_path in profiles:
    for treatment in treatments:
      to_run.append(Benchmark(benchmark_path, treatment, index, total))
      index += 1
  

  # create a thread pool for running optimization
  with concurrent.futures.ThreadPoolExecutor(max_workers = 6) as executor:
    futures = {executor.submit(optimize, benchmark) for benchmark in to_run}
    for future in concurrent.futures.as_completed(futures):
      continue

  # running benchmarks sequentially for more reliable results
  # can set this to true for testing
  isParallelBenchmark = False

  if isParallelBenchmark:
    # create a thread pool for running benchmarks
    with concurrent.futures.ThreadPoolExecutor(max_workers = 6) as executor:
      futures = {executor.submit(bench, benchmark) for benchmark in to_run}
      for future in concurrent.futures.as_completed(futures):
        continue
  else:
    for benchmark in to_run:
      bench(benchmark)

  aggregate()

  (overview, detailed) = gen_linecount_table()

  with open(f"{output_path}/data/linecount.tex", "w") as linecount:
      linecount.write(overview)

  with open(f"{output_path}/data/detailed-linecount.tex", "w") as linecount:
      linecount.write(detailed)

  with open(f"{output_path}/data/nightlytable.tex", "w") as nightly_table:
    with open(f"{output_path}/data/profile.json") as data_file:
      data = json.loads(data_file.read())
      nightly_table.write(gen_nightly_table(data))
