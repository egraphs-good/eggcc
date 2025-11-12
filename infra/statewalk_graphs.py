from collections import Counter
from dataclasses import dataclass
from typing import Iterable, Optional

import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from matplotlib.patches import Patch
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset
import numpy as np

from graph_helpers import *


StatewalkRuntime = str
SCATTER_WIDTH_CONFIGURATION = "Live-Off, Satellite-Off"


@dataclass(frozen=True)
class StatewalkTreatment:
  runtime: StatewalkRuntime
  liveness_on: bool
  satellite_on: bool
  region_run_method: str = "eggcc-tiger-ILP-COMPARISON"
  label: Optional[str] = None

  def width_key(self, is_average: bool) -> str:
    live_part = "liveon" if self.liveness_on else "liveoff"
    satellite_part = "satelliteon" if self.satellite_on else "satelliteoff"
    agg_part = "avg" if is_average else "max"
    return f"statewalk_width_{live_part}_{satellite_part}_{agg_part}"

  def runtime_display_name(self) -> str:
    return {
      "tiger": "Statewalk DP",
      "ilp_gurobi": "Gurobi",
      "ilp_cbc": "CBC",
    }.get(self.runtime, self.runtime)

  def duration_field(self) -> str:
    if self.runtime == "tiger":
      live_part = "liveon" if self.liveness_on else "liveoff"
      satellite_part = "satelliteon" if self.satellite_on else "satelliteoff"
      return f"extract_time_{live_part}_{satellite_part}"
    if self.runtime == "ilp_gurobi":
      return "ilp_extract_time"
    if self.runtime == "ilp_cbc":
      return "cbc_ilp_extract_time"
    raise ValueError(f"Unknown runtime source {self.runtime}")

  def timeout_field(self) -> Optional[str]:
    if self.runtime == "tiger":
      return None
    if self.runtime == "ilp_gurobi":
      return "ilp_timed_out"
    if self.runtime == "ilp_cbc":
      return "cbc_ilp_timed_out"
    raise ValueError(f"Unknown runtime source {self.runtime}")

  def infeasible_field(self) -> Optional[str]:
    if self.runtime == "tiger":
      return None
    if self.runtime == "ilp_gurobi":
      return "ilp_infeasible"
    if self.runtime == "ilp_cbc":
      return "cbc_ilp_infeasible"
    raise ValueError(f"Unknown runtime source {self.runtime}")

  def timeout_label(self) -> str:
    return "Timeout (5 min)"

  def timeout_color(self) -> str:
    if self.runtime == "ilp_cbc":
      return "#8c564b"
    if self.runtime == "ilp_gurobi":
      return "#d62728"
    return "red"

  def infeasible_label(self) -> str:
    if self.runtime == "ilp_cbc":
      return "ILP (CBC) Infeasible"
    return "ILP Infeasible"

  def display_name(self) -> str:
    if self.label:
      return self.label
    runtime_name = self.runtime_display_name()
    modifier_text = ""
    
    if self.runtime == "tiger":
      modifiers = [
        "Live-On" if self.liveness_on else "Live-Off",
        "Satellite-On" if self.satellite_on else "Satellite-Off",
      ]
      modifier_text = f" ({', '.join(modifiers)})"
    return f"{runtime_name}{modifier_text}"

  def color(self) -> str:
    if self.runtime == "tiger":
      if self.liveness_on and self.satellite_on:
        return COLOR_MAP.get("eggcc-tiger-WL-O0-O0", "magenta")
      return COLOR_MAP.get("eggcc-tiger-O0-O0", "blue")
    if self.runtime == "ilp_gurobi":
      return COLOR_MAP.get("eggcc-tiger-ILP-O0-O0", "green")
    if self.runtime == "ilp_cbc":
      return COLOR_MAP.get("eggcc-tiger-ILP-CBC-O0-O0", "olive")
    return "black"

  def modifiers_suffix(self) -> str:
    if self.label:
      return self.label
    modifiers = [
      "Live-On" if self.liveness_on else "Live-Off",
      "Satellite-On" if self.satellite_on else "Satellite-Off",
    ]
    return " ".join(modifiers)



