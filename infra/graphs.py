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

EGGCC_NAME = "eggcc"

GRAPH_RUN_MODES = ["llvm-O0-O0", "eggcc-O0-O0", "llvm-O3-O0"]

if profile.TO_ABLATE != "":
  GRAPH_RUN_MODES.extend(["eggcc-ablation-O0-O0", "eggcc-ablation-O3-O0", "eggcc-ablation-O3-O3"])

# need ilp and graph run modes for this script to work
NECESSARY_MODES = GRAPH_RUN_MODES + ["eggcc-ILP-O0-O0"]

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
  "eggcc-ILP-O0-O0": "red",
  "eggcc-tiger-O0-O0": "cyan",
  "eggcc-tiger-WL-O0-O0": "magenta",
  "eggcc-tiger-ILP-O0-O0": "green",
  "eggcc-tiger-ILP-CBC-O0-O0": "olive",
  "eggcc-tiger-ILP-NOMIN-O0-O0": "darkgreen",
  "eggcc-tiger-ILP-WITHCTX-O0-O0": "orange",
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
  "eggcc-tiger-O0-O0": "o",
  'eggcc-tiger-ILP-O0-O0': "o",
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

def is_ilp_timeout(data, benchmark_name, run_method):
  if run_method != "eggcc-tiger-ILP-O0-O0":
    return False
  for row in data:
    if row['benchmark'] == benchmark_name and row['runMethod'] == run_method:
      return row['ILPTimeOut']

  raise KeyError(f"Missing benchmark {benchmark_name} with runMethod {run_method}")


def is_ilp_infeasible(data, benchmark_name, run_method):
  if run_method != "eggcc-tiger-ILP-O0-O0":
    return False
  for row in data:
    if row['benchmark'] == benchmark_name and row['runMethod'] == run_method:
      return row["failed"] and ("ILP solver reported infeasibility" in row["error"])
  raise KeyError(f"Missing benchmark {benchmark_name} with runMethod {run_method}")

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
    extract_time = sample["extract_time"]
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
    extract_time = sample["extract_time"]
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


def make_extraction_time_cdf(data, output):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  extract_times = []
  ilp_times = []

  for sample in points:
    extract_time = sample["extract_time"]
    if extract_time is not None:
      extract_value = extract_time["secs"] + extract_time["nanos"] / 1e9
      extract_times.append(extract_value)

    ilp_infeasible = sample.get("ilp_infeasible", False)
    if ilp_infeasible or sample["ilp_timed_out"]:
      continue

    ilp_time = sample["ilp_extract_time"]
    if ilp_time is None:
      continue

    ilp_value = ilp_time["secs"] + ilp_time["nanos"] / 1e9
    ilp_times.append(ilp_value)

  if not extract_times and not ilp_times:
    print("WARNING: No extraction timing data found; skipping CDF plot")
    return

  plt.figure(figsize=(10, 6))

  ax = plt.gca()
  plotted_any = False
  max_time = 0.0
  min_time = None
  max_count = 0

  def _plot_cdf(times, label, color):
    nonlocal plotted_any, max_time, min_time, max_count
    if not times:
      return
    raw_values = np.array(times, dtype=float)
    if np.any(raw_values <= 0):
      invalid = int(np.count_nonzero(raw_values <= 0))
      raise ValueError(
        f"Encountered {invalid} non-positive {label} values while building extraction time CDF"
      )

    sorted_times = np.sort(raw_values)
    counts = np.arange(1, len(sorted_times) + 1, dtype=float)
    ax.step(sorted_times, counts, where='post', label=label, color=color, linewidth=2)
    plotted_any = True
    latest_time = float(sorted_times[-1])
    earliest_time = float(sorted_times[0])
    max_time = max(max_time, latest_time)
    min_time = earliest_time if min_time is None else min(min_time, earliest_time)
    max_count = max(max_count, int(counts[-1]))

  _plot_cdf(extract_times, f'{EGGCC_NAME} Extraction Time', 'blue')
  _plot_cdf(ilp_times, 'ILP Solve Time', 'green')

  if not plotted_any:
    print("WARNING: No data plotted in make_extraction_time_cdf")
    plt.close()
    return

  ax.set_xlabel('Time (Seconds)')
  ax.set_ylabel('Number of Benchmarks')
  ax.set_title('CDF of Extraction Times')

  if max_time <= 0:
    max_time = 1.0
  if min_time is None or min_time <= 0:
    min_time = max_time / 100.0
    if min_time <= 0:
      min_time = 1e-3

  ax.set_xscale('log')
  ax.set_xlim(left=min_time * 0.9, right=max_time * 1.1)

  if max_count == 0:
    max_count = 1
  ax.set_ylim(0.0, max_count * 1.05)

  ax.grid(axis='both', linestyle='--', alpha=0.4)

  ax.xaxis.set_major_locator(mticker.LogLocator(base=10, numticks=10))
  ax.xaxis.set_minor_locator(mticker.LogLocator(base=10, subs=np.arange(2, 10), numticks=10))

  def _format_time_tick(value, _pos):
    if value <= 0:
      return ''
    text = f"{value:.8f}".rstrip('0').rstrip('.')
    if text == '':
      text = '0'
    if '.' not in text and len(text) > 3:
      # add thousands separators for whole numbers
      text = f"{int(round(value)):,}"
    return text

  ax.xaxis.set_major_formatter(mticker.FuncFormatter(_format_time_tick))
  ax.xaxis.set_minor_formatter(mticker.FuncFormatter(lambda value, _pos: ''))
  ax.yaxis.set_major_locator(mticker.MaxNLocator(integer=True, prune=None))
  ax.yaxis.set_minor_locator(mticker.AutoMinorLocator())

  def _format_count_tick(value, _pos):
    if value < 0:
      return ''
    if abs(value - round(value)) < 1e-6:
      return f"{int(round(value)):,}"
    return f"{value:.2f}"

  ax.yaxis.set_major_formatter(mticker.FuncFormatter(_format_count_tick))

  ax.legend(loc='lower right')

  plt.tight_layout()
  plt.savefig(output)


