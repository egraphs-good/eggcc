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
from matplotlib.ticker import FuncFormatter

from profile import NightlyConfig
from graph_helpers import *
from statewalk_graphs import *
from extract_time_graph import *
from ilp_encoding_graph import *
from peggy_comparison_graph import make_peggy_comparison_graph
from macros import *


# note: use ["..."] for indexing samples instead of .get(...) to fail fast on missing keys

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
        ilp_infeasible_points.append((egraph_size, get_ilp_timeout_seconds()))
      else:
        ilp_time = ilp_solve_time["secs"] + ilp_solve_time["nanos"] / 1e9
        ilp_infeasible_points.append((egraph_size, ilp_time))
    elif ilp_solve_time is None:
      ilp_timeout_points.append((egraph_size, get_ilp_timeout_seconds()))
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
        label=f"ILP Timeout ({format_timeout_label()})",
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


def make_fenwick_cycles_bar_chart(data, output):
  benchmark = "fenwick_tree"
  title_fontsize = 18
  axis_label_fontsize = 16
  tick_fontsize = 13
  treatments = [
    "eggcc-tiger-WITHCTX-O0-O0",
    "llvm-O3-O3",
    "llvm-O3-O0",
    "llvm-O0-O0",
    "eggcc-tiger-nohacker-WITHCTX-O0-O0",
  ]
  labels = [to_paper_names_treatment(treatment) for treatment in treatments]

  means_millions = []
  stddevs_millions = []
  colors = []

  for treatment in treatments:
    row = get_row(data, benchmark, treatment)
    cycles = row["cycles"]
    if row["failed"] or not cycles:
      print(f"WARNING: Skipping Fenwick cycles bar chart because {benchmark} {treatment} has no cycle data")
      return

    means_millions.append(float(np.mean(cycles)) / 1e6)
    stddevs_millions.append(float(np.std(cycles)) / 1e6)
    colors.append(COLOR_MAP.get(treatment, "gray"))

  fig, ax = plt.subplots(figsize=(5.25, 6))
  x_positions = np.arange(len(labels))
  ax.bar(
    x_positions,
    means_millions,
    yerr=stddevs_millions,
    color=colors,
    edgecolor="black",
    capsize=8,
    width=0.7,
  )

  ax.set_xticks(x_positions)
  ax.set_xticklabels(labels, rotation=30, ha="right", fontsize=tick_fontsize)
  ax.set_ylabel("Cycles (Millions)", fontsize=axis_label_fontsize)
  ax.set_title("Fenwick Tree Runtime", fontsize=title_fontsize)
  ax.tick_params(axis="y", labelsize=tick_fontsize)
  ax.yaxis.set_major_formatter(
    FuncFormatter(lambda value, _pos: "0" if np.isclose(value, 0) else (f"{value:.0f}" if value >= 10 else f"{value:.1f}"))
  )
  ax.grid(axis="y", linestyle="--", linewidth=0.5, alpha=0.5)

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

def normalized(profile, benchmark, treatment):
  baseline = get_baseline_cycles(profile, benchmark)
  treatment_cycles = get_cycles(profile, benchmark, treatment)
  return mean(treatment_cycles) / mean(baseline)

