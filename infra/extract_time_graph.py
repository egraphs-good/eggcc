from collections import Counter
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from matplotlib.patches import Patch
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset
import numpy as np

from graph_helpers import *


def make_extraction_time_cdf(data, output, use_log_x, use_exp_y=False):
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

  if not extract_times and not ilp_times and ilp_timeout_count == 0:
    print("WARNING: No extraction timing data found; skipping CDF plot")
    return

  plt.figure(figsize=(10, 6))

  ax = plt.gca()
  plotted_any = False
  max_time = 0.0
  min_time = None
  max_percent = 100.0

  def _plot_cdf(times, total_count, label, color):
    nonlocal plotted_any, max_time, min_time, max_percent
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
    total = float(total_count if total_count is not None else len(sorted_times))
    if total <= 0:
      return
    ranks = np.arange(1, len(sorted_times) + 1, dtype=float)
    percents = ranks / total * 100.0
    plotted_values = percents
    plotted_any = True
    ax.step(sorted_times, plotted_values, where='post', label=label, color=color, linewidth=2)
    latest_time = float(sorted_times[-1])
    earliest_time = float(sorted_times[0])
    max_time = max(max_time, latest_time)
    min_time = earliest_time if min_time is None else min(min_time, earliest_time)

  extract_total = len(extract_times)
  ilp_total = len(ilp_times) + ilp_timeout_count

  _plot_cdf(extract_times, extract_total, f'{EGGCC_NAME} Extraction Time', 'blue')
  _plot_cdf(ilp_times, ilp_total, 'ILP Solve Time', 'green')

  if ilp_timeout_count:
    current_count = len(ilp_times)
    final_count = current_count + ilp_timeout_count
    if final_count <= 0:
      final_count = ilp_timeout_count
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
    total = float(final_count)
    if total <= 0:
      total = float(ilp_timeout_count)
    current_percent = (current_count / total) * 100.0 if total else 0.0
    final_percent = 100.0
    tail_percents = np.array([current_percent, final_percent, final_percent], dtype=float)
    ax.step(tail_times, tail_percents, where='post', color='red', linewidth=2, label='ILP Timeouts')

    plotted_any = True

    positive_tail_times = [t for t in tail_times if t > 0]
    if positive_tail_times:
      tail_min_time = min(positive_tail_times)
      if min_time is None:
        min_time = tail_min_time
      else:
        min_time = min(min_time, tail_min_time)
    max_time = max(max_time, float(np.max(tail_times)))
    max_percent = max(max_percent, final_percent)

  if not plotted_any:
    print("WARNING: No data plotted in make_extraction_time_cdf")
    plt.close()
    return

  ax.set_xlabel('Time (Seconds)')
  ax.set_ylabel('Percent of Benchmarks')
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

  if use_exp_y:
    def _exp_forward(values):
      arr = np.asarray(values, dtype=float)
      clipped = np.clip(arr, 0.0, max_percent)
      transformed = np.log1p(clipped)
      return transformed

    def _exp_inverse(values):
      arr = np.asarray(values, dtype=float)
      clipped = np.clip(arr, 0.0, max_percent)
      norm = np.expm1(clipped)
      return np.clip(norm, 0.0, max_percent)

    ax.set_yscale('function', functions=( _exp_inverse, _exp_forward))
    ax.set_ylim(0.0, max_percent)
    tick_candidates = [0.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0, 99.5, 99.9, 100.0]
    ticks = sorted({t for t in tick_candidates if t <= max_percent + 1e-6})
    if max_percent not in ticks:
      ticks.append(max_percent)
    ax.yaxis.set_major_locator(mticker.FixedLocator(ticks))
    ax.yaxis.set_minor_locator(mticker.NullLocator())
  else:
    lower_bound = 0.0
    upper_bound = min(max_percent * 1.05, 105.0)
    if upper_bound <= lower_bound:
      upper_bound = lower_bound + 1.0
    ax.set_yscale('linear')
    ax.set_ylim(lower_bound, upper_bound)

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

  if not use_exp_y:
    ax.yaxis.set_major_locator(mticker.MaxNLocator(nbins=10, prune=None))
    ax.yaxis.set_minor_locator(mticker.AutoMinorLocator())

  def _format_percent_tick(value, _pos):
    if value < 0:
      return ''
    if value >= 99.995:
      return '100%'
    if abs(value - round(value)) < 1e-6:
      return f"{int(round(value))}%"
    if value >= 10.0:
      return f"{value:.1f}%"
    return f"{value:.2f}%"

  ax.yaxis.set_major_formatter(mticker.FuncFormatter(_format_percent_tick))

  ax.legend(loc='lower right')

  plt.tight_layout()
  plt.savefig(output)
