#!/usr/bin/env python3

import json
import random
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
import numpy as np
import sys
import os

RUN_MODES = ["llvm-O0-O0", "llvm-eggcc-O0-O0", "llvm-O3-O0"]
BAR_CHART_RUN_MODES = ["llvm-O3-O3", "llvm-O3-O0", "llvm-eggcc-O0-O0"]
# copied from chart.js
COLOR_MAP = {
  "llvm-O0-O0" : "purple",
  "llvm-eggcc-O0-O0" : "pink",
  "llvm-O3-O0" : "gray",
  "llvm-O3-O3": "gold",
}
BENCHMARK_SPACE = 1.0 / len(RUN_MODES)
CIRCLE_SIZE = 15

RUN_MODE_Y_OFFSETS = []
for runMode in RUN_MODES:
  RUN_MODE_Y_OFFSETS.append(len(RUN_MODE_Y_OFFSETS) * BENCHMARK_SPACE)



# rows has the same type as the profile.json file: a list of dictionaries
# however it should only contain rows for a single benchmark, with all the different runMethods
def group_baseline_cycles(rows):
  # assert only one row has a runMethod of llvm-O0-O0
  count = [row.get('runMethod', '') == 'llvm-O0-O0' for row in rows].count(True)
  assert(count == 1)

  for row in rows:
      if row.get('runMethod', '') == 'llvm-O0-O0':
          return row.get('cycles', [])
  # throw exception if we don't have a baseline
  raise KeyError("Missing baseline in profile.json")


# given a profile.json, find the baseline cycles for the benchmark
def get_baseline_cycles(data, benchmark_name):
  group = [row for row in data if row.get('benchmark', '') == benchmark_name]
  return group_baseline_cycles(group)

def get_row(data, benchmark_name, run_method):
  for row in data:
    if row.get('benchmark', '') == benchmark_name and row.get('runMethod', '') == run_method:
      return row
  raise KeyError(f"Missing benchmark {benchmark_name} with runMethod {run_method}")

def get_cycles(data, benchmark_name, run_method):
  return get_row(data, benchmark_name, run_method).get('cycles')

def get_eggcc_compile_time(data, benchmark_name):
  return get_row(data, benchmark_name, 'llvm-eggcc-O0-O0').get('eggccCompileTimeSecs')

def group_by_benchmark(profile):
  grouped_by_benchmark = {}
  for benchmark in profile:
    benchmark_name = benchmark.get('benchmark', '')
    if benchmark_name not in grouped_by_benchmark:
      grouped_by_benchmark[benchmark_name] = []
    grouped_by_benchmark[benchmark_name].append(benchmark)
  return [grouped_by_benchmark[benchmark] for benchmark in grouped_by_benchmark]

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
  grouped_by_benchmark.sort(key=lambda group: sum(group_baseline_cycles(group)) / len(group))
  
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
  plt.legend(handles, list(COLOR_MAP.keys()) + [f'Outliers above {upper_x_bound}'], title='Run Method', loc='upper right', bbox_to_anchor=(1.25, 1.05))

  # Save the plot to a PNG file in the nightly directory
  plt.tight_layout()
  plt.savefig(output)

def mean(lst):
  return float(sum(lst)) / float(len(lst))

def normalized(profile, benchmark, treatment):
  baseline = get_baseline_cycles(profile, benchmark)
  treatment_cycles = get_cycles(profile, benchmark, treatment)
  return mean(treatment_cycles) / mean(baseline)

