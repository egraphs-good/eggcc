#!/usr/bin/env python3

import json
import os
import time
from glob import glob
from sys import stdout
import subprocess

import concurrent.futures

treatments = [
  "rvsdg-round-trip-to-executable",
  #"cranelift-O3", currently disabled since it doesn't support measuring cycles yet
  "llvm-O0",
  "llvm-O1",
  "llvm-O2",
  "llvm-O0-eggcc",
  "llvm-O3",
  "llvm-O3-eggcc",
]

# Where to output files that are needed for nightly report
DATA_DIR = None

# Where to write intermediate files that should be cleaned up at the end of this script
TMP_DIR = "tmp"

def get_eggcc_options(run_mode, benchmark):
  llvm_out_dir = f"{DATA_DIR}/llvm/{benchmark}/{run_mode}"
  match run_mode:
    case "rvsdg-round-trip-to-executable":
      return f'--run-mode rvsdg-round-trip-to-executable --llvm-output-dir {llvm_out_dir}'
    case "cranelift-O3":
      return f'--run-mode cranelift --optimize-egglog false --optimize-brilift true'
    case "llvm-O0":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0 --llvm-output-dir {llvm_out_dir}'
    case "llvm-O1":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O1 --llvm-output-dir {llvm_out_dir}'
    case "llvm-O2":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O2 --llvm-output-dir {llvm_out_dir}'
    case "llvm-O3":
      return f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3 --llvm-output-dir {llvm_out_dir}'
    case "llvm-O0-eggcc":
      return f'--run-mode llvm --optimize-egglog true --optimize-bril-llvm O0 --llvm-output-dir {llvm_out_dir}'
    case "llvm-O3-eggcc":
      return f'--run-mode llvm --optimize-egglog true --optimize-bril-llvm O3 --llvm-output-dir {llvm_out_dir}'
    case _:
      raise Exception("Unexpected run mode: " + run_mode)
    

class Benchmark:
  def __init__(self, path, treatment, index, total):
    self.path = path
    self.name = path.split("/")[-1][:-len(".bril")]
    self.treatment = treatment
    # index of this benchmark (for printing)
    self.index = index
    # total number of benchmarks being run
    self.total = total

def benchmark_profile_dir(name):
  return f'{TMP_DIR}/{name}'

def setup_benchmark(name):
  profile_dir = benchmark_profile_dir(name)
  os.mkdir(profile_dir)

def optimize(benchmark):
  print(f'[{benchmark.index}/{benchmark.total}] Optimizing {benchmark.name} with {benchmark.treatment}')
  profile_dir = benchmark_profile_dir(benchmark.name)
  cmd = f'cargo run --release {benchmark.path} {get_eggcc_options(benchmark.treatment, benchmark.name)} -o {profile_dir}/{benchmark.treatment}'
  print(f'Running: {cmd}', flush=True)
  start = time.time()
  process = subprocess.run(cmd, shell=True)
  process.check_returncode()
  end = time.time()
  return (f"{profile_dir}/{benchmark.treatment}", end-start)



def bench(benchmark):
  print(f'[{benchmark.index}/{benchmark.total}] Benchmarking {benchmark.name} with {benchmark.treatment}', flush=True)
  profile_dir = benchmark_profile_dir(benchmark.name)

  with open(f'{profile_dir}/{benchmark.treatment}-args') as f:
    args = f.read().rstrip()

    # check that we have a file for the benchmark
    if not os.path.isfile(f'{profile_dir}/{benchmark.treatment}'):
      # TODO add an error to the errors file
      #with open('nightly/data/errors.txt', 'a') as f:
        #f.write(f'ERROR: No executable found for {name} in {benchmark.path}\n')
      return None
    else:
      # hyperfine command for measuring time, unused in favor of cycles
      # cmd = f'hyperfine --style none --warmup 1 --max-runs 2 --export-json /dev/stdout "{profile_dir}/{benchmark.treatment}{" " + args if len(args) > 0 else ""}"'
      time_per_benchmark = 1.0
      resulting_num_cycles = []
      time_start = time.time()
      while True:
        args_str = " " + args if len(args) > 0 else ""
        cmd = f'{profile_dir}/{benchmark.treatment}{args_str}'
        result = subprocess.run(cmd, capture_output=True, shell=True)
        
        if result.returncode != 0:
          raise Exception(f'Error running {benchmark.name} with {benchmark.treatment}: {result.stderr}')
        res_cycles = int(result.stderr)
        resulting_num_cycles.append(res_cycles)

        # if we have run for at least 1 second and we have at least 2 samples, stop
        if time.time() - time_start > time_per_benchmark and len(resulting_num_cycles) >= 2:
          break

      return (f'{profile_dir}/{benchmark.treatment}', resulting_num_cycles)

