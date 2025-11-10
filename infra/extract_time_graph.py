from collections import Counter
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from matplotlib.patches import Patch
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset
import numpy as np

from graph_helpers import *


def make_extraction_time_cdf(data, output, use_log_x):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  extract_times = []
  ilp_times = []
  ilp_timeout_count = 0

  for sample in points:
    extract_time = sample["extract_time"]
    if extract_time is not None:
      extract_value = extract_time["secs"] + extract_time["nanos"] / 1e9
      extract_times.append(extract_value)

    ilp_infeasible = sample.get("ilp_infeasible", False)
    if ilp_infeasible:
      continue

    if sample["ilp_timed_out"]:
      ilp_timeout_count += 1
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

    if use_log_x:
      if np.any(raw_values <= 0):
        invalid = int(np.count_nonzero(raw_values <= 0))
        raise ValueError(
          f"Encountered {invalid} non-positive {label} values while building extraction time CDF"
        )
    else:
      raw_values = raw_values[~np.isnan(raw_values)]
      if raw_values.size == 0:
        return

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

  if ilp_timeout_count:
    current_count = len(ilp_times)
    final_count = current_count + ilp_timeout_count
    tail_end_time = float(ILP_TIMEOUT_SECONDS)
    tail_start_time = float(np.max(ilp_times)) if ilp_times else tail_end_time
    if tail_start_time > tail_end_time:
      tail_start_time = tail_end_time

    if use_log_x:
      if tail_end_time <= 0:
        tail_end_time = 1e-3
      tail_plot_end = tail_end_time * 1.05 if tail_end_time > 0 else 1e-3
    else:
      delta = max(0.05 * tail_end_time, 1.0)
      tail_plot_end = tail_end_time + delta

    tail_times = np.array([tail_start_time, tail_end_time, tail_plot_end], dtype=float)
    tail_counts = np.array([current_count, final_count, final_count], dtype=float)
    ax.step(tail_times, tail_counts, where='post', color='red', linewidth=2, label='ILP Timeouts')

    plotted_any = True

    positive_tail_times = [t for t in tail_times if t > 0]
    if positive_tail_times:
      tail_min_time = min(positive_tail_times)
      if min_time is None:
        min_time = tail_min_time
      else:
        min_time = min(min_time, tail_min_time)
    max_time = max(max_time, float(np.max(tail_times)))
    max_count = max(max_count, final_count)

  if not plotted_any:
    print("WARNING: No data plotted in make_extraction_time_cdf")
    plt.close()
    return

  ax.set_xlabel('Time (Seconds)')
  ax.set_ylabel('Number of Benchmarks')
  ax.set_title('CDF of Extraction Times')

  if max_time <= 0:
    max_time = 1.0

  if use_log_x:
    if min_time is None or min_time <= 0:
      min_time = max_time / 100.0
      if min_time <= 0:
        min_time = 1e-3

    ax.set_xscale('log')
    ax.set_xlim(left=min_time * 0.9, right=max_time * 1.1)
  else:
    if min_time is None:
      min_time = 0.0
    span = max(max_time - min_time, max_time if max_time > 0 else 1.0)
    left = min(0.0, min_time - 0.05 * span)
    right = max_time + 0.05 * span
    if right <= left:
      right = left + span if span > 0 else left + 1.0
    ax.set_xscale('linear')
    ax.set_xlim(left=left, right=right)

  if max_count == 0:
    max_count = 1
  ax.set_ylim(0.0, max_count * 1.05)

  ax.grid(axis='both', linestyle='--', alpha=0.4)

  if use_log_x:
    ax.xaxis.set_major_locator(mticker.LogLocator(base=10, numticks=10))
    ax.xaxis.set_minor_locator(mticker.LogLocator(base=10, subs=np.arange(2, 10), numticks=10))

    def _format_time_tick(value, _pos):
      if value <= 0:
        return ''
      text = f"{value:.8f}".rstrip('0').rstrip('.')
      if text == '':
        text = '0'
      if '.' not in text and len(text) > 3:
        text = f"{int(round(value)):,}"
      return text

    ax.xaxis.set_major_formatter(mticker.FuncFormatter(_format_time_tick))
    ax.xaxis.set_minor_formatter(mticker.FuncFormatter(lambda value, _pos: ''))
  else:
    ax.xaxis.set_major_locator(mticker.MaxNLocator(nbins=12, prune=None, min_n_ticks=6))
    ax.xaxis.set_minor_locator(mticker.AutoMinorLocator())
    ax.xaxis.set_major_formatter(mticker.FuncFormatter(lambda value, _pos: f"{value:g}"))

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