def make_statewalk_width_histogram(data, output, is_liveon, is_average, max_width=None):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  widths = []
  missing_widths = 0
  filtered_above_threshold = 0
  max_width_label = None
  if max_width is not None:
    max_width_label = f"{int(max_width):,}" if float(max_width).is_integer() else f"{max_width:g}"

  for sample in points:
    width_name = f"statewalk_width_{"liveon" if is_liveon else "liveoff"}_{"avg" if is_average else "max"}"
    width = sample[width_name]
    if width is None:
      missing_widths += 1
      continue
    if max_width is not None and width > max_width:
      filtered_above_threshold += 1
      continue
    widths.append(width)

  if missing_widths:
    print(f"WARNING: Skipping {missing_widths} timing samples with missing statewalk_width")
  if filtered_above_threshold and max_width_label is not None:
    print(
      f"WARNING: Skipping {filtered_above_threshold} samples with statewalk_width > {max_width_label}"
    )

  if not widths:
    print("WARNING: No statewalk width data found; skipping histogram")
    return

  counts = Counter(widths)
  sorted_widths = sorted(counts.keys())
  frequencies = [counts[w] for w in sorted_widths]

  plt.figure(figsize=(10, 6))
  plt.bar(sorted_widths, frequencies, color='skyblue', edgecolor='black')
  plt.xlabel(f'Statewalk Width{" Average" if is_average else ""}')
  plt.ylabel('Number of Regionalized E-Graphs')
  title = f'Distribution of Statewalk Width{" With Liveness Analysis" if is_liveon else ""}'
  if max_width_label is not None:
    title += f' (≤ {max_width_label})'
  plt.title(title)
  # log scale y axis
  plt.yscale('log')

  def _format_tick(value, _pos):
    if value <= 0:
      return ''
    if value < 1:
      return f'{value:.2f}'.rstrip('0').rstrip('.')
    if value < 10:
      return f'{value:.1f}'.rstrip('0').rstrip('.')
    return f'{value:g}'

  ax = plt.gca()
  ax.yaxis.set_major_formatter(mticker.FuncFormatter(_format_tick))

  plt.grid(axis='y', linestyle='--', alpha=0.5)
  plt.tight_layout()
  plt.savefig(output)


