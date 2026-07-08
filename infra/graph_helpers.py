import profile
import os
import math
EGGCC_NAME = "eggcc"
TIGER_NAME = "Statewalk DP"
TIGER_INLINE_NAME = "DP"

GRAPH_RUN_MODES = ["llvm-O0-O0", "eggcc-O0-O0", "llvm-O3-O0"]

# note: use ["..."] for indexing samples instead of .get(...) to fail fast on missing keys
# never hide errors about missing keys or anything like that
# if data is empty so you can't compute a number, don't default to 0, abort instead with a warning


if profile.TO_ABLATE != "":
  GRAPH_RUN_MODES.extend(["eggcc-ablation-O0-O0", "eggcc-ablation-O3-O0", "eggcc-ablation-O3-O3"])

# need ilp and graph run modes for this script to work
NECESSARY_MODES = GRAPH_RUN_MODES + ["eggcc-ILP-O0-O0"]

# Per-region ILP solver timeout (seconds) used to generate the current run's data.
# make_graphs sets this from the nightly config so graphs report the real timeout
# (e.g. "30 s" for the default nightly, "5 min" for paper). Defaults to 5 minutes.
_ILP_TIMEOUT_SECONDS = 5 * 60


def set_ilp_timeout_seconds(seconds):
  global _ILP_TIMEOUT_SECONDS
  _ILP_TIMEOUT_SECONDS = int(seconds)


def get_ilp_timeout_seconds():
  return _ILP_TIMEOUT_SECONDS


def format_timeout_label(seconds=None):
  """Human-readable timeout, e.g. '5 min' or '30 s'."""
  s = get_ilp_timeout_seconds() if seconds is None else int(seconds)
  if s % 60 == 0:
    return f"{s // 60} min"
  return f"{s} s"

# copied from chart.js
COLOR_MAP = {
  "rvsdg-round-trip-to-executable": "red",
  "llvm-O0-O0": "black",
  "llvm-O1-O0": "green",
  "llvm-O2-O0": "orange",
  "llvm-O3-O0": "purple",
  "llvm-O3-O3": "gold",
  "eggcc-O0-O0": "blue",
  "eggcc-sequential-O0-O0": "pink",
  "eggcc-O3-O0": "brown",
  "eggcc-O3-O3": "lightblue",
  "eggcc-ablation-O0-O0": "blue",
  "eggcc-ablation-O3-O0": "green",
  "eggcc-ablation-O3-O3": "orange",
  "eggcc-tiger-O0-O0": "green",
  "eggcc-tiger-WL-O0-O0": "magenta",
  "eggcc-tiger-ILP-O0-O0": "#3784ff",
  "eggcc-tiger-ILP-CBC-O0-O0": "olive",
  "eggcc-tiger-ILP-NOMIN-O0-O0": "darkgreen",
  "eggcc-tiger-ILP-WITHCTX-O0-O0": "orange",
  "eggcc-tiger-WITHCTX-O0-O0": "lightblue",
  "eggcc-tiger-nohacker-WITHCTX-O0-O0": "#009E73",
}

SHAPE_MAP = {
  "rvsdg-round-trip-to-executable": "o",
  "llvm-O0-O0": "s",
  "llvm-O1-O0": "o",
  "llvm-O2-O0": "o",
  "llvm-O3-O0": "o",
  "llvm-O3-O3": "o",
  "eggcc-O0-O0": "o",
  "eggcc-sequential-O0-O0": "o",
  "eggcc-O3-O0": "o",
  "eggcc-O3-O3": "o",
  "eggcc-ablation-O0-O0": "o",
  "eggcc-ablation-O3-O0": "o",
  "eggcc-ablation-O3-O3": "o",
  "eggcc-tiger-WL-O0-O0": "o",
  "eggcc-tiger-O0-O0": "o",
  "eggcc-tiger-ILP-O0-O0": "^",
  "eggcc-tiger-ILP-CBC-O0-O0": "^",
  "eggcc-tiger-ILP-NOMIN-O0-O0": "^",
  "eggcc-tiger-ILP-WITHCTX-O0-O0": "^",
  "eggcc-WITHCTX-O0-O0": "o",
  "eggcc-tiger-WITHCTX-O0-O0": "o",
  "eggcc-tiger-nohacker-WITHCTX-O0-O0": "o",
}

EXTRACTION_INSET_BOUNDS = (0.4, 0.3, 0.38 * 1.5, 0.35 * 1.5) # x y width height

