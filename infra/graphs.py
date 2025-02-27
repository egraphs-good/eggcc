#!/usr/bin/env python3

import json
import random
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
import numpy as np
import sys
import os
import profile

EGGCC_NAME = "eggcc"

RUN_MODES = ["llvm-O0-O0", "llvm-eggcc-O0-O0", "llvm-O3-O0"]

if profile.TO_ABLATE != "":
  RUN_MODES.extend(["llvm-eggcc-ablation-O0-O0", "llvm-eggcc-ablation-O3-O0", "llvm-eggcc-ablation-O3-O3"])

# copied from chart.js
COLOR_MAP = {
  "rvsdg-round-trip-to-executable": "red",
  "llvm-O0-O0": "orange",
  "llvm-O1-O0": "green",
  "llvm-O2-O0": "black",
  "llvm-O3-O0": "purple",
  "llvm-O3-O3": "gold",
  "llvm-eggcc-O0-O0": "blue",
  "llvm-eggcc-sequential-O0-O0": "pink",
  "llvm-eggcc-O3-O0": "brown",
  "llvm-eggcc-O3-O3": "lightblue",
  "llvm-eggcc-ablation-O0-O0": "blue",
  "llvm-eggcc-ablation-O3-O0": "green",
  "llvm-eggcc-ablation-O3-O3": "orange",
}

SHAPE_MAP = {
  "rvsdg-round-trip-to-executable": "o",
  "llvm-O0-O0": "s",
  "llvm-O1-O0": "o",
  "llvm-O2-O0": "o",
  "llvm-O3-O0": "o",
  "llvm-O3-O3": "o",
  "llvm-eggcc-O0-O0": "o",
  "llvm-eggcc-sequential-O0-O0": "o",
  "llvm-eggcc-O3-O0": "o",
  "llvm-eggcc-O3-O3": "o",
  "llvm-eggcc-ablation-O0-O0": "o",
  "llvm-eggcc-ablation-O3-O0": "o",
  "llvm-eggcc-ablation-O3-O3": "o",
}

BENCHMARK_SPACE = 1.0 / len(RUN_MODES)
CIRCLE_SIZE = 15

RUN_MODE_Y_OFFSETS = []
for runMode in RUN_MODES:
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

def get_cycles(data, benchmark_name, run_method):
  return get_row(data, benchmark_name, run_method)['cycles']

def get_eggcc_compile_time(data, benchmark_name):
  return get_row(data, benchmark_name, 'llvm-eggcc-O0-O0')['eggccCompileTimeSecs']

def get_eggcc_extraction_time(data, benchmark_name):
  return get_row(data, benchmark_name, 'llvm-eggcc-O0-O0')['eggccExtractionTimeSecs']

def get_ilp_test_times(data, benchmark_name):
  row = get_row(data, benchmark_name, 'eggcc-ILP-O0-O0')
  return row['ilpTestTimes']

def group_by_benchmark(profile):
  grouped_by_benchmark = {}
  for benchmark in profile:
    benchmark_name = benchmark.get('benchmark', '')
    if benchmark_name not in grouped_by_benchmark:
      grouped_by_benchmark[benchmark_name] = []
    grouped_by_benchmark[benchmark_name].append(benchmark)
  return [grouped_by_benchmark[benchmark] for benchmark in grouped_by_benchmark]