def make_statewalk_width_histogram(data, output, treatment: StatewalkTreatment, is_average, max_width=None):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points(treatment.region_run_method, data, benchmarks)

  widths = []
  missing_widths = 0
  filtered_above_threshold = 0
  max_width_label = None
  if max_width is not None:
    max_width_label = f"{int(max_width):,}" if float(max_width).is_integer() else f"{max_width:g}"

  width_key = treatment.width_key(is_average)

  for sample in points:
    width = sample.get(width_key)
    if width is None:
      missing_widths += 1
      continue
    if max_width is not None and width > max_width:
      filtered_above_threshold += 1
      continue
    widths.append(width)

  if missing_widths:
    print(
      f"WARNING: Skipping {missing_widths} timing samples with missing {width_key}"
    )
  if filtered_above_threshold and max_width_label is not None:
    print(
      f"WARNING: Skipping {filtered_above_threshold} samples with {width_key} > {max_width_label}"
    )

  if not widths:
    print("WARNING: No statewalk width data found; skipping histogram")
    return

  counts = Counter(widths)
  sorted_widths = sorted(counts.keys())
  frequencies = [counts[w] for w in sorted_widths]

  plt.figure(figsize=(10, 6))
  plt.bar(sorted_widths, frequencies, color='skyblue', edgecolor='black')
  treatment_label = treatment.display_name()
  plt.xlabel(f"Statewalk Width{' Average' if is_average else ''} ({treatment.modifiers_suffix()})")
  plt.ylabel('Number of Regionalized E-Graphs')
  title = f"Distribution of Statewalk Width – {treatment_label}"
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
  treatment: StatewalkTreatment,
  is_average,
  limit=10,
  max_width=None,
):
  benchmarks = dedup([b.get('benchmark') for b in data])
  width_key = treatment.width_key(is_average)

  max_width_label = None
  if max_width is not None:
    max_width_label = f"{int(max_width):,}" if float(max_width).is_integer() else f"{max_width:g}"

  samples = []

  for benchmark in benchmarks:
    timings = get_extract_region_timings(treatment.region_run_method, data, benchmark)
    if timings is False:
      continue
    for sample in timings:
      width = sample.get(width_key)
      if width is None:
        continue
      if max_width is not None and width > max_width:
        continue
      samples.append((width, benchmark))

  if not samples:
    print(f"WARNING: No statewalk width data found for {treatment.display_name()} ({width_key})")
    return

  samples.sort(key=lambda entry: entry[0], reverse=True)
  if limit is None or limit <= 0:
    limit = 10
  limit = min(limit, len(samples))

  descriptor_parts = [
    'live-on' if treatment.liveness_on else 'live-off',
    'average' if is_average else 'maximum',
    'satellite-on' if treatment.satellite_on else 'satellite-off',
  ]
  descriptor = ' '.join(descriptor_parts)
  filter_suffix = f" (≤ {max_width_label})" if max_width_label is not None else ''

  print(f"Top {limit} statewalk widths ({descriptor}) for {treatment.display_name()}{filter_suffix}:")

  def _format_width(value):
    if isinstance(value, (int, np.integer)):
      return f"{int(value):,}"
    if isinstance(value, float):
      if value.is_integer():
        return f"{int(value):,}"
      return f"{value:.4g}"
    return str(value)

  for idx, (width, benchmark) in enumerate(samples[:limit], start=1):
    width_display = _format_width(width)
    print(f"  {idx}. {benchmark} – {width_display}")


def _collect_statewalk_scatter_points(
  points,
  treatment: StatewalkTreatment,
  is_average,
  scale_by_egraph_size,
  width_min,
  width_max=None,
):
  def _jitter(values, magnitude=0.5):
    if not values:
      return values
    noise = np.random.uniform(-magnitude, magnitude, size=len(values))
    jittered = []
    for value, delta in zip(values, noise):
      jittered_value = value + float(delta)
      if jittered_value < 0:
        jittered_value = 0.0
      jittered.append(jittered_value)
    return jittered

  agg_part = "avg" if is_average else "max"
  width_key = f"statewalk_width_liveoff_satelliteoff_{agg_part}"
  duration_field = treatment.duration_field()
  timeout_field = treatment.timeout_field()
  infeasible_field = treatment.infeasible_field()
  is_ilp_runtime = timeout_field is not None

  x_values = []
  y_values = []
  timeout_x = []
  timeout_y = []
  infeasible_x = []
  infeasible_y = []

  for sample in points:
    width = sample.get(width_key)
    if width is None:
      raise KeyError(f"Missing {width_key} in sample for benchmark {sample.get('benchmark')}")
    if width_min is not None and width < width_min:
      continue
    if width_max is not None and width > width_max:
      continue

    x_magnitude = width
    if scale_by_egraph_size:
      egraph_size = sample.get("egraph_size")
      if egraph_size is None:
        raise KeyError(f"Missing egraph_size in sample for benchmark {sample.get('benchmark')}")
      if egraph_size <= 0:
        raise ValueError(f"Non-positive egraph_size in sample for benchmark {sample.get('benchmark')}")
      x_magnitude = width * egraph_size

    if is_ilp_runtime:
      if sample.get(infeasible_field, False):
        infeasible_x.append(x_magnitude)
        infeasible_y.append(ILP_TIMEOUT_SECONDS)
        continue
      if sample.get(timeout_field, False):
        timeout_x.append(x_magnitude)
        timeout_y.append(ILP_TIMEOUT_SECONDS)
        continue

    runtime_value = sample.get(duration_field)
    if runtime_value is None:
      raise KeyError(f"Missing {duration_field} in sample for benchmark {sample.get('benchmark')}")
      
    value = duration_to_seconds(runtime_value)

    x_values.append(x_magnitude)
    y_values.append(value)

  return {
    "x_values": x_values,
    "y_values": y_values,
    "timeout_x": timeout_x,
    "timeout_y": _jitter(timeout_y),
    "infeasible_x": infeasible_x,
    "infeasible_y": infeasible_y,
    "duration_field": duration_field,
    "is_ilp_runtime": is_ilp_runtime,
  }