# make a bar chart given a profile.json
def make_bar_chart(profile, output_file):
  # for each benchmark
  grouped_by_benchmark = group_by_benchmark(profile)
  sorted_by_llvm_O3_O0 = sorted(grouped_by_benchmark, key=lambda x: normalized(profile, x[0].get('benchmark'), 'llvm-O3-O0'))
  benchmarks = [group[0].get('benchmark') for group in sorted_by_llvm_O3_O0]


  # add a bar for each runmode, benchmark pair
  label_x = np.arange(len(benchmarks))
  bar_w = 0.25
  current_pos = 0

  y_vals = []
  x_vals = []
  x_colors = []

  fig, ax = plt.subplots()
  fig.set_size_inches(15, 6)
  
  for benchmark in benchmarks:
    baseline_cycles = get_baseline_cycles(profile, benchmark)
    for runmode in BAR_CHART_RUN_MODES:
      y_vals.append(normalized(profile, benchmark, runmode))
      x_vals.append(current_pos)
      x_colors.append(COLOR_MAP[runmode])
      current_pos += bar_w
    current_pos += bar_w

  ax.set_ylabel('Normalized Cycles')
  ax.set_title('Normalized Cycles by Benchmark and Run Mode')
  ax.set_xticks(label_x + bar_w, benchmarks, rotation=45, ha='right')

  # add the bars
  for idx, val in enumerate(y_vals):
    ax.bar(x_vals[idx], val, bar_w, color=x_colors[idx])
  
  # add the legend
  handles = [plt.Rectangle((0,0),1,1, color=COLOR_MAP[rm]) for rm in BAR_CHART_RUN_MODES]
  ax.legend(handles, BAR_CHART_RUN_MODES, title='Run Mode', loc='upper right', bbox_to_anchor=(1.25, 1.05))

  # make a max of 1.5 slowdown
  ax.set_ylim(0, 1.5)

  # add a dotted line at 1.0
  ax.axhline(y=1.0, color='gray', linestyle='--', linewidth=0.5)

  # for outliers, add x marks to the top
  for idx, val in enumerate(y_vals):
    if val > 1.5:
      ax.text(x_vals[idx], 1.5, 'x', color='red', ha='center', va='center')


  plt.tight_layout()
  plt.savefig(output_file)

def dedup(lst):
  return list(dict.fromkeys(lst))

def format_latex_macro(name, value):
  return f"\\newcommand{{\\{name}}}{{{value}}}\n"

# given a ratio, format it as a percentage and create a latex macro
def format_latex_macro_percent(name, percent_as_ratio):
  percent = percent_as_ratio * 100
  return format_latex_macro(name, f"{percent:.2f}")

def make_macros(profile, output_file):
  number_recover_80_percent_performance_improvement_eggcc_vs_llvm_O3_O0 = 0
  benchmarks = dedup([b.get('benchmark') for b in profile])

  for benchmark in benchmarks:
    baseline_cycles = get_baseline_cycles(profile, benchmark)
    llvm_O3_O0_cycles = get_cycles(profile, benchmark, 'llvm-O3-O0')
    eggcc_O0_O0_cycles = get_cycles(profile, benchmark, 'llvm-eggcc-O0-O0')

    perf_improvement_llvm = mean(baseline_cycles) - mean(llvm_O3_O0_cycles)
    perf_improvement_eggcc = mean(baseline_cycles) - mean(eggcc_O0_O0_cycles)
    if perf_improvement_llvm < 0:
      number_recover_80_percent_performance_improvement_eggcc_vs_llvm_O3_O0 += 1
    else:
      if perf_improvement_eggcc > 0.8 * perf_improvement_llvm:
        number_recover_80_percent_performance_improvement_eggcc_vs_llvm_O3_O0 += 1
  ratio_recover_80_percent_performance_improvement_eggcc_vs_llvm_O3_O0 = number_recover_80_percent_performance_improvement_eggcc_vs_llvm_O3_O0 / len(benchmarks)

  with open(output_file, 'w') as f:
    f.write(format_latex_macro_percent('percentRecoverEightyPercentPerformanceImprovementEggccVsLlvmOThreeOZero', ratio_recover_80_percent_performance_improvement_eggcc_vs_llvm_O3_O0))



def benchmarks_in_folder(folder):
  # recursively find all files
  files = []
  for root, _, filenames in os.walk(folder):
    for filename in filenames:
      files.append(os.path.join(root, filename))
  # just get file name without extension
  return [os.path.splitext(os.path.basename(f))[0] for f in files]
  

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


