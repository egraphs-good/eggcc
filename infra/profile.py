#!/usr/bin/env python3

import json
import os
import signal
from glob import glob
from sys import stdout
import sys
import subprocess
import resource

import concurrent.futures
import time
from generate_cfgs import make_cfgs


# testing mode takes much fewer samples than the real eval in the paper
IS_TESTING_MODE = True

def eggcc_timeout_secs():
  if IS_TESTING_MODE:
    return 10 * 60 # 10 minutes
  else:
    return 6 * 60 * 60 # 6 hours (ILP can take a long time, timing out on lots of regions after 5 min)

def num_warmup_samples():
  if IS_TESTING_MODE:
    return 2
  return 10
  
# TODO bump this to 200
def num_samples():
  if IS_TESTING_MODE:
    return 100
  return 100


def average(lst):
  return sum(lst) / len(lst)

TO_ABLATE = "" # change to a ruleset to ablate


# use for running a subset of the treatments
# disables checks that ensure the data is complete
UNSAFE_TREATMENTS = False
treatments = [
  "rvsdg-round-trip-to-executable",
  "llvm-O0-O0",
  "llvm-O1-O0",
  "llvm-O2-O0",
  "eggcc-O0-O0",
  "eggcc-sequential-O0-O0",
  "llvm-O3-O0",
  "llvm-O3-O3",
  "eggcc-O3-O0",
  "eggcc-O3-O3",
  "eggcc-tiger-WL-O0-O0",
  "eggcc-tiger-O0-O0",
  "eggcc-tiger-ILP-O0-O0",
  "eggcc-tiger-ILP-CBC-O0-O0",
  "eggcc-tiger-ILP-NOMIN-O0-O0",
  #"eggcc-tiger-ILP-WITHCTX-O0-O0", #disabled for now
  "eggcc-WITHCTX-O0-O0",
  # run both tiger and ILP on the same egraphs, keep this last in the list
  "eggcc-tiger-ILP-COMPARISON", 
]

example_subset_treatments = [
  "llvm-O0-O0",
  "eggcc-O0-O0",
  "llvm-O3-O0",
  "eggcc-tiger-WL-O0-O0",
  "eggcc-tiger-O0-O0"
]


if TO_ABLATE != "":
  treatments.extend([
    "eggcc-ablation-O0-O0",
    "eggcc-ablation-O3-O0",
    "eggcc-ablation-O3-O3",
  ])

# Where to output files that are needed for nightly report
DATA_DIR = None

# Where to write intermediate files that should be cleaned up at the end of this script
TMP_DIR = "tmp"

EGGCC_BINARY = "target/release/eggcc"

MEMORY_LIMIT_BYTES = 16 * 1024 * 1024 * 1024 
MEMORY_LIMIT_HUMAN = "16 GiB"

ILP_TREATMENT_MARKER = "ILP"


def _should_enforce_memory_limit(treatment: str) -> bool:
  return not ILP_TREATMENT_MARKER in treatment


def _set_memory_limits():
  limits_to_set = [getattr(resource, name, None) for name in ("RLIMIT_AS", "RLIMIT_DATA")]
  for limit in limits_to_set:
    if limit is None:
      continue
    try:
      resource.setrlimit(limit, (MEMORY_LIMIT_BYTES, MEMORY_LIMIT_BYTES))
    except (ValueError, OSError):
      continue


def _memory_limit_exceeded(returncode):
  if returncode is None:
    return False

  signals = (signal.SIGKILL, signal.SIGABRT)
  if returncode < 0:
    return -returncode in signals

  return returncode in [128 + sig for sig in signals]


def _terminate_process_tree(proc):
  try:
    os.killpg(proc.pid, signal.SIGTERM)
  except ProcessLookupError:
    return
  try:
    proc.wait(timeout=5)
  except subprocess.TimeoutExpired:
    try:
      os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
      pass
    proc.wait()


