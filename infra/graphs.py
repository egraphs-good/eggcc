#!/usr/bin/env python3

import json
import random
from collections import Counter
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from matplotlib.patches import Patch
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset
import numpy as np
import sys
import os
import profile

from graph_helpers import *
from statewalk_graphs import *
from extract_time_graph import *
from ilp_encoding_graph import *
from macros import *


# a graph of how the ilp solver time changes
# compared to the size of the egraph
# when the ilp solve time is null it timed out
def make_region_extract_plot(json, output, plot_ilp):
  benchmarks = dedup([b.get('benchmark') for b in json])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", json, benchmarks)

  eggcc_points = []
  ilp_points = []
  ilp_timeout_points = []
  ilp_infeasible_points = []

  for sample in points:
    extract_time = sample["extract_time_liveon_satelliteon"]
    egraph_size = sample["egraph_size"]
    ilp_solve_time = sample["ilp_extract_time"]
    ilp_infeasible = sample.get("ilp_infeasible", False)

    eggcc_points.append((egraph_size, extract_time["secs"] + extract_time["nanos"] / 1e9))

    if ilp_infeasible:
      if ilp_solve_time is None:
        ilp_infeasible_points.append((egraph_size, 5 * 60))
      else:
        ilp_time = ilp_solve_time["secs"] + ilp_solve_time["nanos"] / 1e9
        ilp_infeasible_points.append((egraph_size, ilp_time))
    elif ilp_solve_time is None:
      ilp_timeout_points.append((egraph_size, 5 * 60))
    else:
      ilp_time = ilp_solve_time["secs"] + ilp_solve_time["nanos"] / 1e9
      ilp_points.append((egraph_size, ilp_time))

  if plot_ilp and not (ilp_points or ilp_timeout_points or ilp_infeasible_points):
    print("WARNING: No ILP timing data found; skipping ILP scatter plot")
    return
  if not plot_ilp and not eggcc_points:
    print("WARNING: No Tiger extraction timing data found; skipping scatter plot")
    return

  plt.figure(figsize=(10, 8))

  psize = 150
  alpha = 0.2
  circleLineWidth = 1.0
  timeoutLineWidth = 3.0

  plotted_any = False

  if not plot_ilp:
    eggcc_x, eggcc_y = zip(*eggcc_points)
    plt.scatter(
      eggcc_x,
      eggcc_y,
      color='blue',
      label=f'{EGGCC_NAME} Extraction Time',
      s=psize,
      alpha=alpha,
      linewidths=circleLineWidth,
      edgecolors='blue',
    )
    plotted_any = True
  else:
    if ilp_points:
      ilp_x, ilp_y = zip(*ilp_points)
      plt.scatter(
        ilp_x,
        ilp_y,
        color='green',
        label="ILP Solve Time",
        alpha=alpha,
        s=psize,
        linewidths=circleLineWidth,
        edgecolors='green',
      )
      plotted_any = True
    if ilp_timeout_points:
      timeout_x, timeout_y = zip(*ilp_timeout_points)
      plt.scatter(
        timeout_x,
        timeout_y,
        color='red',
        marker='x',
        label="ILP Timeout (5 min)",
        alpha=alpha,
        s=psize,
        linewidths=timeoutLineWidth,
        edgecolors='red',
      )
      plotted_any = True
    if ilp_infeasible_points:
      infeasible_x, infeasible_y = zip(*ilp_infeasible_points)
      plt.scatter(
        infeasible_x,
        infeasible_y,
        color='orange',
        marker='x',
        label="ILP Infeasible",
        alpha=alpha,
        s=psize,
        linewidths=timeoutLineWidth,
        edgecolors='orange',
      )
      plotted_any = True

  if not plotted_any:
    print("WARNING: No data plotted in make_region_extract_plot")
    return

  fsize = 27
  plt.xlabel('Size of Regionalized e-graph', fontsize=fsize)
  ylabel = 'ILP Solve Time (Seconds)' if plot_ilp else 'Extraction Time (Seconds)'
  plt.ylabel(ylabel, fontsize=fsize)

  if plotted_any:
    plt.legend(fontsize=fsize, loc='upper right', bbox_to_anchor=(1, 1.3))

  plt.xticks(fontsize=fsize)
  plt.yticks(fontsize=fsize)

  plt.tight_layout()

  plt.savefig(output)