def make_xy_graph(profile, output):
  y_axis_treatment = 'llvm-eggcc-O0-O0'
  x_axis_treatment = 'llvm-O3-O0'

  benchmarks = dedup([b.get('benchmark') for b in profile])

  data = []
  for benchmark in benchmarks:
    x = normalized(profile, benchmark, x_axis_treatment)
    y = normalized(profile, benchmark, y_axis_treatment)
    data.append((x, y))
  
  x = [d[0] for d in data]
  y = [d[1] for d in data]
  print(x)
  print(y)
  # graph data
  plt.figure(figsize=(10, 10))
  plt.scatter(x, y)
  plt.xlabel(f'{x_axis_treatment} Cycles (Normliazed to LLVM-O0-O0)')
  plt.ylabel(f'{y_axis_treatment} Cycles (Normalized to LLVM-O0-O0)')
  plt.title(f'{y_axis_treatment} vs {x_axis_treatment}')

  # set max bounds to be the same
  max_val = max(max(x), max(y))
  # set max bound
  plt.xlim(0, 1.4)
  plt.ylim(0, 1.4)

  # show outliers as red x marks
  for idx, val in enumerate(x):
    if val > 1.4 or y[idx] > 1.4:
      plt.text(min(val, 1.4), min(y[idx], 1.4), 'x', color='red', ha='center', va='center')
      


  # add a line for the diagonal
  plt.plot([0, max_val], [0, max_val], color='gray', linestyle='--', linewidth=0.5)

  # save the graph
  plt.savefig(output)

def make_code_size_vs_compile_time(profile, output, suites_path):
  benchmarks = dedup([b.get('benchmark') for b in profile])

  data = []
  for benchmark in benchmarks:
    compile_time = get_eggcc_compile_time(profile, benchmark)
    code_size = get_code_size(benchmark, suites_path)
    data.append((code_size, compile_time))

  x = [d[0] for d in data]
  y = [d[1] for d in data]

  # graph data
  plt.figure(figsize=(10, 6))
  plt.scatter(x, y)
  plt.xlabel('Bril Number of Instructions')
  plt.ylabel('EggCC Compile Time (s)')
  plt.title('EggCC Compile Time vs Code Size')
  plt.savefig(output)




if __name__ == '__main__':
  # parse two arguments: the output folder and the profile.json file
  if len(sys.argv) != 4:
      print("Usage: python graphs.py <output_folder> <profile.json> <benchmark_suite_folder>")
      sys.exit(1)
  output_folder = sys.argv[1]
  graphs_folder = output_folder + '/graphs'
  profile_file = sys.argv[2]
  benchmark_suite_folder = sys.argv[3]

  # Read profile.json from nightly/output/data/profile.json
  profile = []
  with open(profile_file) as f:
      profile = json.load(f)

  # folders in 
  benchmark_suites = [f for f in os.listdir(benchmark_suite_folder) if os.path.isdir(os.path.join(benchmark_suite_folder, f))]

  make_jitter(profile, 4, f'{graphs_folder}/jitter_plot_max_4.png')

  for suite in benchmark_suites:
    suite_path = os.path.join(benchmark_suite_folder, suite)
    suite_benchmarks = benchmarks_in_folder(suite_path)
    profile_for_suite = [b for b in profile if b.get('benchmark') in suite_benchmarks]
    make_bar_chart(profile_for_suite, f'{graphs_folder}/{suite}_bar_chart.png')

  make_macros(profile, f'{output_folder}/nightlymacros.tex')

  make_code_size_vs_compile_time(profile, f'{graphs_folder}/code_size_vs_compile_time.png', benchmark_suite_folder)

  make_xy_graph(profile, f'{graphs_folder}/xy_graph_eggcc_llvm.png')

  # make json list of graph names and put in in output
  graph_names = []
  # read all files in the graphs folder
  for root, _, filenames in os.walk(graphs_folder):
    for filename in filenames:
      graph_names.append(filename)
  with open(f'{output_folder}/graphs.json', 'w') as f:
    json.dump(graph_names, f)

  