# a graph of how the ilp solver time changes
# compared to the number of lines in the bril file
# when the ilp solve time is null it timed out
def make_ilp(json, output, benchmark_suite_folder):
  ilp_timeout = profile.ilp_extraction_test_timeout()

  eggcc_points = []
  ilp_timeout_points = []
  ilp_points = []

  benchmarks = dedup([b.get('benchmark') for b in json])

  for benchmark in benchmarks:
    # exclude raytrace, since it uses too much memory
    if benchmark == 'raytrace':
      continue
    # a list of ExtractionTimeSample
    ilp_test_times = get_ilp_test_times(json, benchmark)

    for sample in ilp_test_times:
      ilp_time = sample["ilp_time"]
      egraph_size = sample["egraph_size"]
      eggcc_time = sample["eggcc_time"]

      eggcc_time = eggcc_time["secs"] + eggcc_time["nanos"] / 1e9

      if ilp_time == None:
        ilp_timeout_points.append([egraph_size, ilp_timeout])
      else:
        ilp_points.append([egraph_size, ilp_time["secs"] + ilp_time["nanos"] / 1e9])
      eggcc_points.append([egraph_size, eggcc_time])
  
    # graph data
  plt.figure(figsize=(10, 8))

  psize = 350
  # Plot extraction time points
  eggcc_x, eggcc_y = zip(*eggcc_points) if eggcc_points else ([], [])
  plt.scatter(eggcc_x, eggcc_y, color='blue', label='EggCC Extraction Time', alpha=0.7, edgecolors='w', linewidth=0.5, s=psize)

  # Plot ILP timeout points
  ilp_timeout_x, ilp_timeout_y = zip(*ilp_timeout_points) if ilp_timeout_points else ([], [])
  plt.scatter(ilp_timeout_x, ilp_timeout_y, color='red', label='ILP Timeout', alpha=0.7, marker='x', s=psize)

  # Plot ILP solve time points
  ilp_x, ilp_y = zip(*ilp_points) if ilp_points else ([], [])
  plt.scatter(ilp_x, ilp_y, color='green', label='ILP Solve Time', alpha=0.7, edgecolors='w', linewidth=0.5, s=psize)

  fsize = 27
  plt.xlabel('Size of egraph', fontsize=fsize)
  plt.ylabel('Extraction Time', fontsize=fsize)
  plt.gca().xaxis.set_major_formatter(mticker.FuncFormatter(format_k))
  # slightly down
  plt.legend(fontsize=fsize, loc='upper right', bbox_to_anchor=(1, 0.9))
  plt.grid(True, linestyle='--', linewidth=0.5)

  # set axis font size
  plt.xticks(fontsize=fsize)
  plt.yticks(fontsize=fsize)

  # set x limit to 330 k
  plt.gca().set_xlim(left=0, right=330000)
  plt.tight_layout()

  plt.savefig(output)


# Format x-axis labels to be in "k" format
def format_k(x, pos):
    return f"{int(x / 1000)}k"

def make_jitter(profile, upper_x_bound, output):
  # Prepare the data for the jitter plot
  # first y label is empty, underneath the first benchmark
  y_labels = []
  y_data = []
  x_data = []
  colors = []

  filtered = [b for b in profile if b.get('runMethod', '') in RUN_MODES]

  grouped_by_benchmark = group_by_benchmark(filtered)

  # sort each group by runMethod
  for group in grouped_by_benchmark:
      group.sort(key=lambda b: RUN_MODES.index(b.get('runMethod', '')))

  # the order of the groups is the average cycles of the baseline
  grouped_by_benchmark.sort(key=lambda group: sum(group_cycles(group, BASELINE_TREATMENT)) / len(group))
  
  filtered = [benchmark for group in grouped_by_benchmark for benchmark in group]

      
  filtered = sorted(filtered, key=lambda b: b.get('benchmark', ''))

  # Assign numeric y values to each benchmark label
  y_label_map = {}
  outlier_x = []
  outlier_y = []

  for idx, benchmark in enumerate(filtered):
    benchmark_name = benchmark.get('benchmark', f'benchmark_{idx}')
    run_method = benchmark.get('runMethod', '')

    if benchmark_name not in y_label_map:
      y_label_map[benchmark_name] = len(y_labels)
      y_labels.append(benchmark_name)

    # Assign color for each runMethod
    if 'runMethod' not in benchmark:
      raise KeyError(f"Missing 'runMethod' field in benchmark: {benchmark_name}")
    color = COLOR_MAP[run_method]

    baseline_cycles = get_baseline_cycles(filtered, benchmark_name)
    baseline_mean = sum(baseline_cycles) / len(baseline_cycles)
    
    for cycle in benchmark.get('cycles', [])[:100]:
      normalized = cycle / baseline_mean
      # Add a small random jitter to y value to prevent overlap
      jittered_y = y_label_map[benchmark_name] + random.uniform(0.0, BENCHMARK_SPACE) + RUN_MODE_Y_OFFSETS[RUN_MODES.index(run_method)]
      if upper_x_bound != None and normalized > upper_x_bound:
          # Record outlier data
          outlier_x.append(upper_x_bound)
          outlier_y.append(jittered_y)
      else:
          # Normal data points
          x_data.append(normalized)
          y_data.append(jittered_y)
          colors.append(color)

  # Create the jitter plot
  # HACK: make the plot longer when we have more benchamrks
  plt.figure(figsize=(10, max(len(filtered) / (len(RUN_MODES)*2), 6)))
  plt.scatter(x_data, y_data, c=colors, alpha=0.7, edgecolors='w', linewidth=0.5, s=CIRCLE_SIZE)

  # Plot outliers as red 'x' marks
  if upper_x_bound:
    plt.scatter(outlier_x, outlier_y, color='red', marker='x', s=50, label=f'Outliers above {upper_x_bound}', alpha=0.9)

  # Use y labels on the minor ticks
  plt.yticks([a+0.5 for a in range(len(y_labels))], y_labels, rotation=0, ha='right')

  plt.ylabel('Benchmark')
  plt.xlabel('Cycles Normalized to Baseline Mean')
  plt.title('Jitter Plot of Benchmarks and Normalized Cycles')

  # Add horizontal lines at each tick
  for i in range(len(y_labels)):
      plt.axhline(y=i, color='gray', linestyle='--', linewidth=0.5)

  # Set x-axis to start at zero and display numbers instead of scientific notation
  plt.gca().set_xlim(left=0)

  # Create a legend based on runMethod
  handles = [plt.Line2D([0], [0], marker='o', color='w', markerfacecolor=COLOR_MAP[rm], markersize=10, alpha=0.7) for rm in COLOR_MAP]
  if upper_x_bound != None:
    handles.append(plt.Line2D([0], [0], marker='x', color='red', markersize=10, linestyle='None', label=f'Outliers above {upper_x_bound}'))
  paper_names = [to_paper_names_treatment(rm) for rm in COLOR_MAP]
  plt.legend(handles, paper_names + [f'Outliers above {upper_x_bound}'], title='Treatment', loc='upper right', bbox_to_anchor=(1.25, 1.05))

  # Save the plot to a PNG file in the nightly directory
  plt.tight_layout()
  plt.savefig(output)