BENCHMARK_SPACE = 1.0 / len(GRAPH_RUN_MODES)
CIRCLE_SIZE = 15

RUN_MODE_Y_OFFSETS = []
for runMode in GRAPH_RUN_MODES:
  RUN_MODE_Y_OFFSETS.append(len(RUN_MODE_Y_OFFSETS) * BENCHMARK_SPACE)


BASELINE_TREATMENT = 'llvm-O3-O0'


# rows has the same type as the profile.json file: a list of dictionaries
# however it should only contain rows for a single benchmark, with all the different runMethods
def group_cycles(rows, treatment):
  # assert only one row has a runMethod of llvm-O0-O0
  count = [row.get('runMethod', '') == treatment for row in rows].count(True)
  assert(count == 1)

  for row in rows:
      if row.get('runMethod', '') == treatment:
          return row.get('cycles', [])
  # throw exception if we don't have a baseline
  raise KeyError("Missing baseline in profile.json")


# given a profile.json, find the baseline cycles for the benchmark
def get_baseline_cycles(data, benchmark_name):
  group = [row for row in data if row.get('benchmark', '') == benchmark_name]
  return group_cycles(group, BASELINE_TREATMENT)

def get_row(data, benchmark_name, run_method):
  for row in data:
    if row.get('benchmark', '') == benchmark_name and row.get('runMethod', '') == run_method:
      return row
  raise KeyError(f"Missing benchmark {benchmark_name} with runMethod {run_method}")

# ILP extraction treatments whose timeouts/infeasibilities are drawn on the normalized
# charts. The chart uses the Gurobi one when Gurobi is available and the CBC one otherwise.
ILP_EXTRACTION_TREATMENTS = {"eggcc-tiger-ILP-O0-O0", "eggcc-tiger-ILP-CBC-O0-O0"}


def is_ilp_timeout(data, benchmark_name, run_method):
  if run_method not in ILP_EXTRACTION_TREATMENTS:
    return False
  for row in data:
    if row['benchmark'] == benchmark_name and row['runMethod'] == run_method:
      return row['ILPRegionTimeOut']

  raise KeyError(f"Missing benchmark {benchmark_name} with runMethod {run_method}")


def is_ilp_infeasible(data, benchmark_name, run_method):
  if run_method not in ILP_EXTRACTION_TREATMENTS:
    return False
  for row in data:
    if row['benchmark'] == benchmark_name and row['runMethod'] == run_method:
      return row["failed"] and ("ILP solver reported infeasibility" in row["error"])
  raise KeyError(f"Missing benchmark {benchmark_name} with runMethod {run_method}")


def has_run_cycles(data, benchmark_name, run_method):
  """True if this benchmark+treatment produced a non-empty cycles list.

  A treatment can fail without being a flagged region timeout or a reported
  infeasibility -- e.g. ILP extraction exceeds the wall-clock timeout, so the eggcc
  process is killed and no binary is produced (cycles == False). Charts must skip
  such treatments instead of averaging an empty list."""
  for row in data:
    if row.get('benchmark') == benchmark_name and row.get('runMethod') == run_method:
      cycles = row.get('cycles')
      return isinstance(cycles, list) and len(cycles) > 0
  return False

def get_cycles(data, benchmark_name, run_method):
  return get_row(data, benchmark_name, run_method)['cycles']

def get_eggcc_compile_time(data, benchmark_name):
  return get_row(data, benchmark_name, 'eggcc-O0-O0')['eggccCompileTimeSecs']

def get_eggcc_extraction_time(data, benchmark_name):
  return get_row(data, benchmark_name, 'eggcc-O0-O0')['eggccExtractionTimeSecs']

def get_extract_region_timings(treatment, data, benchmark_name):
  row = get_row(data, benchmark_name, treatment)
  return row['extractRegionTimings']

def group_by_benchmark(profile):
  grouped_by_benchmark = {}
  for benchmark in profile:
    benchmark_name = benchmark.get('benchmark', '')
    if benchmark_name not in grouped_by_benchmark:
      grouped_by_benchmark[benchmark_name] = []
    grouped_by_benchmark[benchmark_name].append(benchmark)
  return [grouped_by_benchmark[benchmark] for benchmark in grouped_by_benchmark]

def all_region_extract_points(treatment, data, benchmarks):
  res = []
  for benchmark in benchmarks:
    # a list of ExtractRegionTiming records
    timings = get_extract_region_timings(treatment, data, benchmark)
    if timings == False:
      print("WARNING: Skipping benchmark " + benchmark + " treatment " + treatment + " because it errored")
    else:
      res = res + timings

  return res


