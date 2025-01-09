#!/usr/bin/env python3

import json
import random
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
import sys

RUN_MODES = ["llvm-O0-O0", "llvm-eggcc-O0-O0", "llvm-O3-O0"]
# copied from chart.js
COLOR_MAP = {
  "llvm-O0-O0" : "purple",
  "llvm-eggcc-O0-O0" : "pink",
  "llvm-O3-O0" : "gray"
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

def make_plot(profile, upper_x_bound, output):
  # Prepare the data for the jitter plot
  # first y label is empty, underneath the first benchmark
  y_labels = []
  y_data = []
  x_data = []
  colors = []

  filtered = [b for b in profile if b.get('runMethod', '') in RUN_MODES]

  grouped_by_benchmark = {}
  for benchmark in filtered:
      benchmark_name = benchmark.get('benchmark', '')
      if benchmark_name not in grouped_by_benchmark:
          grouped_by_benchmark[benchmark_name] = []
      grouped_by_benchmark[benchmark_name].append(benchmark)
  grouped_by_benchmark = [grouped_by_benchmark[benchmark] for benchmark in grouped_by_benchmark]

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

if __name__ == '__main__':
    # parse two arguments: the output folder and the profile.json file
    if len(sys.argv) != 3:
        print("Usage: python jitter.py <output_folder> <profile.json>")
        sys.exit(1)
    output_folder = sys.argv[1]
    profile_file = sys.argv[2]

    # Read profile.json from nightly/output/data/profile.json
    profile = []
    with open(profile_file) as f:
        profile = json.load(f)

    make_plot(profile, 4, f'{output_folder}/jitter_plot_max_4.png')