def mean(lst):
  return sum(lst) / len(lst)

def normalized(profile, benchmark, treatment):
  baseline = get_baseline_cycles(profile, benchmark)
  treatment_cycles = get_cycles(profile, benchmark, treatment)
  return mean(treatment_cycles) / mean(baseline)

# make a bar chart given a profile.json
def make_normalized_chart(profile, output_file, treatments, y_max, width, height, xanchor, yanchor):
  # for each benchmark
  grouped_by_benchmark = group_by_benchmark(profile)
  sorted_by_eggcc = sorted(grouped_by_benchmark, key=lambda x: normalized(profile, x[0].get('benchmark'), treatments[0]))
  benchmarks = [group[0].get('benchmark') for group in sorted_by_eggcc]


  spacing = 0.2
  current_pos = 0

  fig, ax = plt.subplots()
  fig.set_size_inches(width, height)
  
  for benchmark in benchmarks:
    miny = 100000
    maxy = 0
    min_color = None

    for runmode in treatments:
      yval = normalized(profile, benchmark, runmode)

      if yval < miny:
        min_color = COLOR_MAP[runmode]
        miny = yval
      maxy = max(maxy, yval)

    # draw a line between the two points
    ax.plot([current_pos, current_pos], [miny, maxy], color=min_color, linestyle='--', linewidth=1, zorder=2)

    i = 0
    for runmode in treatments:
      yval = normalized(profile, benchmark, runmode)

      # for outliers, add x marks to the top
      if yval > y_max:
        jitter_amt = 0.05
        ax.text(current_pos + jitter_amt*i, y_max, 'x', ha='center', va='center', zorder=3, color=COLOR_MAP[runmode])
      else:
        ax.scatter(current_pos, yval, color=COLOR_MAP[runmode], s=CIRCLE_SIZE, zorder=3, marker=SHAPE_MAP[runmode])
      i += 1

    current_pos += spacing * 3

  ax.set_ylabel('Time Relative To LLVM-O3-O0')
  # ax.set_title('Normalized Cycles by Benchmark and Run Mode')
  # add a bar for each runmode, benchmark pair
  # ax.set_xticks(label_x + bar_w, benchmarks, rotation=45, ha='right')
  # turn off x labels
  ax.set_xticks([])
  ax.set_xticklabels([])

    
  
  # add the legend
  #handles = [plt.Rectangle((0,0),1,1, color=COLOR_MAP[rm]) for rm in treatments]
  handles = [plt.Line2D([0], [0], marker=SHAPE_MAP[rm], color='w', markerfacecolor=COLOR_MAP[rm], markersize=10, alpha=0.7) for rm in treatments]

  # add dotted line at 1.0 to handles
  handles.append(plt.Line2D([0], [0], color='gray', linestyle='--', linewidth=1.0, label='1.0'))

  treatmentsLegend = [f"{rm}" for rm in treatments]
  treatmentsLegend.append(BASELINE_TREATMENT)
  treatmentsLegend = [to_paper_names_treatment(t) for t in treatmentsLegend]


  ax.legend(handles, treatmentsLegend, title=
          'Treatment', loc='upper right', bbox_to_anchor=(xanchor, yanchor))

  ax.set_ylim(0.25, y_max)

  ax.set_xlim(-spacing, current_pos - spacing * 2)

  # add a dotted line at 1.0
  ax.axhline(y=1.0, color='gray', linestyle='--', linewidth=1.0)

  plt.tight_layout()
  plt.savefig(output_file)