def _compute_extraction_histogram_bins(tiger_times, ilp_times, hist_min, hist_max, bin_count):
  if bin_count < 1:
    bin_count = 1
  if hist_max <= hist_min:
    hist_max = hist_min + 1.0

  bin_edges = np.linspace(hist_min, hist_max, bin_count + 1)

  tiger_counts = np.zeros(bin_count, dtype=int)
  ilp_counts = np.zeros(bin_count, dtype=int)
  if tiger_times:
    tiger_counts, _ = np.histogram(tiger_times, bins=bin_edges)
  if ilp_times:
    ilp_counts, _ = np.histogram(ilp_times, bins=bin_edges)

  bin_width = bin_edges[1] - bin_edges[0] if len(bin_edges) > 1 else 1.0
  tiger_width = bin_width * 0.45
  ilp_width = bin_width * 0.45

  tiger_lefts = bin_edges[:-1]
  ilp_lefts = bin_edges[:-1] + (bin_width - ilp_width)

  return {
    "bin_edges": bin_edges,
    "tiger_counts": tiger_counts,
    "ilp_counts": ilp_counts,
    "bin_width": bin_width,
    "tiger_width": tiger_width,
    "ilp_width": ilp_width,
    "tiger_lefts": tiger_lefts,
    "ilp_lefts": ilp_lefts,
    "hist_min": hist_min,
    "hist_max": hist_max,
  }