# make a bar chart given a profile.json
def make_normalized_chart(profile, output_file, treatments, y_max, width, height, xanchor, yanchor, benchmarks_to_include=None, legend=True, ilp_label="Gurobi"):
  # for each benchmark
  grouped_by_benchmark = group_by_benchmark(profile)
  # Sort by the first treatment's normalized time; push benchmarks whose sort
  # treatment produced no cycles to the end so the key never averages an empty list.
  def _sort_key(group):
    b = group[0].get('benchmark')
    if not has_run_cycles(profile, b, treatments[0]):
      return float('inf')
    return normalized(profile, b, treatments[0])
  sorted_by_eggcc = sorted(grouped_by_benchmark, key=_sort_key)
  benchmarks = [group[0].get('benchmark') for group in sorted_by_eggcc]
  if benchmarks_to_include is not None:
    # keep sorting but filter to only benchmarks in benchmarks_to_include
    benchmarks = [b for b in benchmarks if b in benchmarks_to_include]


  spacing = 0.2
  current_pos = 0

  fig, ax = plt.subplots()
  fig.set_size_inches(width, height)
  
  has_ilp_infeasible = False

  for benchmark in benchmarks:
    miny = 100000
    maxy = 0
    min_color = None

    for runmode in treatments:
      if has_run_cycles(profile, benchmark, runmode) and (not is_ilp_timeout(profile, benchmark, runmode)) and (not is_ilp_infeasible(profile, benchmark, runmode)):
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
        ax.text(
          current_pos,
          y_max,
          'x',
          ha='center',
          va='center',
          zorder=3,
          color="red",
          fontsize=14,
          fontweight='bold',
        )
        i += 1
        continue

      if is_ilp_infeasible(profile, benchmark, runmode):
        # for infeasibles, add x marks to the top
        has_ilp_infeasible = True
        ax.text(
          current_pos,
          y_max,
          'x',
          ha='center',
          va='center',
          zorder=3,
          color="orange",
          fontsize=14,
          fontweight='bold',
        )
        i += 1
        continue

      if not has_run_cycles(profile, benchmark, runmode):
        # Treatment produced no binary (e.g. ILP extraction hit the wall-clock
        # timeout); mark it so the missing bar is visible.
        ax.text(
          current_pos,
          y_max,
          'x',
          ha='center',
          va='center',
          zorder=3,
          color=COLOR_MAP[runmode],
          fontsize=14,
          fontweight='bold',
        )
        i += 1
        continue

      yval = normalized(profile, benchmark, runmode)

      # for outliers, add x marks to the top
      if yval > y_max:
        ax.text(
          current_pos,
          y_max,
          'x',
          ha='center',
          va='center',
          zorder=3,
          color=COLOR_MAP[runmode],
          fontsize=14,
          fontweight='bold',
        )
      else:
        ax.scatter(current_pos, yval, color=COLOR_MAP[runmode], s=CIRCLE_SIZE, zorder=3, marker=SHAPE_MAP[runmode])

    current_pos += spacing * 3

  ax.set_ylabel('Time Relative To LLVM-O3-O0')
  # ax.set_title('Normalized Cycles by Benchmark and Run Mode')
  # add a bar for each runmode, benchmark pair
  # ax.set_xticks(label_x + bar_w, benchmarks, rotation=45, ha='right')
  # turn off x labels
  ax.set_xticks([])
  ax.set_xticklabels([])
  if legend:
    ax.set_xlabel(
      f"Benchmarks sorted by {to_paper_names_treatment(treatments[0])}",
      fontsize=18,
    )

    
  
  # add the legend
  legend_handles = []
  legend_labels = []
  for rm in treatments:
    legend_handles.append(
      plt.Line2D(
        [0],
        [0],
        marker=SHAPE_MAP[rm],
        color='w',
        markerfacecolor=COLOR_MAP[rm],
        markersize=10,
        alpha=0.7,
      )
    )
    legend_labels.append(to_paper_names_treatment(rm))

  legend_handles.append(plt.Line2D([0], [0], color='gray', linestyle='--', linewidth=1.0))
  legend_labels.append('LLVM-O3-O0')

  legend_handles.append(
    plt.Line2D([0], [0], marker='x', color='red', linestyle='None', markersize=10, markeredgewidth=3.0)
  )
  legend_labels.append(f'{ilp_label} Timeout ({format_timeout_label()})')

  if has_ilp_infeasible:
    legend_handles.append(
      plt.Line2D([0], [0], marker='x', color='orange', linestyle='None', markersize=10, markeredgewidth=3.0)
    )
    legend_labels.append(f'{ilp_label} Infeasible')

  anchor_point = (xanchor, yanchor) if xanchor is not None and yanchor is not None else (0.02, 0.98)
  if legend:
    ax.legend(
      legend_handles,
      legend_labels,
      title='Treatment',
      loc='upper left',
      bbox_to_anchor=anchor_point,
      borderaxespad=0.3,
      fontsize=12,
    )

  ax.set_ylim(0.25, y_max)

  ax.set_xlim(-spacing, current_pos - spacing * 2)

  # add a dotted line at 1.0
  ax.axhline(y=1.0, color='gray', linestyle='--', linewidth=1.0)

  plt.tight_layout()
  plt.savefig(output_file)