# TODO change back after anonymization is lifted
def to_paper_names_treatment(treatment):
  if treatment == 'llvm-O0-O0':
    return 'LLVM-O0-O0'
  if treatment == 'llvm-O3-O0':
    return 'LLVM-O3-O0'
  if treatment == 'llvm-eggcc-O0-O0':
    return 'EQCC-O0-O0'
  if treatment == 'llvm-eggcc-O3-O0':
    return 'EQCC-O3-O0'
  if treatment == 'llvm-eggcc-ablation-O0-O0':
    return 'EQCC-Ablation-O0-O0'
  if treatment == 'llvm-eggcc-ablation-O3-O0':
    return 'EQCC-Ablation-O3-O0'
  if treatment == 'llvm-eggcc-ablation-O3-O3':
    return 'EQCC-Ablation-O3-O3'
  if treatment == 'rvsdg-round-trip-to-executable':
    return 'RVSDG-Executable'
  if treatment == 'llvm-O1-O0':
    return 'LLVM-O1-O0'
  if treatment == 'llvm-O2-O0':
    return 'LLVM-O2-O0'
  if treatment == 'llvm-O3-O3':
    return 'LLVM-O3-O3'
  if treatment == 'llvm-eggcc-sequential-O0-O0':
    return 'EQCC-Sequential-O0-O0'
  if treatment == 'llvm-eggcc-O3-O3':
    return 'EQCC-O3-O3'
  raise KeyError(f"Unknown treatment {treatment}")


def dedup(lst):
  return list(dict.fromkeys(lst))

def format_latex_macro(name, value):
  return f"\\newcommand{{\\{name}}}{{{value}\\xspace}}\n"

# given a ratio, format it as a percentage and create a latex macro
def format_latex_macro_percent(name, percent_as_ratio):
  percent = percent_as_ratio * 100
  return format_latex_macro(name, f"{percent:.2f}")

def benchmarks_in_folder(folder):
  # recursively find all files
  files = []
  for root, _, filenames in os.walk(folder):
    for filename in filenames:
      files.append(os.path.join(root, filename))
  # just get file name without extension
  return [os.path.splitext(os.path.basename(f))[0] for f in files]


# given a profile.json, list of suite paths, and an output file
def make_macros(profile, benchmark_suites, output_file):
  with open(output_file, 'a') as out:
    # report number of benchmarks in each benchmark suite
    for suite in benchmark_suites:
      suite_name = os.path.basename(suite)
      benchmarks = benchmarks_in_folder(suite)
      macro_name = f"Num{suite_name}Benchmarks"
      out.write(format_latex_macro(macro_name, len(benchmarks)))
    
    # report the number of benchmarks in the profile
    benchmarks = dedup([b.get('benchmark') for b in profile])
    out.write(format_latex_macro("NumBenchmarksAllSuites", len(benchmarks)))

    ilp_all = []
    for benchmark in benchmarks:
      # skip raytrace
      if benchmark == 'raytrace':
        continue
      ilp_all = ilp_all + get_ilp_test_times(profile, benchmark)

    ilp_all_above_100k = list(filter(lambda x: x["egraph_size"] > 100000, ilp_all))
    out.write(format_latex_macro_percent("PercentILPTimeout", len(list(filter(lambda x: x["ilp_time"] == None, ilp_all))) / len(ilp_all)))
    out.write(format_latex_macro_percent("PercentILPTimeoutAbove100k", len(list(filter(lambda x: x["ilp_time"] == None, ilp_all_above_100k))) / max(len(ilp_all_above_100k), 1)))
  

def get_code_size(benchmark, suites_path):
  # search for all files in the benchmark folder
  files = []
  for root, _, filenames in os.walk(suites_path):
    for filename in filenames:
      files.append(os.path.join(root, filename))
  
  file = False
  # find file with matching name to benchmark without extension
  # error if two files match
  for f in files:
    if os.path.splitext(os.path.basename(f))[0] == benchmark:
      if file:
        raise KeyError(f"Multiple files match benchmark {benchmark}")
      file = f
  
  if not file:
    raise KeyError(f"No file found for benchmark {benchmark}")
  
  # get the size of the file
  # if it's a bril file use lines without empty lines
  if file.endswith('.bril'):
    with open(file) as f:
      return len([line for line in f if line.strip()])
    
  # if it's a .rs files convert it to bril first with `cargo run --run-mode parse`
  if file.endswith('.rs'):
    popen_res = os.popen(f'cargo run {file} --run-mode parse')
    output_str = popen_res.read()
    error_code = popen_res.close()
    if error_code:
      raise KeyError(f"Failed to convert {file} to bril")

    return len([line for line in output_str.split('\n') if line.strip()])
  
  raise KeyError(f"Unsupported file type for benchmark {benchmark}: {file}")


