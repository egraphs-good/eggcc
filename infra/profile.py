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
  "rvsdg-round-trip-to-executable",
  "cranelift-O3",
  "llvm-O0",
  "llvm-O0-eggcc",
  "llvm-O3",
  "llvm-O3-eggcc",
]

def get_eggcc_options(name, profile_dir):
  match name:
    case "rvsdg-round-trip-to-executable":
      return f'--run-mode rvsdg-round-trip-to-executable --llvm-output-dir {profile_dir}/llvm-{name}'
    case "cranelift-O3":
      return f'--run-mode cranelift --optimize-egglog false --optimize-brilift true'
    case "llvm-O0":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm false --llvm-output-dir {profile_dir}/llvm-{name}'
    case "llvm-O3":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm true --llvm-output-dir {profile_dir}/llvm-{name}'
    case "llvm-O0-eggcc":
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
  start = time.time()
  subprocess.call(cmd, shell=True)
  end = time.time()
  return (f"{profile_dir}/{benchmark.treatment}", end-start)



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
      return None
    else:
      # TODO for final nightly results, remove `--max-runs 2` and let hyperfine find stable results
      cmd = f'hyperfine --style none --warmup 1 --max-runs 2 --export-json /dev/stdout "{profile_dir}/{benchmark.treatment}{" " + args if len(args) > 0 else ""}"'
      result = subprocess.run(cmd, capture_output=True, shell=True)
      return (f'{profile_dir}/{benchmark.treatment}', json.loads(result.stdout))

# Run modes that we expect to output llvm IR
def should_have_llvm_ir(runMethod):
  return runMethod in [
    "rvsdg-round-trip-to-executable",
    "llvm-O0",
    "llvm-O0-eggcc",
    "llvm-O3",
    "llvm-O3-eggcc",
  ]

def get_llvm_ir(runMethod, benchmark):
  path = f'./tmp/bench/{benchmark}/llvm-{runMethod}/{benchmark}-{runMethod}.ll'

  try:
    with open(path) as f:
      return f.read()
  except OSError:
    return ""


# aggregate all profile info into a single json array.
def aggregate(compile_times, bench_times):
    res = []

    for path in sorted(compile_times.keys()):
      name = path.split("/")[-2]
      runMethod = path.split("/")[-1]
      result = {"runMethod": runMethod, "benchmark": name, "hyperfine": bench_times[path], "compileTime": compile_times[path]}
      if should_have_llvm_ir(runMethod):
        result["llvm_ir"] = get_llvm_ir(runMethod, name)

      res.append(result)
    return res


if __name__ == '__main__':
  # expect two arguments
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
  

  compile_times = {}
  # create a thread pool for running optimization
  with concurrent.futures.ThreadPoolExecutor(max_workers = 6) as executor:
    futures = {executor.submit(optimize, benchmark) for benchmark in to_run}
    for future in concurrent.futures.as_completed(futures):
      (path, compile_time) = future.result()
      compile_times[path] = compile_time

  # running benchmarks sequentially for more reliable results
  # can set this to true for testing
  isParallelBenchmark = False

  bench_data = {}
  if isParallelBenchmark:
    # create a thread pool for running benchmarks
    with concurrent.futures.ThreadPoolExecutor(max_workers = 6) as executor:
      futures = {executor.submit(bench, benchmark) for benchmark in to_run}
      for future in concurrent.futures.as_completed(futures):
        res = future.result()
        if res is None:
          continue
        (path, _bench_data) = res
        bench_data[path] = _bench_data
  else:
    for benchmark in to_run:
      res = bench(benchmark)
      if res is None:
        continue
      (path, _bench_data) = res
      bench_data[path] = _bench_data

  nightly_data = aggregate(compile_times, bench_data)
  with open(f"{output_path}/data/profile.json", "w") as profile:
    json.dump(nightly_data, profile, indent=2)

  (overview, detailed) = gen_linecount_table()

  with open(f"{output_path}/data/linecount.tex", "w") as linecount:
      linecount.write(overview)

  with open(f"{output_path}/data/detailed-linecount.tex", "w") as linecount:
      linecount.write(detailed)

  with open(f"{output_path}/data/nightlytable.tex", "w") as nightly_table:
      nightly_table.write(gen_nightly_table(nightly_data))