# TODO change back after anonymization is lifted
def to_paper_names_treatment(treatment):
  if treatment == 'llvm-O0-O0':
    return 'LLVM-O0'
  if treatment == 'llvm-O3-O0':
    return 'LLVM-O3-O0'
  if treatment == 'eggcc-O0-O0':
    # eggcc-O0-O0 is the default (Statewalk DP) extraction; keep the DP paper label.
    return f'EQCC-{TIGER_INLINE_NAME}-O0'
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
  if treatment == 'eggcc-WITHCTX-O0-O0':
    return 'EQCC-WITHCTX-O0-O0'
  if treatment == 'eggcc-tiger-WITHCTX-O0-O0':
    # not talking about context in the paper
    return f'EQCC-{TIGER_INLINE_NAME}-O0'
  if treatment == 'eggcc-tiger-nohacker-WITHCTX-O0-O0':
    # not talking about context in the paper
    return f'EQCC-{TIGER_INLINE_NAME}-NOHACKER-O0'
  if treatment == 'eggcc-tiger-WL-O0-O0':
    return f'EQCC-{TIGER_INLINE_NAME}-WL-O0'
  if treatment == 'eggcc-tiger-ILP-O0-O0':
    return f'EQCC-GUROBI-O0'
  if treatment == 'eggcc-tiger-ILP-CBC-O0-O0':
    return f'EQCC-{TIGER_INLINE_NAME}-ILP-CBC-O0'
  if treatment == 'eggcc-tiger-ILP-WITHCTX-O0-O0':
    return f'EQCC-{TIGER_INLINE_NAME}-ILP-WITHCTX-O0'
  if treatment == 'eggcc-tiger-ILP-NOMIN-O0-O0':
    return f'EQCC-{TIGER_INLINE_NAME}-ILP-NOMIN-O0'
  if treatment == 'eggcc-tiger-ILP-COMPARISON':
    return f'EQCC-{TIGER_INLINE_NAME}-ILP-Comparison'
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