# The treatment that records per-region tiger/CBC/Gurobi timing samples.
COMPARISON_TREATMENT = "eggcc-tiger-ILP-COMPARISON"


def comparison_ran(data):
  """True if the COMPARISON treatment produced any region-timing samples.

  When true, tiger and CBC timing data are available for every sample, so all the
  tiger-only and CBC graphs can be generated (regardless of Gurobi)."""
  for row in data:
    if row.get("runMethod") != COMPARISON_TREATMENT:
      continue
    if row.get("extractRegionTimings"):
      return True
  return False


def has_gurobi_ilp_data(data):
  """True if any COMPARISON sample recorded a real Gurobi run.

  Samples set ilp_ran=False when Gurobi was skipped (e.g. gurobi_cl not installed).
  Older data predates the field, so a missing ilp_ran is treated as a Gurobi run."""
  for row in data:
    if row.get("runMethod") != COMPARISON_TREATMENT:
      continue
    timings = row.get("extractRegionTimings")
    if not timings:
      continue
    for sample in timings:
      if sample.get("ilp_ran", True):
        return True
  return False


def dedup(lst):
  return list(dict.fromkeys(lst))


def timeout_benchmarks_for_run(profile, run_method, suite=None):
  benchmarks_with_timeouts = {
    row["benchmark"]
    for row in profile
    if row["runMethod"] == run_method
    and row["ILPRegionTimeOut"]
    and (suite is None or row["suite"] == suite)
  }
  if None in benchmarks_with_timeouts:
    raise ValueError(
      f"Found benchmark with name None for run {run_method} and suite {suite}; data is malformed"
    )
  return benchmarks_with_timeouts

_DIGIT_WORDS = {
  "0": "Zero",
  "1": "One",
  "2": "Two",
  "3": "Three",
  "4": "Four",
  "5": "Five",
  "6": "Six",
  "7": "Seven",
  "8": "Eight",
  "9": "Nine",
}

def convert_string_to_valid_latex_var(name):
  text = str(name)
  sanitized_parts = []
  for ch in text:
    if ch.isdigit():
      sanitized_parts.append(_DIGIT_WORDS[ch])
    elif ch.isalpha():
      sanitized_parts.append(ch)
    # drop characters that LaTeX macro names can't include, like spaces or punctuation
  sanitized_name = ''.join(sanitized_parts)
  if not sanitized_name:
    sanitized_name = "Macro"
  elif not sanitized_name[0].isalpha():
    sanitized_name = f"Macro{sanitized_name}"
  return sanitized_name


def format_latex_macro(name, value, group_thousands=False):
  sanitized_name = convert_string_to_valid_latex_var(name)
  if group_thousands:
    if isinstance(value, bool) or not isinstance(value, int):
      raise ValueError(
        f"format_latex_macro(group_thousands=True) requires an integer value, got {type(value).__name__}"
      )
    macro_value = f"{value:,}"
  else:
    macro_value = str(value)
  return f"\\newcommand{{\\{sanitized_name}}}{{{macro_value}\\xspace}}\n"

# given a ratio, format it as a percentage and create a latex macro
def format_latex_macro_percent(name, percent_as_ratio, group_thousands=False):
  percent = percent_as_ratio * 100
  return format_latex_macro(name, f"{percent:.1f}\\%", group_thousands=group_thousands)

def benchmarks_in_folder(folder):
  # recursively find all files
  files = []
  for root, _, filenames in os.walk(folder):
    for filename in filenames:
      files.append(os.path.join(root, filename))

  # filter out README.md
  files = [f for f in files if os.path.basename(f).lower() != "readme.md"]
  # just get file name without extension
  return [os.path.splitext(os.path.basename(f))[0] for f in files]


def duration_to_seconds(duration):
  return float(duration["secs"]) + float(duration["nanos"]) / 1_000_000_000.0


def mean(values):
  if not values:
    raise ValueError("mean() requires at least one value")
  return sum(values) / len(values)


def geometric_mean(values):
  if not values:
    raise ValueError("geometric_mean() requires at least one value")
  log_sum = 0.0
  count = 0
  for value in values:
    if value <= 0:
      raise ValueError("geometric_mean() requires all values to be positive")
    log_sum += math.log(value)
    count += 1
  if count == 0:
    raise ValueError("geometric_mean() requires at least one positive value")
  return math.exp(log_sum / count)