def make_statewalk_width_performance_scatter(
  data,
  output,
  treatment: StatewalkTreatment,
  is_average,
  scale_by_egraph_size,
  width_min=None,
):
  benchmarks = dedup([b.get('benchmark') for b in data])
  points = all_region_extract_points(treatment.region_run_method, data, benchmarks)

  results = _collect_statewalk_scatter_points(points, treatment, is_average, scale_by_egraph_size, width_min)
  duration_field = results["duration_field"]
  is_ilp_runtime = results["is_ilp_runtime"]

  plt.figure(figsize=(10, 6))

  plotted_any = False
  primary_label = treatment.display_name()
  primary_color = treatment.color()

  x_values = results["x_values"]
  y_values = results["y_values"]
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

  timeout_x = results["timeout_x"]
  timeout_y = results["timeout_y"]
  infeasible_x = results["infeasible_x"]
  infeasible_y = results["infeasible_y"]

  if is_ilp_runtime and timeout_x:
    plt.scatter(
      timeout_x,
      timeout_y,
      color=treatment.timeout_color(),
      marker='x',
      label=treatment.timeout_label(),
      linewidths=2.0,
      s=100,
    )
    plotted_any = True

  if is_ilp_runtime and infeasible_x:
    plt.scatter(
      infeasible_x,
      infeasible_y,
      color='orange',
      marker='x',
      label=treatment.infeasible_label(),
      linewidths=2.0,
      s=100,
    )
    plotted_any = True

  if not plotted_any:
    print("WARNING: No data plotted in make_statewalk_width_performance_scatter")
    plt.close()
    return

  if scale_by_egraph_size:
    if is_average:
      x_label = "Statewalk Width Average × E-graph Size"
    else:
      x_label = "Statewalk Width × E-graph Size"
    x_label += f" ({SCATTER_WIDTH_CONFIGURATION})"
  else:
    x_label = f"Statewalk Width{' Average' if is_average else ''}"
    x_label += f" ({SCATTER_WIDTH_CONFIGURATION})"

  plt.xlabel(x_label)
  ylabel = 'Runtime (Seconds)'
  plt.ylabel(ylabel)

  title = f"Statewalk Width vs Runtime – {treatment.display_name()}"
  if is_average and not scale_by_egraph_size:
    title += ' (Average Width)'
  if scale_by_egraph_size:
    title += ' (Width × Size)'
  plt.title(title)

  plt.grid(alpha=0.3)

  ax = plt.gca()
  ax.set_xscale('log')

  plt.tight_layout()
  plt.savefig(output)


def make_statewalk_width_performance_scatter_multi(
  data,
  output,
  treatments: Iterable[StatewalkTreatment],
  is_average,
  scale_by_egraph_size,
  width_min=None,
  width_max=None,
):
  treatment_list = list(treatments)
  if len(treatment_list) < 2:
    raise ValueError("Expected at least two treatments for multi scatter plot")

  benchmarks = dedup([b.get('benchmark') for b in data])

  plt.figure(figsize=(10, 6))
  plotted_any = False

  for treatment in treatment_list:
    points = all_region_extract_points(treatment.region_run_method, data, benchmarks)
    results = _collect_statewalk_scatter_points(points, treatment, is_average, scale_by_egraph_size, width_min, width_max)

    x_values = results["x_values"]
    y_values = results["y_values"]

    if not x_values:
      continue

    plt.scatter(
      x_values,
      y_values,
      color=treatment.color(),
      label=treatment.display_name(),
      alpha=0.7,
      edgecolors='black',
      linewidths=0.5,
      s=60,
    )
    plotted_any = True

    if results["is_ilp_runtime"]:
      if results["timeout_x"]:
        plt.scatter(
          results["timeout_x"],
          results["timeout_y"],
          color=treatment.timeout_color(),
          marker='x',
          label=f"{treatment.display_name()} – {treatment.timeout_label()}",
          linewidths=2.0,
          s=100,
        )
      if results["infeasible_x"]:
        plt.scatter(
          results["infeasible_x"],
          results["infeasible_y"],
          color='orange',
          marker='x',
          label=f"{treatment.display_name()} – {treatment.infeasible_label()}",
          linewidths=2.0,
          s=100,
        )

  if not plotted_any:
    print("WARNING: No data plotted in make_statewalk_width_performance_scatter_multi")
    plt.close()
    return

  if scale_by_egraph_size:
    if is_average:
      x_label = "Statewalk Width Average × E-graph Size"
    else:
      x_label = "Statewalk Width × E-graph Size"
  else:
    x_label = f"Statewalk Width{' Average' if is_average else ''}"
  x_label += f" ({SCATTER_WIDTH_CONFIGURATION})"

  plt.xlabel(x_label)
  plt.ylabel('Runtime (Seconds)')

  comparison_labels = ' vs '.join(t.display_name() for t in treatment_list)
  title = f"Statewalk Width vs Runtime ({comparison_labels})"
  if scale_by_egraph_size:
    title += ' (Width × Size)'
  plt.title(title)

  plt.grid(alpha=0.3)
  plt.legend(loc='best')

  ax = plt.gca()
  ax.set_xscale('log')

  plt.tight_layout()
  plt.savefig(output)