def make_graphs(output_folder, graphs_folder, profile_file, benchmark_suite_folder, config: NightlyConfig, render_extra_figures=True):
  # render_extra_figures=False (set for artifact/--local runs) renders only the headline
  # figures reproduce.sh copies out: the extraction-time CDF, the normalized perf charts, and
  # the Fenwick chart. Everything else -- statewalk-width, heatmap, ILP-encoding, jitter,
  # region-extract, and the LaTeX macros -- are paper/nightly extras the artifact does not
  # ship, so skipping them shortens the reviewer's run and removes their failure modes from it.

  # Graphs report the per-region ILP timeout the data was generated with (e.g. "30 s"
  # for the default nightly, "5 min" for paper).
  set_ilp_timeout_seconds(getattr(config, "ilp_timeout_seconds", 5 * 60))

  # Read profile.json from nightly/output/data/profile.json
  data = []
  with open(profile_file) as f:
      data = json.load(f)
    
  # folders in 
  benchmark_suites = [f for f in os.listdir(benchmark_suite_folder) if os.path.isdir(os.path.join(benchmark_suite_folder, f))]
  benchmark_suites = [os.path.join(benchmark_suite_folder, f) for f in benchmark_suites]

  # Graph gating is by available data, not a single Gurobi flag:
  #   has_comparison -> the COMPARISON treatment ran (tiger + CBC timing available)
  #   has_gurobi     -> the COMPARISON samples include a real Gurobi run
  has_comparison = comparison_ran(data)
  has_gurobi = has_gurobi_ilp_data(data)
  if has_comparison and not has_gurobi:
    print("INFO: No Gurobi timing data; rendering ILP graphs from CBC/tiger data only.")

  # Always available (no ILP timing data required)
  if render_extra_figures:
    make_jitter(data, 4, f'{graphs_folder}/jitter-plot-max-4.png')
  make_fenwick_cycles_bar_chart(data, f'{graphs_folder}/fenwick-cycles-bar-chart.pdf')

  if has_comparison:
    # CDF plots tiger + CBC series (plus the Gurobi series when it ran)
    make_extraction_time_cdf(data, f'{graphs_folder}/extraction-time-cdf.pdf', use_log_x=True, use_exp_y=False, include_gurobi=has_gurobi)
    if render_extra_figures:
      # tiger greedy extraction time (no Gurobi needed)
      make_region_extract_plot(data, f'{graphs_folder}/egraph-size-vs-tiger-time.pdf', plot_ilp=False)
      if has_gurobi:
        make_region_extract_plot(data, f'{graphs_folder}/egraph-size-vs-ILP-time.pdf', plot_ilp=True)
        make_extraction_time_histogram(data, f'{graphs_folder}/extraction-time-histogram.pdf')
      else:
        print("Skipping ILP-time region plot and extraction-time-histogram (require Gurobi)")
  else:
    print("Skipping all region-timing graphs (eggcc-tiger-ILP-COMPARISON produced no data)")

  statewalk_histogram_max_width = None

  tiger_optimizations_on = StatewalkTreatment(
    runtime="tiger",
    liveness_on=True,
    satellite_on=True,
    label="Optimizations On",
  )
  tiger_optimizations_off = StatewalkTreatment(
    runtime="tiger",
    liveness_on=False,
    satellite_on=False,
    label="Optimizations Off",
  )
  ilp_gurobi = StatewalkTreatment(runtime="ilp_gurobi", liveness_on=False, satellite_on=False)
  ilp_cbc = StatewalkTreatment(runtime="ilp_cbc", liveness_on=False, satellite_on=False)

  # Statewalk-width and ILP-encoding graphs. Tiger/CBC graphs render whenever the
  # COMPARISON treatment ran; the Gurobi-specific graphs need has_gurobi. All of these are
  # paper/nightly extras, so the artifact's headline-only figure set skips the whole block.
  if has_comparison and render_extra_figures:
    make_statewalk_width_histogram(
      data,
      f'{graphs_folder}/statewalk-width-histogram-with-liveness.pdf',
      tiger_optimizations_on,
      is_average=False,
      max_width=statewalk_histogram_max_width,
    )
    make_statewalk_width_histogram(
      data,
      f'{graphs_folder}/statewalk-width-histogram.pdf',
      tiger_optimizations_off,
      is_average=False,
      max_width=statewalk_histogram_max_width,
    )
    print_top_statewalk_width_samples(
      data,
      tiger_optimizations_off,
      is_average=False,
      max_width=statewalk_histogram_max_width,
    )

    make_statewalk_width_performance_scatter_multi(
      data,
      f'{graphs_folder}/statewalk-width-vs-tiger-time.pdf',
      [tiger_optimizations_off, tiger_optimizations_on],
      is_average=False,
      scale_by_egraph_size=False,
      y_break=(0.6, 4.5),
      y_break_runtimes={'tiger'},
    )
    # CBC is always present; add the Gurobi series only when it ran.
    ilp_scatter_treatments = [ilp_cbc] + ([ilp_gurobi] if has_gurobi else [])
    make_statewalk_width_performance_scatter_multi(
      data,
      f'{graphs_folder}/statewalk-width-vs-ilp-time.pdf',
      ilp_scatter_treatments,
      is_average=False,
      scale_by_egraph_size=False,
    )

    make_egraph_size_vs_statewalk_width_heatmap(
      data,
      f'{graphs_folder}/heatmap-tiger-time-with-egraph-size-vs-statewalk-width-no-raytrace.pdf',
      tiger_optimizations_off,
      is_average=False,
      min_width=1,
    )
    make_egraph_size_vs_statewalk_width_heatmap(
      data,
      f'{graphs_folder}/heatmap-tiger-time-with-egraph-size-vs-statewalk-width-no-raytrace-max6000.pdf',
      tiger_optimizations_off,
      is_average=False,
      min_width=1,
      max_width=6000,
    )
    # ILP encoding var count is solver-independent
    make_ilp_encoding_scatter(
      data,
      f'{graphs_folder}/ilp-encoding-size-scatter.pdf',
    )
    make_cbc_encoding_time_scatter(
      data,
      f'{graphs_folder}/ilp-encoding-size-vs-cbc-solve-time.pdf',
    )

    if has_gurobi:
      make_egraph_size_vs_statewalk_width_heatmap(
        data,
        f'{graphs_folder}/heatmap-ilp-time-with-egraph-size-vs-statewalk-width-no-raytrace.pdf',
        ilp_gurobi,
        is_average=False,
        min_width=1,
      )
      make_egraph_size_vs_statewalk_width_heatmap(
        data,
        f'{graphs_folder}/heatmap-ilp-time-with-egraph-size-vs-statewalk-width-no-raytrace-max6000.pdf',
        ilp_gurobi,
        is_average=False,
        min_width=1,
        max_width=6000,
      )
      make_ilp_encoding_time_scatter(
        data,
        f'{graphs_folder}/ilp-encoding-size-vs-solve-time.pdf',
      )
      make_peggy_comparison_graph(
        data,
        "./infra/peggy_data.csv",
        f'{graphs_folder}/eggcc-extraction-time-ratio.pdf',
        f'{graphs_folder}/peggy-extraction-time-ratio.pdf'
      )
    else:
      print("Skipping Gurobi-only ILP graphs (ilp-time heatmaps, Gurobi solve-time scatter, peggy comparison)")
  elif not render_extra_figures:
    print("Skipping statewalk and ILP-related graphs (artifact figure set: only the headline figures are rendered)")
  else:
    print("Skipping statewalk and ILP-related graphs (eggcc-tiger-ILP-COMPARISON produced no data)")
  
  # Normalized charts: use the Gurobi ILP treatment when it ran, otherwise the CBC ILP
  # treatment (both run per-benchmark), so the perf charts render with or without Gurobi.
  ilp_chart_treatment = "eggcc-tiger-ILP-O0-O0" if has_gurobi else "eggcc-tiger-ILP-CBC-O0-O0"
  ilp_chart_label = "Gurobi" if has_gurobi else "CBC"
  if benchmark_suites:
    for suite_path in benchmark_suites:
      suite = os.path.basename(suite_path)
      suite_benchmarks_all = benchmarks_in_folder(suite_path)
      profile_for_suite = [b for b in data if b['benchmark'] in suite_benchmarks_all]
      # Filter suite_benchmarks to only include benchmarks that have profile data
      profiled_benchmarks = set(b['benchmark'] for b in profile_for_suite)
      suite_benchmarks = [b for b in suite_benchmarks_all if b in profiled_benchmarks]

      width = 10
      height = 4
      y_max = 3.5
      xanchor = 0.02
      yanchor = 0.98

      if suite == "polybench":
        y_max = 10.0
        width = 6
        height = 5.0

      chart_treatments = ["eggcc-O0-O0", ilp_chart_treatment, "llvm-O0-O0"]

      if suite == "bril":
        benchmarks_under3 = [b for b in suite_benchmarks if normalized(data, b, "eggcc-O0-O0") <= 3.0]
        benchmarks_over3 = [b for b in suite_benchmarks if normalized(data, b, "eggcc-O0-O0") > 3.0]

        make_normalized_chart(
          profile_for_suite,
          f'{graphs_folder}/normalized-binary-perf-chart-under3-{suite}.pdf',
          chart_treatments,
          y_max,
          width,
          height + 0.5,
          xanchor,
          yanchor,
          benchmarks_under3,
          legend=True,
          ilp_label=ilp_chart_label,
        )
        make_normalized_chart(
          profile_for_suite,
          f'{graphs_folder}/normalized-binary-perf-chart-over3-{suite}.pdf',
          chart_treatments,
          20.0,
          2,
          height,
          xanchor,
          yanchor,
          benchmarks_over3,
          legend=False,
          ilp_label=ilp_chart_label,
        )

      else:
        make_normalized_chart(
          profile_for_suite,
          f'{graphs_folder}/normalized-binary-perf-chart-{suite}.pdf',
          chart_treatments,
          y_max,
          width,
          height,
          xanchor,
          yanchor,
          None,
          legend=True,
          ilp_label=ilp_chart_label,
        )

  # nightlymacros.tex is pervasively Gurobi-dependent (Gurobi speedups, timeout counts,
  # ilp_extract_time). The frontend already treats its absence as expected without Gurobi.
  # It is a paper artifact, not a reproduce.sh figure, so the artifact figure set skips it.
  if has_gurobi and render_extra_figures:
    make_macros(data, benchmark_suites, f'{graphs_folder}/nightlymacros.tex')
  elif has_gurobi:
    print("Skipping macro generation (artifact figure set: only the headline figures are rendered)")
  else:
    print("Skipping macro generation (nightlymacros.tex requires Gurobi data)")

  # make json list of graph names and put in in output
  graph_names = []
  # read all files in the graphs folder
  for root, _, filenames in os.walk(graphs_folder):
    for filename in filenames:
      graph_names.append(filename)
  with open(f'{output_folder}/graphs.json', 'w') as f:
    json.dump(graph_names, f)