# Run modes that we expect to output llvm IR
def should_have_llvm_ir(runMethod):
  return runMethod in [
    "rvsdg-round-trip-to-executable",
    "llvm-O0",
    "llvm-O0-eggcc",
    "llvm-O3",
    "llvm-O3-eggcc",
  ]

# aggregate all profile info into a single json array.
def aggregate(compile_times, bench_times, benchmark_metadata):
    res = []

    for path in sorted(compile_times.keys()):
      name = path.split("/")[-2]
      runMethod = path.split("/")[-1]
      result = {"runMethod": runMethod, "benchmark": name, "cycles": bench_times[path], "compileTime": compile_times[path], "metadata": benchmark_metadata[name]}

      res.append(result)
    return res

def is_looped(bril_file):
  with open(bril_file) as f:
    txt = f.read()
    return "orig_main" in txt

if __name__ == '__main__':
  # expect two arguments
  if len(os.sys.argv) != 3:
    print("Usage: profile.py <bril_directory> <output_directory>")
    exit(1)

  # Create tmp directory for intermediate files
  try:
    os.mkdir(TMP_DIR)
  except FileExistsError:
    print(f"{TMP_DIR} exits, deleting contents")
    # remove the files in the directory
    os.system(f"rm -rf {TMP_DIR}/*")


  bril_dir, DATA_DIR = os.sys.argv[1:]
  profiles = []
  # if it is a directory get all files
  if os.path.isdir(bril_dir):
    print(f'Running all bril files in {bril_dir}')
    profiles = glob(f'{bril_dir}/**/*.bril', recursive=True)
  else:
    profiles = [bril_dir]

  benchmark_metadata = {}
  for profile in profiles:
    name = profile.split("/")[-1][:-len(".bril")]
    benchmark_metadata[name] = {"looped": is_looped(profile), "path": profile}

  to_run = []
  index = 0
  total = len(profiles) * len(treatments)
  for benchmark_path in profiles:
    for treatment in treatments:
      to_run.append(Benchmark(benchmark_path, treatment, index, total))
      index += 1

  benchmark_names = set([benchmark.name for benchmark in to_run])
  for benchmark_name in benchmark_names:
    setup_benchmark(benchmark_name)
  

  compile_times = {}
  # create a thread pool for running optimization
  with concurrent.futures.ThreadPoolExecutor(max_workers = 6) as executor:
    futures = {executor.submit(optimize, benchmark) for benchmark in to_run}
    for future in concurrent.futures.as_completed(futures):
      try:
        (path, compile_time) = future.result()
        compile_times[path] = compile_time
      except Exception as e:
        print(f"Shutting down executor due to error: {e}")
        executor.shutdown(wait=False, cancel_futures=True)
        raise e

  # running benchmarks sequentially for more reliable results
  # can set this to true for testing
  isParallelBenchmark = False

  bench_data = {}
  if isParallelBenchmark:
    # create a thread pool for running benchmarks
    with concurrent.futures.ThreadPoolExecutor(max_workers = 6) as executor:
      futures = {executor.submit(bench, benchmark) for benchmark in to_run}
      for future in concurrent.futures.as_completed(futures):
        try:
          res = future.result()
          if res is None:
            continue
          (path, _bench_data) = res
          bench_data[path] = _bench_data
        except Exception as e:
          print(f"Shutting down executor due to error: {e}")
          executor.shutdown(wait=False, cancel_futures=True)
          raise e
  else:
    for benchmark in to_run:
      res = bench(benchmark)
      if res is None:
        continue
      (path, _bench_data) = res
      bench_data[path] = _bench_data

  nightly_data = aggregate(compile_times, bench_data, benchmark_metadata)
  with open(f"{DATA_DIR}/profile.json", "w") as profile:
    json.dump(nightly_data, profile, indent=2)

  # Clean up intermediate files
  os.system(f"rm -rf {TMP_DIR}")