def print_top_statewalk_width_samples(
  data,
  treatment,
  is_liveon,
  is_average,
  limit=10,
  max_width=None,
):
  benchmarks = dedup([b.get('benchmark') for b in data])
  width_key = f"statewalk_width_{'liveon' if is_liveon else 'liveoff'}_{'avg' if is_average else 'max'}"

  max_width_label = None
  if max_width is not None:
    max_width_label = f"{int(max_width):,}" if float(max_width).is_integer() else f"{max_width:g}"

  samples = []

  for benchmark in benchmarks:
    timings = get_extract_region_timings(treatment, data, benchmark)
    if timings is False:
      continue
    for sample in timings:
      width = sample[width_key]
      if width is None:
        continue
      if max_width is not None and width > max_width:
        continue
      samples.append((width, benchmark, treatment))

  if not samples:
    print(f"WARNING: No statewalk width data found for {treatment} ({width_key})")
    return

  samples.sort(key=lambda entry: entry[0], reverse=True)
  if limit is None or limit <= 0:
    limit = 10
  limit = min(limit, len(samples))

  descriptor_parts = [
    'live-on' if is_liveon else 'live-off',
    'average' if is_average else 'maximum',
  ]
  descriptor = ' '.join(descriptor_parts)
  filter_suffix = f" (≤ {max_width_label})" if max_width_label is not None else ''

  print(f"Top {limit} statewalk widths ({descriptor}) for treatment {treatment}{filter_suffix}:")

  def _format_width(value):
    if isinstance(value, (int, np.integer)):
      return f"{int(value):,}"
    if isinstance(value, float):
      if value.is_integer():
        return f"{int(value):,}"
      return f"{value:.4g}"
    return str(value)

  for idx, (width, benchmark, sample_treatment) in enumerate(samples[:limit], start=1):
    width_display = _format_width(width)
    print(f"  {idx}. {benchmark} ({sample_treatment}) – {width_display}")


def make_statewalk_width_performance_scatter(data, output, plot_ilp, is_liveon, is_average, scale_by_egraph_size):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  width_key = f"statewalk_width_{'liveon' if is_liveon else 'liveoff'}_{'avg' if is_average else 'max'}"

  x_values = []
  y_values = []
  timeout_x = []
  timeout_y = []
  infeasible_x = []
  infeasible_y = []
  missing_widths = 0
  non_positive_widths = 0
  missing_egraph_sizes = 0
  non_positive_products = 0
  missing_timings = 0

  ILP_TIMEOUT_SECONDS = 5 * 60

  for sample in points:
    width = sample[width_key]
    if width is None:
      missing_widths += 1
      continue
    if width <= 0:
      non_positive_widths += 1
      continue

    x_magnitude = width
    if scale_by_egraph_size:
      egraph_size = sample["egraph_size"]
      if egraph_size is None:
        missing_egraph_sizes += 1
        continue
      if egraph_size <= 0:
        non_positive_products += 1
        continue
      x_magnitude = width * egraph_size

    if plot_ilp:
      ilp_time = sample["ilp_extract_time"]
      ilp_infeasible = sample.get("ilp_infeasible", False) # TODO replace with indexing so we error if not present
      if ilp_infeasible:
        infeasible_x.append(x_magnitude)
        infeasible_y.append(ILP_TIMEOUT_SECONDS)
      elif sample["ilp_timed_out"]:
        timeout_x.append(x_magnitude)
        timeout_y.append(ILP_TIMEOUT_SECONDS)
      else:
        value = ilp_time["secs"] + ilp_time["nanos"] / 1e9
        x_values.append(x_magnitude)
        y_values.append(value)
    else:
      extract_time = sample["extract_time"]
      if extract_time is None:
        missing_timings += 1
        continue
      value = extract_time["secs"] + extract_time["nanos"] / 1e9
      x_values.append(x_magnitude)
      y_values.append(value)

  if missing_widths:
    print(f"WARNING: Skipping {missing_widths} samples with missing {width_key}")
  if non_positive_widths:
    print(f"WARNING: Skipping {non_positive_widths} samples with non-positive {width_key}")
  if missing_egraph_sizes:
    print(f"WARNING: Skipping {missing_egraph_sizes} samples with missing egraph_size when scaling x-axis")
  if missing_timings and not plot_ilp:
    print(f"WARNING: Skipping {missing_timings} samples with missing extract_time")

  plt.figure(figsize=(10, 6))

  plotted_any = False
  primary_label = 'ILP Solve Time' if plot_ilp else f'{EGGCC_NAME} Extraction Time'
  primary_color = 'green' if plot_ilp else 'blue'

  if x_values:
    plt.scatter(
      x_values,
      y_values,
      color=primary_color,
      label=primary_label,
      alpha=0.7,
      edgecolors='black',
      linewidths=0.5,
      s=60,
    )
    plotted_any = True

  if plot_ilp and timeout_x:
    plt.scatter(
      timeout_x,
      timeout_y,
      color='red',
      marker='x',
      label='ILP Timeout (5 min)',
      linewidths=2.0,
      s=100,
    )
    plotted_any = True

  if plot_ilp and infeasible_x:
    plt.scatter(
      infeasible_x,
      infeasible_y,
      color='orange',
      marker='x',
      label='ILP Infeasible',
      linewidths=2.0,
      s=100,
    )
    plotted_any = True

  if not plotted_any:
    print("WARNING: No data plotted in make_statewalk_width_performance_scatter")
    plt.close()
    return

  if scale_by_egraph_size:
    x_label = "(Statewalk Width × E-graph Size)"
    if is_average:
      x_label = "(Statewalk Width Average × E-graph Size)"
  else:
    x_label = f"Statewalk Width{' Average' if is_average else ''}"

  plt.xlabel(x_label)
  ylabel = 'ILP Solve Time (Seconds)' if plot_ilp else f'{EGGCC_NAME} Extraction Time (Seconds)'
  plt.ylabel(ylabel)

  title = 'Statewalk Width vs '
  title += 'ILP Solve Time' if plot_ilp else f'{EGGCC_NAME} Extraction Time'
  if is_liveon:
    title += ' (With Liveness Analysis)'
  if is_average and not scale_by_egraph_size:
    title += ' - Average Width'
  if scale_by_egraph_size:
    title += ' (Width × Size)'
  plt.title(title)

  plt.grid(alpha=0.3)

  ax = plt.gca()
  ax.set_xscale('log')

  plt.tight_layout()
  plt.savefig(output)