def make_code_size_vs_compile_and_extraction_time(profile, compile_time_output, extraction_time_output, ratio_output, suites_path):
  benchmarks = dedup([b.get('benchmark') for b in profile])

  data = []
  for benchmark in benchmarks:
    compile_time = get_eggcc_compile_time(profile, benchmark)
    extraction_time = get_eggcc_extraction_time(profile, benchmark)
    code_size = get_code_size(benchmark, suites_path)
    if code_size > 300:
      continue
    data.append((code_size, compile_time, extraction_time))

  x = [d[0] for d in data]
  y1 = [d[1] for d in data]
  y2 = [d[2] for d in data]
  y3 = [d[2] / d[1] for d in data]

  # graph data
  plt.figure(figsize=(10, 6))
  plt.scatter(x, y1)
  plt.xlabel('Bril Number of Instructions')
  plt.ylabel(f'{EGGCC_NAME} Compile Time (s)')
  plt.title(f'{EGGCC_NAME} Compile Time vs Code Size')
  plt.savefig(compile_time_output)


  plt.figure(figsize=(10, 6))
  plt.scatter(x, y2)
  plt.xlabel('Bril Number of Instructions')
  plt.ylabel(f'{EGGCC_NAME} Extraction Time (s)')
  plt.title(f'{EGGCC_NAME} Extraction Time vs Code Size')
  plt.savefig(extraction_time_output)

  plt.figure(figsize=(10, 6))
  plt.scatter(x, y3)
  plt.xlabel('Bril Number of Instructions')
  plt.ylabel('Extraction Ratio')
  plt.title(f'{EGGCC_NAME} Compile Time vs Extraction Time')
  plt.savefig(ratio_output)



def make_graphs(output_folder, graphs_folder, profile_file, benchmark_suite_folder):
  # Read profile.json from nightly/output/data/profile.json
  profile = []
  with open(profile_file) as f:
      profile = json.load(f)
    
  # folders in 
  benchmark_suites = [f for f in os.listdir(benchmark_suite_folder) if os.path.isdir(os.path.join(benchmark_suite_folder, f))]
  benchmark_suites = [os.path.join(benchmark_suite_folder, f) for f in benchmark_suites]

  make_jitter(profile, 4, f'{graphs_folder}/jitter_plot_max_4.png')

  make_ilp(profile, f'{graphs_folder}/ilp_vs_lines.pdf', benchmark_suite_folder)

  for suite_path in benchmark_suites:
    suite = os.path.basename(suite_path)
    suite_benchmarks = benchmarks_in_folder(suite_path)
    profile_for_suite = [b for b in profile if b.get('benchmark') in suite_benchmarks]

    width = 10
    height = 4
    y_max = 2.0
    xanchor = 0.8
    yanchor = 0.4
    if suite == "polybench":
      y_max = 10.0
      width = 5
      height = 3.5
      xanchor = 0.4
      yanchor = 0.95

    make_normalized_chart(profile_for_suite, f'{graphs_folder}/{suite}_bar_chart.pdf', ["llvm-eggcc-O0-O0", "llvm-O0-O0"], y_max, width, height, xanchor, yanchor)

  make_macros(profile, benchmark_suites, f'{output_folder}/nightlymacros.tex')
  """
  make_code_size_vs_compile_and_extraction_time(
    profile, 
    f'{graphs_folder}/code_size_vs_compile_time.png', 
    f'{graphs_folder}/code_size_vs_extraction_time.png', 
    f'{graphs_folder}/extraction_ratio.png',
    benchmark_suite_folder)
    """

  # make json list of graph names and put in in output
  graph_names = []
  # read all files in the graphs folder
  for root, _, filenames in os.walk(graphs_folder):
    for filename in filenames:
      graph_names.append(filename)
  with open(f'{output_folder}/graphs.json', 'w') as f:
    json.dump(graph_names, f)

if __name__ == '__main__':
  # parse two arguments: the output folder and the profile.json file
  if len(sys.argv) != 4:
      print("Usage: python graphs.py <output_folder> <profile.json> <benchmark_suite_folder>")
      sys.exit(1)
  make_graphs(sys.argv[1], sys.argv[1] + '/graphs', sys.argv[2], sys.argv[3])

  