def run_with_timeout_killing_tree(cmd, timeout_secs, *, apply_memory_limits=True):
  preexec = _set_memory_limits if apply_memory_limits else None
  with subprocess.Popen(
      cmd,
      shell=True,
      stdout=subprocess.PIPE,
      stderr=subprocess.PIPE,
      text=True,
      start_new_session=True,
      preexec_fn=preexec) as proc:
    try:
      stdout, stderr = proc.communicate(timeout=timeout_secs)
      return subprocess.CompletedProcess(proc.args, proc.returncode, stdout, stderr)
    except subprocess.TimeoutExpired as exc:
      _terminate_process_tree(proc)
      stdout, stderr = proc.communicate()
      exc.output = stdout
      exc.stderr = stderr
      raise


# returns two strings:
# the first is the eggcc run mode for optimizing the input bril program
# the second is the command line arguments for producing an output file using llvm
def get_eggcc_options(benchmark):
  match benchmark.treatment:
    case "rvsdg-round-trip-to-executable":
      return (f'rvsdg-round-trip',  f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "llvm-O0-O0":
      return (f'parse', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "llvm-O1-O0":
      return (f'parse', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O1_O0')
    case "llvm-O2-O0":
      return (f'parse', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O2_O0')
    case "llvm-O3-O0":
      return (f'parse', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O0')
    case "llvm-O3-O3":
      return (f'parse', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O3')
    case "eggcc-sequential-O0-O0":
      return (f'optimize --eggcc-schedule sequential', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-O0-O0":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-O3-O0":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O0')
    case "eggcc-O3-O3":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O3')
    case "eggcc-ablation-O0-O0":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0 --ablate {TO_ABLATE}')
    case "eggcc-ablation-O3-O0":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O0 --ablate {TO_ABLATE}')
    case "eggcc-ablation-O3-O3":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O3 --ablate {TO_ABLATE}')
    case "eggcc-tiger-WL-O0-O0":
      return (f'optimize --use-tiger', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-tiger-O0-O0":
      return (f'optimize --use-tiger --non-weakly-linear', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-tiger-ILP-COMPARISON":
      return (f'optimize --use-tiger --non-weakly-linear --time-ilp', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-tiger-ILP-O0-O0":
      return (f'optimize --use-tiger --tiger-ilp --non-weakly-linear', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-tiger-ILP-CBC-O0-O0":
      return (f'optimize --use-tiger --tiger-ilp --non-weakly-linear --ilp-solver cbc', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-tiger-ILP-WITHCTX-O0-O0":
      return (f'optimize --use-tiger --tiger-ilp --non-weakly-linear --with-context', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "eggcc-tiger-ILP-NOMIN-O0-O0":
      return (f'optimize --use-tiger --tiger-ilp --non-weakly-linear --ilp-no-minimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')

    case "eggcc-WITHCTX-O0-O0":
      # run with the with-context flag
      return (f'optimize --with-context', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case _:
      raise Exception("Unexpected run mode: " + benchmark.treatment)
    

class Benchmark:
  def __init__(self, path, treatment, index, total):
    self.path = path
    # assert the file doesn't have any dots in the name besides the last
    if path.split("/")[-1].count(".") != 1:
      raise Exception(f"File {path} has multiple dots in name")
    # name is the name of the file without extension
    self.name = path.split("/")[-1].split(".")[0]
    self.treatment = treatment
    # index of this benchmark (for printing)
    self.index = index
    # total number of benchmarks being run
    self.total = total
    self.is_last_before_ilp = False

def benchmark_profile_dir(name):
  return f'{TMP_DIR}/{name}'

def setup_benchmark(name):
  profile_dir = benchmark_profile_dir(name)
  os.mkdir(profile_dir)


# runs optimization on the benchmark, a dictionary with
# a name and treatment (and index for printing)
# returns a dictionary with the path to the optimized binary,
# eggcc compile time, and llvm compile time
def optimize(benchmark):
  print(f'[{benchmark.index}/{benchmark.total}] Optimizing {benchmark.name} with {benchmark.treatment}', flush=True)
  if benchmark.is_last_before_ilp:
    print('Waiting for all other benchmarks to complete before ILP comparison...', flush=True)
  profile_dir = benchmark_profile_dir(benchmark.name)
  optimized_bril_file = f'{profile_dir}/{benchmark.name}-{benchmark.treatment}.bril'
  eggcc_run_data = f'{profile_dir}/{benchmark.treatment}-eggcc-run-data.json'
  llvm_run_data = f'{profile_dir}/{benchmark.treatment}-llvm-run-data.json'

  # get the commands we need to run
  (eggcc_run_mode, llvm_args) = get_eggcc_options(benchmark)
  os.makedirs(f"{DATA_DIR}/llvm/{benchmark.name}/{benchmark.treatment}", exist_ok=True)
  llvm_out_file = f"{DATA_DIR}/llvm/{benchmark.name}/{benchmark.treatment}/optimized.ll"
  cmd1 = f'{EGGCC_BINARY} {benchmark.path} --run-mode {eggcc_run_mode} --run-data-out {eggcc_run_data}'
  cmd2 = f'{EGGCC_BINARY} {optimized_bril_file} --run-data-out {llvm_run_data} --add-timing {llvm_args} -o {profile_dir}/{benchmark.treatment} --llvm-output-dir {llvm_out_file}'

  failure_data = {
      "path": f"{profile_dir}/{benchmark.treatment}",
      "eggccCompileTimeSecs": False,
      "eggccSerializationTimeSecs": False,
      "eggccExtractionTimeSecs": False,
      "llvmCompileTimeSecs": False,
      "extractRegionTimings": False,
      "failed": True,
      "ILPRegionTimeOut": False,
      "error": '',
    }

  enforce_memory_limit = _should_enforce_memory_limit(benchmark.treatment)
  try:
    process = run_with_timeout_killing_tree(
      cmd1,
      eggcc_timeout_secs(),
      apply_memory_limits=enforce_memory_limit,
    )
  except subprocess.TimeoutExpired:
    # Timeouts are failures
    print(f'[{benchmark.index}/{benchmark.total}] Timeout running {cmd1} after {eggcc_timeout_secs()} seconds', flush=True)
    failure_data["error"] = f'Timeout running {cmd1} after {eggcc_timeout_secs()} seconds'
    return failure_data

  if _memory_limit_exceeded(process.returncode):
    stderr_msg = process.stderr.strip()
    detail = f" Details: {stderr_msg}" if stderr_msg else ""
    msg = f'eggcc exceeded the memory limit ({MEMORY_LIMIT_HUMAN}) while optimizing {benchmark.name} with {benchmark.treatment}.{detail}'
    print(f'[{benchmark.index}/{benchmark.total}] {msg}', flush=True, file=sys.stderr)
    failure_data["error"] = msg
    return failure_data


  # check for an ILP timeout in the output
  if "TIMEOUT" in process.stdout:
    failure_data["ILPRegionTimeOut"] = True
    failure_data["error"] = f'ILP timeout while extracting a region.'
    return failure_data

  if process.returncode != 0:
    print(f'[{benchmark.index}/{benchmark.total}] Error running {cmd1}: {process.stderr}', flush=True, file=sys.stderr)
    failure_data["error"] = f'Error running {cmd1}: {process.stderr}'
    return failure_data

  with open(optimized_bril_file, 'w') as f:
    f.write(process.stdout)

  # Second command intentionally has no timeout (can run longer than first phase)
  preexec = None
  if os.name != "nt" and enforce_memory_limit:
    preexec = _set_memory_limits
  process2 = subprocess.run(
    cmd2,
    shell=True,
    text=True,
    capture_output=True,
    preexec_fn=preexec,
  )

  if _memory_limit_exceeded(process2.returncode):
    stderr_msg = process2.stderr.strip()
    details = f"\nDetails: {stderr_msg}" if stderr_msg else ""
    raise RuntimeError(
      f'eggcc exceeded the memory limit ({MEMORY_LIMIT_HUMAN}) while lowering {benchmark.name} with {benchmark.treatment}.{details}'
    )

  if process2.returncode != 0:
    raise Exception(f'Error running {cmd2}: {process2.stderr}')

  eggcc_compile_time = eggcc_extraction_time = eggcc_serialization_time = 0.0
  extract_region_timings = []
  # parse json from eggcc run data (guard if file unexpectedly missing)
  if os.path.isfile(eggcc_run_data):
    with open(eggcc_run_data) as f:
      eggcc_data = json.load(f)
      secs = eggcc_data["eggcc_compile_time"]["secs"]
      nanos = eggcc_data["eggcc_compile_time"]["nanos"]
      eggcc_compile_time = secs + nanos / 1e9
      secs = eggcc_data["eggcc_serialization_time"]["secs"]
      nanos = eggcc_data["eggcc_serialization_time"]["nanos"]
      eggcc_serialization_time = secs + nanos / 1e9
      secs = eggcc_data["eggcc_extraction_time"]["secs"]
      nanos = eggcc_data["eggcc_extraction_time"]["nanos"]
      eggcc_extraction_time = secs + nanos / 1e9
  extract_region_timings = eggcc_data.get("extract_region_timings", [])

  llvm_compile_time = 0.0
  if os.path.isfile(llvm_run_data):
    with open(llvm_run_data) as f:
      llvm_data = json.load(f)
      secs = llvm_data["llvm_compile_time"]["secs"]; nanos = llvm_data["llvm_compile_time"]["nanos"]; llvm_compile_time = secs + nanos / 1e9

  res = {
    "path": f"{profile_dir}/{benchmark.treatment}",
    "eggccCompileTimeSecs": eggcc_compile_time,
    "eggccSerializationTimeSecs": eggcc_serialization_time,
    "eggccExtractionTimeSecs": eggcc_extraction_time,
    "llvmCompileTimeSecs": llvm_compile_time,
    "extractRegionTimings": extract_region_timings,
    "failed": False,
    "ILPRegionTimeOut": False,
  }
  return res


def take_sample(cmd, benchmark):
  try:
    # (No timeout for benchmark sample execution; extraction already constrained if tiger)
    result = subprocess.run(cmd, capture_output=True, shell=True)
  except subprocess.TimeoutExpired:
    raise Exception(f'Timeout executing benchmark sample for {benchmark.name} {benchmark.treatment}: {cmd}')
  if result.returncode != 0:
    raise Exception(f'Error running {benchmark.name} with {benchmark.treatment}: {result.stderr}')
  return int(result.stderr)


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

      args_str = " " + args if len(args) > 0 else ""
      cmd = f'{profile_dir}/{benchmark.treatment}{args_str}'
      num_samples_so_far = 0
      warmup_cycles = []
      resulting_num_cycles = []

      # take some warmup cycles
      while len(warmup_cycles) < num_warmup_samples():
        warmup_cycles.append(take_sample(cmd, benchmark))

      num_to_run = num_samples()

      while True:
        resulting_num_cycles.append(take_sample(cmd, benchmark))

        num_samples_so_far += 1
        # if we have run for at least 1 second and we have at least 2 samples, stop
        #if time.time() - time_start > time_per_benchmark and len(resulting_num_cycles) >= 2:
         # break
        if num_samples_so_far >= num_to_run:
          break
      
      return (f'{profile_dir}/{benchmark.treatment}', resulting_num_cycles)


# go up in directory until hitting "passing" folder
def get_suite(path):
  while True:
    # if we can't go up anymore, return unknown
    if os.path.dirname(path) == path:
      return "unknown"

    oldpath = path
    path = os.path.dirname(path)
    if os.path.basename(path) == "passing":
      return os.path.basename(oldpath)


# aggregate all profile info into a single json array.
def aggregate(compile_data, bench_times, paths):
    res = []

    for path in sorted(compile_data.keys()):
      name = path.split("/")[-2]
      runMethod = path.split("/")[-1]
      suite = get_suite(paths[name])
      cycles = bench_times.get(path, False)  # false if not benchmarked (e.g., timed out)
      result = {"runMethod": runMethod, "benchmark": name, "cycles": cycles, "path": paths[name], "suite": suite}
      for key in compile_data[path]:
        result[key] = compile_data[path][key]
      # Enforce timeout invariant
      if result.get("failed"):
        for k in ["cycles", "eggccCompileTimeSecs", "eggccSerializationTimeSecs", "eggccExtractionTimeSecs", "llvmCompileTimeSecs", "extractRegionTimings"]:
          result[k] = False
      else:
        # basic sanity checks (best effort)
        for k in ["eggccCompileTimeSecs", "eggccSerializationTimeSecs", "eggccExtractionTimeSecs", "llvmCompileTimeSecs"]:
          if not isinstance(result.get(k), (int, float)):
            raise Exception(f"Non-timeout entry missing numeric field {k} for {name} {runMethod}")
        if not (isinstance(result.get("cycles"), list) and len(result["cycles"]) > 0):
            raise Exception(f"Non-timeout entry has invalid cycles for {name} {runMethod}")
      res.append(result)
    return res

def all_benchmarks(path):
  # if it's a file, return it
  if os.path.isfile(path):
    return [path]

  return glob(f'{path}/**/*.bril', recursive=True) + glob(f'{path}/**/*.rs', recursive=True)

def run_benchmarks_parallel(benchmarks, parallelism, compile_data_out):
  with concurrent.futures.ThreadPoolExecutor(max_workers = parallelism) as executor:
    futures = {executor.submit(optimize, benchmark) for benchmark in benchmarks}
    for future in concurrent.futures.as_completed(futures):
      try:
        res = future.result()
        path = res["path"]
        res.pop("path")
        compile_data_out[path] = res
      except Exception as e:
        print(f"Shutting down executor due to error: {e}")
        executor.shutdown(wait=False, cancel_futures=True)
        raise e

def build_eggcc():
  print("Building eggcc")
  buildres = os.system("cargo build --release")
  if buildres != 0:
    print("Failed to build eggcc")
    exit(1)

if __name__ == '__main__':
  start_time = time.perf_counter()
  # expect two arguments
  if len(os.sys.argv) < 3:
    print("Usage: profile.py <output_directory> <bril_directory> <--parallel> <--paper>")
    exit(1)

  # check for paper flag
  for arg in os.sys.argv:
    if arg == "--paper":
      IS_TESTING_MODE = False

  if IS_TESTING_MODE:
    print("WARNING: Running in testing mode with reduced samples. Pass the --paper flag for the final paper results.")

  # running benchmarks sequentially for more reliable results
  # can set this to true for testing
  isParallelBenchmark = False
  # detect parallel flag
  for arg in os.sys.argv:
    if arg == "--parallel":
      isParallelBenchmark = True

  # Create tmp directory for intermediate files
  try:
    os.mkdir(TMP_DIR)
  except FileExistsError:
    print(f"{TMP_DIR} exits, deleting contents")
    # remove the files in the directory
    os.system(f"rm -rf {TMP_DIR}/*")

  # build eggcc
  build_eggcc()



  DATA_DIR, bril_dir  = os.sys.argv[1:3]
  profiles = []
  # if it is a directory get all files
  if os.path.isdir(bril_dir):
    print(f'Running all bril files in {bril_dir}')
    profiles = all_benchmarks(bril_dir)
  else:
    profiles = [bril_dir]

  paths = {}
  for profile in profiles:
    bench_name_parts = profile.split("/")[-1].split(".")
    if len(bench_name_parts) != 2:
      raise Exception(f"Invalid benchmark name: {profile}")
    name = bench_name_parts[0]
    paths[name] = profile

  to_run = []
  index = 1
  total = len(profiles) * len(treatments)
  for treatment in treatments:
    for benchmark_path in profiles:
      to_run.append(Benchmark(benchmark_path, treatment, index, total))
      index += 1

  benchmark_names = set([benchmark.name for benchmark in to_run])
  for benchmark_name in benchmark_names:
    setup_benchmark(benchmark_name)
  

  # map from path to data (the path includes treatment, benchmark name)
  compile_data = {}
  # get the number of cores on this machine 
  parallelism = (os.cpu_count() - 1)
  # for large machines leave a few cores free
  if parallelism > 30:
    parallelism -= 4
  # Use 10 threads for ILP comparison, and each ILP benchmark will use cup_count() / 11 threads
  # WARNING: if you edit this, edit the number of threads used in time_ilp.cpp
  ilp_parallelism = 10 if os.cpu_count() >= 20 else 1


  # separate to_run into ILP_COMPARISON and others
  ilp_comparison = [b for b in to_run if b.treatment == "eggcc-tiger-ILP-COMPARISON"]
  others = [b for b in to_run if b.treatment != "eggcc-tiger-ILP-COMPARISON"]
  others[-1].is_last_before_ilp = True

  run_benchmarks_parallel(others, parallelism, compile_data)
  print(f'Finished benchmarks, took {time.perf_counter() - start_time:.2f} seconds', flush=True)
  time_after_others = time.perf_counter()
  run_benchmarks_parallel(ilp_comparison, ilp_parallelism, compile_data)
  print(f'Completed ILP comparison benchmarks, took {time.perf_counter() - time_after_others:.2f} seconds', flush=True)
  
  # Derive timed-out paths and successful (non-timeout) run modes per benchmark (avoid duplication later)
  failed_paths = set()
  successful = {}
  for cpath, data in compile_data.items():
    benche = cpath.split("/")[-2]
    mode = cpath.split("/")[-1]
    if data["failed"]:
      failed_paths.add(cpath)
    else:
      successful.setdefault(benche, []).append(mode)

  bench_data = {}
  if isParallelBenchmark:
    with concurrent.futures.ThreadPoolExecutor(max_workers = parallelism) as executor:
      futures = {executor.submit(bench, benchmark) for benchmark in to_run if f"{TMP_DIR}/{benchmark.name}/{benchmark.treatment}" not in failed_paths}
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
      path_key = f"{TMP_DIR}/{benchmark.name}/{benchmark.treatment}"
      if path_key in failed_paths:
        print(f"Skipping benchmarking due to failure: {benchmark.name} {benchmark.treatment}", flush=True)
        continue
      res = bench(benchmark)
      if res is None:
        continue
      (path, _bench_data) = res
      bench_data[path] = _bench_data

  nightly_data = aggregate(compile_data, bench_data, paths)
  with open(f"{DATA_DIR}/profile.json", "w") as profile:
    json.dump(nightly_data, profile, indent=2)

  # Parallel CFG generation only for successful
  with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count()) as executor:
    futures = {executor.submit(make_cfgs, bench, f"{DATA_DIR}/llvm", modes) for bench, modes in successful.items()}
    for future in concurrent.futures.as_completed(futures):
      try:
        future.result()
      except Exception as e:
        print(f"CFG generation error: {e}")
        executor.shutdown(wait=False, cancel_futures=True)
        raise e

  # remove the tmp directory
  os.system(f"rm -rf {TMP_DIR}")

  elapsed = time.perf_counter() - start_time
  print(f"profile.py completed in {elapsed:.2f} seconds", flush=True)