def make_egraph_size_vs_statewalk_width_heatmap(
  data,
  output,
  is_liveon,
  is_average,
  min_width=None,
  max_width=None,
  runtime_source="tiger",
):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  width_key = f"statewalk_width_{'liveon' if is_liveon else 'liveoff'}_{'avg' if is_average else 'max'}"

  sizes = []
  widths = []
  runtimes = []
  missing_runtimes = 0
  skipped_above_max_width = 0
  timeout_sizes = []
  timeout_widths = []
  infeasible_sizes = []
  infeasible_widths = []

  for sample in points:
    width = sample.get(width_key)
    if width is None:
      continue
    if min_width is not None and width <= min_width:
      continue
    if max_width is not None and width > max_width:
      skipped_above_max_width += 1
      continue

    egraph_size = sample.get("egraph_size")

    if runtime_source == "ilp":
      if sample.get("ilp_infeasible", False):
        if egraph_size is not None:
          infeasible_sizes.append(egraph_size)
          infeasible_widths.append(width)
        continue
      if sample.get("ilp_timed_out", False):
        if egraph_size is not None:
          timeout_sizes.append(egraph_size)
          timeout_widths.append(width)
        continue
      ilp_time = sample.get("ilp_extract_time")
      if ilp_time is None:
        missing_runtimes += 1
        continue
      runtime_secs = ilp_time["secs"] + ilp_time["nanos"] / 1e9
    else:
      extract_time = sample.get("extract_time")
      if extract_time is None:
        missing_runtimes += 1
        continue
      runtime_secs = extract_time["secs"] + extract_time["nanos"] / 1e9

    if runtime_secs <= 0:
      continue

    sizes.append(egraph_size)
    widths.append(width)
    runtimes.append(runtime_secs)

  if missing_runtimes:
    label = 'ILP solve time' if runtime_source == "ilp" else 'extract_time'
    print(f"WARNING: Skipping {missing_runtimes} samples with missing {label}")

  if not sizes:
    print("WARNING: No data plotted in make_egraph_size_vs_statewalk_width_heatmap")
    return

  sizes = np.array(sizes)
  widths = np.array(widths)
  runtimes = np.array(runtimes)

  num_bins = 30

  def _edges(values):
    vmin = values.min()
    vmax = values.max()
    if vmin == vmax:
      delta = max(1.0, abs(vmin) * 0.1)
      return np.array([vmin, vmin + delta])
    return np.linspace(vmin, vmax, num_bins + 1)

  size_edges = _edges(sizes)
  width_edges = _edges(widths)

  sum_heat, _, _ = np.histogram2d(sizes, widths, bins=[size_edges, width_edges], weights=runtimes)
  count_heat, _, _ = np.histogram2d(sizes, widths, bins=[size_edges, width_edges])

  with np.errstate(invalid='ignore', divide='ignore'):
    avg_heat = np.divide(sum_heat, count_heat, where=count_heat > 0)
  avg_heat[count_heat == 0] = np.nan

  valid = avg_heat[np.isfinite(avg_heat) & (avg_heat > 0)]
  if valid.size == 0:
    print("WARNING: No valid runtime data for heatmap")
    return

  plt.figure(figsize=(10, 6))
  cmap = plt.get_cmap('inferno').copy()
  cmap.set_bad(color='lightgray')

  mesh = plt.pcolormesh(size_edges, width_edges, avg_heat.T, cmap=cmap, shading='auto')
  cbar = plt.colorbar(mesh)
  if runtime_source == "ilp":
    cbar.set_label('ILP Solve Time (Seconds)')
  else:
    cbar.set_label(f'{EGGCC_NAME} Extraction Time (Seconds)')

  legend_handles = []
  legend_labels = []

  solved_points = plt.scatter(
    sizes,
    widths,
    color='white',
    edgecolors='black',
    linewidths=0.2,
    s=20,
    alpha=0.6,
    zorder=3,
    label='Solved Points' if runtime_source == "ilp" else None,
  )

  if runtime_source == "ilp" and sizes.size:
    legend_handles.append(solved_points)
    legend_labels.append('Solved Points')

  if runtime_source == "ilp" and timeout_sizes:
    timeout_scatter = plt.scatter(
      timeout_sizes,
      timeout_widths,
      marker='x',
      color='red',
      linewidths=1.5,
      s=60,
      label='ILP Timeout',
      zorder=4,
    )
    legend_handles.append(timeout_scatter)
    legend_labels.append('ILP Timeout')

  if runtime_source == "ilp" and infeasible_sizes:
    infeasible_scatter = plt.scatter(
      infeasible_sizes,
      infeasible_widths,
      marker='x',
      color='orange',
      linewidths=1.5,
      s=60,
      label='ILP Infeasible',
      zorder=4,
    )
    legend_handles.append(infeasible_scatter)
    legend_labels.append('ILP Infeasible')

  if legend_handles:
    plt.legend(legend_handles, legend_labels, loc='upper right')

  plt.xlabel('Regionalized E-graph Size')
  y_label = 'Statewalk Width'
  y_label += ' Average' if is_average else ' Maximum'
  y_label += ' (With Liveness)' if is_liveon else ' (No Liveness)'
  plt.ylabel(y_label)

  if runtime_source == "ilp":
    title = 'ILP Runtime Heatmap by E-graph Size and Statewalk Width'
  else:
    title = 'Tiger Runtime Heatmap by E-graph Size and Statewalk Width'
  title += ' (Average Width)' if is_average else ''
  title += ' with Liveness' if is_liveon else ' without Liveness'
  if max_width is not None:
    title += f' (≤ Width {max_width})'
    plt.ylim(bottom=0, top=max_width)
  plt.title(title)

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
  make_extraction_time_cdf(profile, f'{graphs_folder}/extraction_time_cdf.pdf')
  statewalk_histogram_max_width = None
  statewalk_histogram_treatment = "eggcc-tiger-ILP-COMPARISON"
  make_statewalk_width_histogram(
    profile,
    f'{graphs_folder}/statewalk_width_histogram_with_liveness_analysis.pdf',
    True,
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