def make_egraph_size_vs_statewalk_width_heatmap(
  data,
  output,
  treatment: StatewalkTreatment,
  is_average,
  min_width=None,
  max_width=None,
):
  benchmarks = dedup([b.get('benchmark') for b in data])
  benchmarks = [b for b in benchmarks if b != 'raytrace']
  if not benchmarks:
    print("WARNING: No benchmarks available after filtering raytrace; skipping heatmap")
    return
  points = all_region_extract_points(treatment.region_run_method, data, benchmarks)

  width_key = treatment.width_key(is_average)
  duration_field = treatment.duration_field()
  timeout_field = treatment.timeout_field()
  infeasible_field = treatment.infeasible_field()
  is_ilp_runtime = timeout_field is not None

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

    if is_ilp_runtime:
      if sample.get(infeasible_field, False):
        if egraph_size is not None:
          infeasible_sizes.append(egraph_size)
          infeasible_widths.append(width)
        continue
      if sample.get(timeout_field, False):
        if egraph_size is not None:
          timeout_sizes.append(egraph_size)
          timeout_widths.append(width)
        continue

    runtime_value = sample.get(duration_field)
    if runtime_value is None:
      missing_runtimes += 1
      continue
    runtime_secs = duration_to_seconds(runtime_value)

    if runtime_secs <= 0:
      continue

    sizes.append(egraph_size)
    widths.append(width)
    runtimes.append(runtime_secs)

  if missing_runtimes:
    print(f"WARNING: Skipping {missing_runtimes} samples with missing {duration_field}")

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
  cbar.set_label(f"{treatment.display_name()} Runtime (Seconds)")

  legend_handles = []
  legend_labels = []

  if is_ilp_runtime:
    solved_points = plt.scatter(
      sizes,
      widths,
      color='white',
      edgecolors='black',
      linewidths=0.2,
      s=20,
      alpha=0.6,
      zorder=3,
      label='Solved Points',
    )
    legend_handles.append(solved_points)
    legend_labels.append('Solved Points')

  if is_ilp_runtime and timeout_sizes:
    timeout_scatter = plt.scatter(
      timeout_sizes,
      timeout_widths,
      marker='x',
      color='red',
      linewidths=1.5,
      s=60,
      label=treatment.timeout_label(),
      zorder=4,
    )
    legend_handles.append(timeout_scatter)
    legend_labels.append(treatment.timeout_label())

  if is_ilp_runtime and infeasible_sizes:
    infeasible_scatter = plt.scatter(
      infeasible_sizes,
      infeasible_widths,
      marker='x',
      color='orange',
      linewidths=1.5,
      s=60,
      label=treatment.infeasible_label(),
      zorder=4,
    )
    legend_handles.append(infeasible_scatter)
    legend_labels.append(treatment.infeasible_label())

  if legend_handles:
    plt.legend(legend_handles, legend_labels, loc='upper right')

  plt.xlabel('Regionalized E-graph Size')
  y_label = 'Statewalk Width'
  y_label += ' Average' if is_average else ' Maximum'
  y_label += f" ({treatment.modifiers_suffix()})"
  plt.ylabel(y_label)

  title = f"Runtime Heatmap vs Statewalk Width – {treatment.display_name()}"
  if is_average:
    title += ' (Average Width)'
  if max_width is not None:
    title += f' (≤ Width {max_width})'
    plt.ylim(bottom=0)
  plt.title(title)

  plt.tight_layout()
  plt.savefig(output)
