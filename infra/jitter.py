#!/usr/bin/env python3

import json
import random
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
import sys

runModes = ["llvm-O0-O0", "llvm-eggcc-O0-O0"]
runModeYOffsets = []
for runMode in runModes:
  runModeYOffsets.append(len(runModeYOffsets) * 0.5)

def make_plot(profile, lower_x_bound, upper_x_bound, output):
  # Prepare the data for the jitter plot
  y_labels = []
  y_data = []
  x_data = []
  colors = []
  color_map = {}
  next_color = 0

  filtered = profile
  filtered = [b for b in profile if b.get('runMethod', '') in runModes]

  # Sort benchmarks by name
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
          # HACK: add a new line to move the benchmark name down
          y_labels.append("\n" + benchmark_name)

      # Assign color for each runMethod
      if 'runMethod' not in benchmark:
          raise KeyError(f"Missing 'runMethod' field in benchmark: {benchmark_name}")
      if run_method not in color_map:
          color_map[run_method] = f'C{next_color}'
          next_color += 1
      color = color_map[run_method]

      for cycle in benchmark.get('cycles', [])[:100]:
          # Add a small random jitter to y value to prevent overlap
          jittered_y = y_label_map[benchmark_name] + random.uniform(-0.2, 0.2) + runModeYOffsets[runModes.index(run_method)] - 0.2
          if cycle < lower_x_bound:
              outlier_x.append(lower_x_bound)
              outlier_y.append(jittered_y)
          elif upper_x_bound != None and cycle > upper_x_bound:
              # Record outlier data
              outlier_x.append(upper_x_bound)
              outlier_y.append(jittered_y)
          else:
              # Normal data points
              x_data.append(cycle)
              y_data.append(jittered_y)
              colors.append(color)

  # Create the jitter plot
  plt.figure(figsize=(10, max(len(filtered) / (len(runModes)*3), 6)))
  plt.scatter(x_data, y_data, c=colors, alpha=0.7, edgecolors='w', linewidth=0.5, s=15)

  # Plot outliers as red 'x' marks
  if upper_x_bound:
    plt.scatter(outlier_x, outlier_y, color='red', marker='x', s=50, label=f'Outliers not between {lower_x_bound} and {upper_x_bound} cycles', alpha=0.9)

  # Set the labels and title
  plt.yticks(range(len(y_labels)), y_labels, rotation=0, ha='right')
  plt.ylabel('Benchmark')
  plt.xlabel('Cycles')
  plt.title('Jitter Plot of Benchmarks and Cycles')

  # Add horizontal lines at each tick
  for i in range(len(y_labels)):
      plt.axhline(y=i, color='gray', linestyle='--', linewidth=0.5)

  # Set x-axis to start at zero and display numbers instead of scientific notation
  plt.gca().set_xlim(left=0)
  plt.gca().xaxis.set_major_formatter(mticker.FuncFormatter(lambda x, _: f'{int(x)}'))

  # Create a legend based on runMethod
  handles = [plt.Line2D([0], [0], marker='o', color='w', markerfacecolor=color_map[rm], markersize=10, alpha=0.7) for rm in color_map]
  if upper_x_bound != None:
    handles.append(plt.Line2D([0], [0], marker='x', color='red', markersize=10, linestyle='None', label=f'Outliers not between {lower_x_bound} and {upper_x_bound} cycles'))
  plt.legend(handles, list(color_map.keys()) + [f'Outliers not between {lower_x_bound} and {upper_x_bound} cycles'], title='Run Method', loc='upper right')

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

    make_plot(profile, 0, None, f'{output_folder}/jitter_plot_full_range.png')
    make_plot(profile, 0, 2000, f'{output_folder}/jitter_plot_2k_cycles.png')
    make_plot(profile, 0, 100000, f'{output_folder}/jitter_plot_100k_cycles.png')