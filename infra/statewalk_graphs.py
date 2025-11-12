from collections import Counter
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from matplotlib.patches import Patch
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset
import numpy as np

from graph_helpers import *


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
    width_name = f"statewalk_width_{"liveon" if is_liveon else "liveoff"}_satelliteoff_{"avg" if is_average else "max"}"
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
  width_key = f"statewalk_width_{'liveon' if is_liveon else 'liveoff'}_satelliteoff_{'avg' if is_average else 'max'}"

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


def make_statewalk_width_performance_scatter(data, output, plot_ilp, is_liveon, is_average, scale_by_egraph_size, width_min=None):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  width_key = f"statewalk_width_{'liveon' if is_liveon else 'liveoff'}_satelliteoff_{'avg' if is_average else 'max'}"

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

  for sample in points:
    width = sample[width_key]
    if width is None:
      raise KeyError(f"Missing {width_key} in sample for benchmark {sample.get('benchmark')}")
    if width_min is not None and width < width_min:
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
      extract_time = sample["extract_time_liveon_satelliteon"]
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
  benchmarks = [b for b in benchmarks if b != 'raytrace']
  if not benchmarks:
    print("WARNING: No benchmarks available after filtering raytrace; skipping heatmap")
    return
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  width_key = f"statewalk_width_{'liveon' if is_liveon else 'liveoff'}_satelliteoff_{'avg' if is_average else 'max'}"

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
      extract_time = sample.get("extract_time_liveon_satelliteon")
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
    plt.ylim(bottom=0)
  plt.title(title)

  plt.tight_layout()
  plt.savefig(output)
