#!/usr/bin/env python3

import json
import os
import time
from glob import glob
from sys import stdout
import subprocess

import concurrent.futures

NUM_WARMUP_SAMPLES = 50
SAMPLES_PER_BENCHMARK_AND_TREATMENT = 200

treatments = [
  "rvsdg-round-trip-to-executable",
  #"cranelift-O3", currently disabled since it doesn't support measuring cycles yet
  "llvm-O0-O0",
  "llvm-O1-O0",
  "llvm-O2-O0",
  "llvm-eggcc-O0-O0",
  "llvm-eggcc-sequential-O0-O0",
  "llvm-O3-O0",
  "llvm-O3-O3",
  "llvm-eggcc-O3-O0",
  "llvm-eggcc-O3-O3",
]

# Where to output files that are needed for nightly report
DATA_DIR = None

# Where to write intermediate files that should be cleaned up at the end of this script
TMP_DIR = "tmp"

EGGCC_BINARY = "target/release/eggcc"


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
    case "llvm-eggcc-sequential-O0-O0":
      return (f'optimize --eggcc-schedule sequential', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "llvm-eggcc-O0-O0":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O0_O0')
    case "llvm-eggcc-O3-O0":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O0')
    case "llvm-eggcc-O3-O3":
      return (f'optimize', f'--run-mode llvm --optimize-egglog false --optimize-bril-llvm O3_O3')
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
  print(f'[{benchmark.index}/{benchmark.total}] Optimizing {benchmark.name} with {benchmark.treatment}')
  profile_dir = benchmark_profile_dir(benchmark.name)
  optimized_bril_file = f'{profile_dir}/{benchmark.name}-{benchmark.treatment}.bril'
  eggcc_run_data = f'{profile_dir}/{benchmark.treatment}-eggcc-run-data.json'
  llvm_run_data = f'{profile_dir}/{benchmark.treatment}-llvm-run-data.json'

  # get the commands we need to run
  (eggcc_run_mode, llvm_args) = get_eggcc_options(benchmark)
  # make the llvm output directory
  os.makedirs(f"{DATA_DIR}/llvm/{benchmark.name}/{benchmark.treatment}", exist_ok=True)
  llvm_out_file = f"{DATA_DIR}/llvm/{benchmark.name}/{benchmark.treatment}/optimized.ll"

  cmd1 = f'{EGGCC_BINARY} {benchmark.path} --run-mode {eggcc_run_mode} --run-data-out {eggcc_run_data}'
  cmd2 = f'{EGGCC_BINARY} {optimized_bril_file} --run-data-out {llvm_run_data} --add-timing {llvm_args} -o {profile_dir}/{benchmark.treatment} --llvm-output-dir {llvm_out_file}'

  print(f'Running c1: {cmd1}', flush=True)
  process = subprocess.run(cmd1, shell=True, capture_output=True, text=True)
  process.check_returncode()

  # write the std out to the optimized bril file
  with open(optimized_bril_file, 'w') as f:
    f.write(process.stdout)

  print(f'Running c2: {cmd2}', flush=True)
  process2 = subprocess.run(cmd2, shell=True)
  process2.check_returncode()

  eggcc_compile_time = 0
  # parse json from eggcc run data
  with open(eggcc_run_data) as f:
    eggcc_data = json.load(f)
    secs = eggcc_data["eggcc_compile_time"]["secs"]
    nanos = eggcc_data["eggcc_compile_time"]["nanos"]
    eggcc_compile_time = secs + nanos / 1e9
  
  llvm_compile_time = 0
  with open(llvm_run_data) as f:
    llvm_data = json.load(f)
    secs = llvm_data["llvm_compile_time"]["secs"]
    nanos = llvm_data["llvm_compile_time"]["nanos"]
    llvm_compile_time = secs + nanos / 1e9


  res = {"path": f"{profile_dir}/{benchmark.treatment}", "eggccCompileTimeSecs": eggcc_compile_time, "llvmCompileTimeSecs": llvm_compile_time}
  return res



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
      num_samples_so_far = 0
      resulting_num_cycles = []
      while True:
        args_str = " " + args if len(args) > 0 else ""
        cmd = f'{profile_dir}/{benchmark.treatment}{args_str}'
        result = subprocess.run(cmd, capture_output=True, shell=True)
        
        if result.returncode != 0:
          raise Exception(f'Error running {benchmark.name} with {benchmark.treatment}: {result.stderr}')
        res_cycles = int(result.stderr)
        resulting_num_cycles.append(res_cycles)

        num_samples_so_far += 1
        # if we have run for at least 1 second and we have at least 2 samples, stop
        #if time.time() - time_start > time_per_benchmark and len(resulting_num_cycles) >= 2:
         # break
        if num_samples_so_far >= SAMPLES_PER_BENCHMARK_AND_TREATMENT + NUM_WARMUP_SAMPLES:
          break
      # throw away the first NUM_WARMUP_SAMPLES samples
      resulting_num_cycles = resulting_num_cycles[NUM_WARMUP_SAMPLES:]

      return (f'{profile_dir}/{benchmark.treatment}', resulting_num_cycles)

# Run modes that we expect to output llvm IR
def should_have_llvm_ir(runMethod):
  return runMethod in [
    "rvsdg-round-trip-to-executable",
    "llvm-O0-O0",
    "llvm-O1-O0",
    "llvm-O2-O0",
    "llvm-eggcc-O0-O0",
    "llvm-eggcc-sequential-O0-O0",
    "llvm-O3-O0",
    "llvm-O3-O3",
    "llvm-eggcc-O3-O0",
  ]

def get_suite(path):
  # get the absolute path to the benchmark
  benchmark_path = os.path.abspath(path)
  suite_name = "unknown"
  
  while not os.path.basename(benchmark_path) == "passing":
    suite_name = os.path.basename(benchmark_path)
    print(f"Suite name: {suite_name}")
    # go up one dir
    benchmark_path = os.path.dirname(benchmark_path)

    # if we are at the root, break
    if benchmark_path == "/":
      suite_name = "unknown"
      break
  return suite_name
  
  

# aggregate all profile info into a single json array.
def aggregate(compile_data, bench_times, paths):
  res = []

  for path in sorted(compile_data.keys()):
    name = path.split("/")[-2]
    runMethod = path.split("/")[-1]
    result = {"runMethod": runMethod, "benchmark": name, "cycles": bench_times[path], "path": paths[name], "suite": get_suite(paths[name])}

    # add compile time info
    for key in compile_data[path]:
      result[key] = compile_data[path][key]

    res.append(result)
  return res

if __name__ == '__main__':
  # expect two arguments
  if len(os.sys.argv) != 3 and len(os.sys.argv) != 4:
    print("Usage: profile.py <output_directory> <bril_directory> <--parallel>")
    exit(1)

  # running benchmarks sequentially for more reliable results
  # can set this to true for testing
  isParallelBenchmark = False
  if len(os.sys.argv) == 4:
    if os.sys.argv[3] == "--parallel":
      isParallelBenchmark = True
    else:
      print("Usage: profile.py <output_directory> <bril_directory> <--parallel>")
      exit(1)

  # Create tmp directory for intermediate files
  try:
    os.mkdir(TMP_DIR)
  except FileExistsError:
    print(f"{TMP_DIR} exits, deleting contents")
    # remove the files in the directory
    os.system(f"rm -rf {TMP_DIR}/*")

  # build eggcc
  print("Building eggcc")
  buildres = os.system("cargo build --release")
  if buildres != 0:
    print("Failed to build eggcc")
    exit(1)



  DATA_DIR, bril_dir  = os.sys.argv[1:3]
  profiles = []
  # if it is a directory get all files
  if os.path.isdir(bril_dir):
    print(f'Running all bril files in {bril_dir}')
    profiles = glob(f'{bril_dir}/**/*.bril', recursive=True) + glob(f'{bril_dir}/**/*.rs', recursive=True)
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
  index = 0
  total = len(profiles) * len(treatments)
  for benchmark_path in profiles:
    for treatment in treatments:
      to_run.append(Benchmark(benchmark_path, treatment, index, total))
      index += 1

  benchmark_names = set([benchmark.name for benchmark in to_run])
  for benchmark_name in benchmark_names:
    setup_benchmark(benchmark_name)
  

  compile_data = {}
  # get the number of cores on this machine 
  parallelism = os.cpu_count()

  # create a thread pool for running optimization
  with concurrent.futures.ThreadPoolExecutor(max_workers = parallelism) as executor:
    futures = {executor.submit(optimize, benchmark) for benchmark in to_run}
    for future in concurrent.futures.as_completed(futures):
      try:
        res = future.result()
        path = res["path"]
        # remove the path from compile data
        res.pop("path")
        compile_data[path] = res
      except Exception as e:
        print(f"Shutting down executor due to error: {e}")
        executor.shutdown(wait=False, cancel_futures=True)
        raise e


  bench_data = {}
  if isParallelBenchmark:
    # create a thread pool for running benchmarks
    with concurrent.futures.ThreadPoolExecutor(max_workers = parallelism) as executor:
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

  nightly_data = aggregate(compile_data, bench_data, paths)
  with open(f"{DATA_DIR}/profile.json", "w") as profile:
    json.dump(nightly_data, profile, indent=2)

  # remove the tmp directory
  os.system(f"rm -rf {TMP_DIR}")