def make_extraction_time_histogram(data, output):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  extract_times = []
  ilp_times = []
  ilp_timeout_count = 0
  ilp_infeasible_count = 0

  for sample in points:
    extract_time = sample["extract_time_liveon_satelliteon"]
    extract_value = extract_time["secs"] + extract_time["nanos"] / 1e9
    extract_times.append(extract_value)

    ilp_time = sample["ilp_extract_time"]
    ilp_infeasible = sample.get("ilp_infeasible", False)
    if ilp_infeasible:
      ilp_infeasible_count += 1
      continue
    if sample["ilp_timed_out"]:
      ilp_timeout_count += 1
    else:
      ilp_value = ilp_time["secs"] + ilp_time["nanos"] / 1e9
      ilp_times.append(ilp_value)

  if not extract_times and not ilp_times and ilp_timeout_count == 0 and ilp_infeasible_count == 0:
    print("WARNING: No extraction timing data found; skipping histogram")
    return

  plt.figure(figsize=(10, 6))

  all_times = extract_times + ilp_times
  bin_count = 30
  hist_min = 0.0
  if all_times:
    hist_max = max(all_times)
  else:
    hist_max = 1.0

  histogram = _compute_extraction_histogram_bins(extract_times, ilp_times, hist_min, hist_max, bin_count)
  eggcc_counts = histogram["tiger_counts"]
  ilp_counts = histogram["ilp_counts"]
  eggcc_width = histogram["tiger_width"]
  ilp_width = histogram["ilp_width"]
  eggcc_lefts = histogram["tiger_lefts"]
  ilp_lefts = histogram["ilp_lefts"]
  bin_width = histogram["bin_width"]
  hist_max = histogram["hist_max"]

  legend_handles = []
  legend_labels = []

  eggcc_mask = eggcc_counts > 0
  if eggcc_mask.any():
    plt.bar(
      eggcc_lefts[eggcc_mask],
      eggcc_counts[eggcc_mask],
      width=eggcc_width,
      align='edge',
      color='blue',
      edgecolor='black',
      alpha=0.7,
    )
    legend_handles.append(Patch(facecolor='blue', edgecolor='black', alpha=0.7))
    legend_labels.append(f'{EGGCC_NAME} Extraction Time')

  ilp_mask = ilp_counts > 0
  if ilp_mask.any():
    plt.bar(
      ilp_lefts[ilp_mask],
      ilp_counts[ilp_mask],
      width=ilp_width,
      align='edge',
      color='green',
      edgecolor='black',
      alpha=0.7,
    )
    legend_handles.append(Patch(facecolor='green', edgecolor='black', alpha=0.7))
    legend_labels.append('ILP Solve Time')

  plt.xlabel('Time (Seconds)')
  plt.ylabel('Number of Regions')
  plt.title(f'Distribution of Extraction Times')

  xlim_right = hist_max
  special_width = bin_width * 0.4
  special_left = hist_max
  if ilp_timeout_count:
    plt.bar(
      special_left,
      ilp_timeout_count,
      width=special_width,
      color='red',
      edgecolor='black',
      align='edge',
      alpha=0.7,
    )
    legend_handles.append(Patch(facecolor='red', edgecolor='black', alpha=0.7))
    legend_labels.append('ILP Timeouts')
    special_left += special_width
    xlim_right = special_left
  if ilp_infeasible_count:
    plt.bar(
      special_left,
      ilp_infeasible_count,
      width=special_width,
      color='orange',
      edgecolor='black',
      align='edge',
      alpha=0.7,
    )
    legend_handles.append(Patch(facecolor='orange', edgecolor='black', alpha=0.7))
    legend_labels.append('ILP Infeasible')
    special_left += special_width
    xlim_right = special_left

  ax = plt.gca()

  def _format_histogram_tick(value, _pos):
    if value <= 0:
      return ''
    if value < 1:
      return f'{value:.2f}'.rstrip('0').rstrip('.')
    if value < 10:
      return f'{value:.1f}'.rstrip('0').rstrip('.')
    if value < 1000:
      return f'{value:g}'
    return f'{int(value):,}'

  hist_tick_formatter = mticker.FuncFormatter(_format_histogram_tick)

  ax.set_xlim(hist_min, xlim_right)
  ax.set_yscale('log')

  max_count = 0
  if eggcc_counts.size:
    max_count = max(max_count, int(eggcc_counts.max()))
  if ilp_counts.size:
    max_count = max(max_count, int(ilp_counts.max()))
  max_count = max(max_count, int(ilp_timeout_count), int(ilp_infeasible_count))
  if max_count == 0:
    max_count = 1

  ax.set_ylim(0, max_count * 1.1)
  ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True, prune=None))
  ax.yaxis.set_major_formatter(hist_tick_formatter)
  ax.yaxis.set_minor_locator(mticker.AutoMinorLocator())
  ax.xaxis.set_major_locator(mticker.MaxNLocator(nbins=12, prune=None, min_n_ticks=6))
  ax.xaxis.set_minor_locator(mticker.AutoMinorLocator())

  if extract_times:
    zoom_max_time = max(extract_times) * 1.1
    if zoom_max_time > 0:
      axins = ax.inset_axes(list(EXTRACTION_INSET_BOUNDS))

      inset_bin_count = max(bin_count * 2, 20)
      inset_hist = _compute_extraction_histogram_bins(
        [t for t in extract_times if t <= zoom_max_time],
        [t for t in ilp_times if t <= zoom_max_time],
        hist_min,
        zoom_max_time,
        inset_bin_count,
      )

      inset_tiger_counts = inset_hist["tiger_counts"]
      inset_ilp_counts = inset_hist["ilp_counts"]
      inset_tiger_lefts = inset_hist["tiger_lefts"]
      inset_ilp_lefts = inset_hist["ilp_lefts"]
      inset_tiger_width = inset_hist["tiger_width"]
      inset_ilp_width = inset_hist["ilp_width"]

      inset_tiger_mask = inset_tiger_counts > 0
      inset_ilp_mask = inset_ilp_counts > 0

      if inset_tiger_mask.any():
        axins.bar(
          inset_tiger_lefts[inset_tiger_mask],
          inset_tiger_counts[inset_tiger_mask],
          width=inset_tiger_width,
          align='edge',
          color='blue',
          edgecolor='black',
          alpha=0.7,
          zorder=2,
        )
      if inset_ilp_mask.any():
        axins.bar(
          inset_ilp_lefts[inset_ilp_mask],
          inset_ilp_counts[inset_ilp_mask],
          width=inset_ilp_width,
          align='edge',
          color='green',
          edgecolor='black',
          alpha=0.7,
          zorder=2,
        )

      axins.set_xlim(hist_min, zoom_max_time)
      axins.set_yscale('log')
      axins.yaxis.set_major_formatter(hist_tick_formatter)
      inset_max_count = 0
      if inset_tiger_mask.any():
        inset_max_count = max(inset_max_count, int(inset_tiger_counts[inset_tiger_mask].max()))
      if inset_ilp_mask.any():
        inset_max_count = max(inset_max_count, int(inset_ilp_counts[inset_ilp_mask].max()))
      if inset_max_count == 0:
        inset_max_count = 1
      axins.set_ylim(0, inset_max_count * 1.1)
      axins.yaxis.set_major_locator(mticker.MaxNLocator(integer=True, prune=None))
      axins.yaxis.set_minor_locator(mticker.AutoMinorLocator())
      axins.tick_params(axis='both', labelsize=8)

      axins.set_title(f'Zoomed (0-{zoom_max_time:.2f} sec)', fontsize=9)

      connectors = mark_inset(
        ax,
        axins,
        loc1=1,
        loc2=4,
        fc='none',
        ec='black',
        linewidth=1.2,
      )
      for connector in connectors:
        connector.set_color('black')
        connector.set_alpha(0.9)
        connector.set_linewidth(1.2)

  if legend_handles:
    plt.legend(legend_handles, legend_labels)

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

  filtered = [b for b in profile if b.get('runMethod', '') in GRAPH_RUN_MODES]

  grouped_by_benchmark = group_by_benchmark(filtered)

  # sort each group by runMethod
  for group in grouped_by_benchmark:
      group.sort(key=lambda b: GRAPH_RUN_MODES.index(b.get('runMethod', '')))

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
      jittered_y = y_label_map[benchmark_name] + random.uniform(0.0, BENCHMARK_SPACE) + RUN_MODE_Y_OFFSETS[GRAPH_RUN_MODES.index(run_method)]
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
  plt.figure(figsize=(10, max(len(filtered) / (len(GRAPH_RUN_MODES)*2), 6)))
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
      if (not is_ilp_timeout(profile, benchmark, runmode)) and (not is_ilp_infeasible(profile, benchmark, runmode)):
        yval = normalized(profile, benchmark, runmode)
        if yval < miny:
          min_color = COLOR_MAP[runmode]
          miny = yval
        maxy = max(maxy, yval)

    # draw a line between the points
    ax.plot([current_pos, current_pos], [miny, maxy], color=min_color, linestyle='--', linewidth=1, zorder=2)

    i = 0
    for runmode in treatments:
      if is_ilp_timeout(profile, benchmark, runmode):
        # for timeouts, add x marks to the top
        jitter_amt = 0.05
        ax.text(current_pos + jitter_amt*i, y_max, 'x', ha='center', va='center', zorder=3, color="red")
        i += 1
        continue

      if is_ilp_infeasible(profile, benchmark, runmode):
        # for infeasibles, add x marks to the top
        jitter_amt = 0.05
        ax.text(current_pos + jitter_amt*i, y_max, 'x', ha='center', va='center', zorder=3, color="orange")
        i += 1
        continue
      
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
  if treatment == 'eggcc-O0-O0':
    return 'EQCC-O0-O0'
  if treatment == 'eggcc-O3-O0':
    return 'EQCC-O3-O0'
  if treatment == 'eggcc-ablation-O0-O0':
    return 'EQCC-Ablation-O0-O0'
  if treatment == 'eggcc-ablation-O3-O0':
    return 'EQCC-Ablation-O3-O0'
  if treatment == 'eggcc-ablation-O3-O3':
    return 'EQCC-Ablation-O3-O3'
  if treatment == 'rvsdg-round-trip-to-executable':
    return 'RVSDG-Executable'
  if treatment == 'llvm-O1-O0':
    return 'LLVM-O1-O0'
  if treatment == 'llvm-O2-O0':
    return 'LLVM-O2-O0'
  if treatment == 'llvm-O3-O3':
    return 'LLVM-O3-O3'
  if treatment == 'eggcc-sequential-O0-O0':
    return 'EQCC-Sequential-O0-O0'
  if treatment == 'eggcc-O3-O3':
    return 'EQCC-O3-O3'
  if treatment == 'eggcc-ILP-O0-O0':
    return 'EQCC-ILP-O0-O0'
  if treatment == 'eggcc-WITHCTX-O0-O0':
    return 'EQCC-WITHCTX-O0-O0'
  if treatment == 'eggcc-tiger-O0-O0':
    return 'EQCC-Tiger-O0-O0'
  if treatment == 'eggcc-tiger-WL-O0-O0':
    return 'EQCC-Tiger-WL-O0-O0'
  if treatment == 'eggcc-tiger-ILP-O0-O0':
    return 'EQCC-Tiger-ILP-O0-O0'
  if treatment == 'eggcc-tiger-ILP-CBC-O0-O0':
    return 'EQCC-Tiger-ILP-CBC-O0-O0'
  if treatment == 'eggcc-tiger-ILP-WITHCTX-O0-O0':
    return 'EQCC-Tiger-ILP-WITHCTX-O0-O0'
  if treatment == 'eggcc-tiger-ILP-NOMIN-O0-O0':
    return 'EQCC-Tiger-ILP-NOMIN-O0-O0'
  if treatment == 'eggcc-tiger-ILP-COMPARISON':
    return 'EQCC-Tiger-ILP-Comparison'
  raise KeyError(f"Unknown treatment {treatment}")



  

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

  make_region_extract_plot(profile, f'{graphs_folder}/egraph_size_vs_tiger_time.pdf', plot_ilp=False)
  make_region_extract_plot(profile, f'{graphs_folder}/egraph_size_vs_ILP_time.pdf', plot_ilp=True)
  make_extraction_time_histogram(profile, f'{graphs_folder}/extraction_time_histogram.pdf')
  make_extraction_time_cdf(profile, f'{graphs_folder}/extraction_time_cdf.pdf', use_log_x=True, use_exp_y=False)
  make_extraction_time_cdf(
    profile,
    f'{graphs_folder}/extraction_time_cdf_linear.pdf',
    use_log_x=False,
    use_exp_y=False,
  )
  make_extraction_time_cdf(
    profile,
    f'{graphs_folder}/extraction_time_cdf_exp_y.pdf',
    use_log_x=False,
    use_exp_y=True,
  )
  #make_ilp_encoding_scatter(
  #  profile,
  #  f'{graphs_folder}/ilp_encoding_vs_egraph_size.pdf',
  #)
  statewalk_histogram_max_width = None
  statewalk_histogram_treatment = "eggcc-tiger-ILP-COMPARISON"


  make_statewalk_width_histogram(
    profile,
    f'{graphs_folder}/statewalk_width_histogram_with_liveness.pdf',
    True,
    is_average=False,
    max_width=statewalk_histogram_max_width,
  )
  make_statewalk_width_histogram(
    profile,
    f'{graphs_folder}/statewalk_width_histogram.pdf',
    False,
    is_average=False,
    max_width=statewalk_histogram_max_width,
  )
  print_top_statewalk_width_samples(
    profile,
    statewalk_histogram_treatment,
    is_liveon=False,
    is_average=False,
    max_width=statewalk_histogram_max_width,
  )

  make_statewalk_width_performance_scatter(profile, f'{graphs_folder}/statewalk_width_vs_tiger_time.pdf', plot_ilp=False, is_liveon=False, is_average=False, scale_by_egraph_size=False)
  make_statewalk_width_performance_scatter(profile, f'{graphs_folder}/statewalk_width_vs_ILP_time.pdf', plot_ilp=True, is_liveon=False, is_average=False, scale_by_egraph_size=False)
  make_statewalk_width_performance_scatter(profile, f'{graphs_folder}/statewalk_width_times_size_vs_tiger_time.pdf', plot_ilp=False, is_liveon=False, is_average=False, scale_by_egraph_size=True)
  make_statewalk_width_performance_scatter(profile, f'{graphs_folder}/statewalk_width_times_size_vs_ILP_time.pdf', plot_ilp=True, is_liveon=False, is_average=False, scale_by_egraph_size=True)
  make_egraph_size_vs_statewalk_width_heatmap(
    profile,
    f'{graphs_folder}/heatmap_tiger_time_with_egraph_size_vs_statewalk_width.pdf',
    is_liveon=False,
    is_average=False,
    min_width=1,
  )
  make_egraph_size_vs_statewalk_width_heatmap(
    profile,
    f'{graphs_folder}/heatmap_ilp_time_with_egraph_size_vs_statewalk_width.pdf',
    is_liveon=False,
    is_average=False,
    min_width=1,
    runtime_source="ilp",
  )
  make_egraph_size_vs_statewalk_width_heatmap(
    profile,
    f'{graphs_folder}/heatmap_tiger_time_with_egraph_size_vs_statewalk_width_max100.pdf',
    is_liveon=False,
    is_average=False,
    min_width=1,
    max_width=100,
  )
  make_egraph_size_vs_statewalk_width_heatmap(
    profile,
    f'{graphs_folder}/heatmap_ilp_time_with_egraph_size_vs_statewalk_width_max100.pdf',
    is_liveon=False,
    is_average=False,
    min_width=1,
    max_width=100,
    runtime_source="ilp",
  )
  
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

    make_normalized_chart(profile_for_suite, f'{graphs_folder}/normalized_binary_perf_chart_{suite}.pdf', ["eggcc-tiger-O0-O0", "eggcc-tiger-ILP-O0-O0", "llvm-O0-O0"], y_max, width, height, xanchor, yanchor)

  make_macros(profile, benchmark_suites, f'{graphs_folder}/nightlymacros.tex')

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
  if len(sys.argv) != 5:
      print("Usage: python graphs.py <nightly_output_folder> <graphs_folder> <profile.json> <benchmark_suite_folder>")
      sys.exit(1)

  # if UNSAFE_TREATMENTS is true and TREATMENTS isn't a subset of treatments, exit
  if profile.UNSAFE_TREATMENTS and not set(NECESSARY_MODES).issubset(set(profile.treatments)):
      print("Skipping graphing: NECESSARY_MODES is true and GRAPH_RUN_MODES is not a subset of treatments")
      sys.exit(0)
  make_graphs(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4])


