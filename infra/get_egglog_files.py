import profile
import os, concurrent, subprocess

def get_egglog_file(benchmark_path, output_dir):
  basename_without_extension = os.path.basename(benchmark_path).split('.')[0]
  new_name = f'{output_dir}/{basename_without_extension}.egglog'
  cmd = f'{profile.EGGCC_BINARY} {benchmark_path} --run-mode egglog > {new_name}'
  print(f"Running {cmd}", flush=True)
  process = subprocess.run(cmd, shell=True, capture_output=True)
  process.check_returncode()

if __name__ == '__main__':
  if len(os.sys.argv) != 3:
    print("Usage: python serialize_all_egraphs.py <benchmark_dir> <output_dir>")
    os.sys.exit(1)
  
  # build eggcc
  print("Building eggcc")
  profile.build_eggcc()

  benchmark_dir = os.sys.argv[1]
  output_dir = os.sys.argv[2]

  # make output dir, error if exists
  if os.path.exists(output_dir):
    print(f"Output directory {output_dir} already exists")
    os.sys.exit(1)
  os.mkdir(output_dir)

  benchmarks = profile.all_benchmarks(benchmark_dir)
  print(f"Found {len(benchmarks)} benchmarks")

  with concurrent.futures.ThreadPoolExecutor(max_workers = os.cpu_count()) as executor:
    futures = {executor.submit(get_egglog_file, benchmark, output_dir) for benchmark in benchmarks}

    for future in concurrent.futures.as_completed(futures):
      try:
        future.result()
      except Exception as e:
        print(f"Exception: {e}")
        executor.shutdown(wait=False, cancel_futures=True)
        raise e

