import matplotlib.pyplot as plt
from graph_helpers import *


def make_ilp_encoding_scatter(data, output):
  benchmarks = dedup([b["benchmark"] for b in data if "benchmark" in b])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  solved_sizes = []
  solved_encodings = []
  timeout_sizes = []
  timeout_encodings = []
  infeasible_sizes = []
  infeasible_encodings = []

  for sample in points:
    if "egraph_size" not in sample:
      raise KeyError("ILP encoding scatter requires 'egraph_size' in every sample")
    if "ilp_encoding_num_vars" not in sample:
      raise KeyError("ILP encoding scatter requires 'ilp_encoding_num_vars' in every sample")
    egraph_size = sample["egraph_size"]
    encoding_size = sample["ilp_encoding_num_vars"]
    if egraph_size <= 0 or encoding_size <= 0:
      raise ValueError("ILP encoding scatter received non-positive egraph or encoding size")

    if "ilp_infeasible" not in sample:
      raise KeyError("ILP encoding scatter requires 'ilp_infeasible' in every sample")
    if sample["ilp_infeasible"]:
      infeasible_sizes.append(egraph_size)
      infeasible_encodings.append(encoding_size)
      continue
    if "ilp_timed_out" not in sample:
      raise KeyError("ILP encoding scatter requires 'ilp_timed_out' in every sample")
    if sample["ilp_timed_out"]:
      timeout_sizes.append(egraph_size)
      timeout_encodings.append(encoding_size)
      continue

    solved_sizes.append(egraph_size)
    solved_encodings.append(encoding_size)

  if not (solved_sizes or timeout_sizes or infeasible_sizes):
    print("WARNING: No ILP encoding data available for scatter plot")
    return

  if not solved_sizes and (timeout_sizes or infeasible_sizes):
    print(
      "WARNING: No solved ILP samples for encoding scatter; plotting only "
      f"timeouts ({len(timeout_sizes)}) and infeasible cases ({len(infeasible_sizes)})."
    )

  plt.figure(figsize=(10, 6))

  if solved_sizes:
    plt.scatter(
      solved_sizes,
      solved_encodings,
      color='green',
      label='Gurobi (Solved)',
      alpha=0.7,
      edgecolors='black',
      linewidths=0.5,
      s=60,
      marker='o',
    )

  if timeout_sizes:
    plt.scatter(
      timeout_sizes,
      timeout_encodings,
      color='red',
      label='Gurobi (Timeout)',
      alpha=0.9,
      linewidths=1.2,
      s=70,
      marker='x',
    )

  if infeasible_sizes:
    plt.scatter(
      infeasible_sizes,
      infeasible_encodings,
      color='orange',
      label='Gurobi (Infeasible)',
      alpha=0.9,
      edgecolors='black',
      linewidths=0.5,
      s=65,
      marker='^',
    )

  plt.xlabel('E-graph Size (# of Terms)', fontsize=24)
  plt.ylabel('ILP Encoding Size (# of Variables)', fontsize=24)
  plt.title('ILP Encoding Size vs E-graph Size', fontsize=28)

  ax = plt.gca()
  xlim = ax.get_xlim()
  ylim = ax.get_ylim()
  lower = min(xlim[0], ylim[0])
  upper = max(xlim[1], ylim[1])
  ax.plot(
    [lower, upper],
    [lower, upper],
    linestyle='--',
    color='blue',
    linewidth=1,
    label='x = y',
    zorder=3,
  )
  ax.set_xlim(xlim)
  ax.set_ylim(ylim)

  ax.tick_params(axis='both', which='major', labelsize=22)

  plt.grid(alpha=0.3)
  plt.legend(loc='upper left', fontsize=20)
  plt.tight_layout()
  plt.savefig(output)


def _collect_encoding_time_categories(
  points,
  solved_time_key,
  timeout_key,
  infeasible_key,
  fast_cutoff,
  medium_cutoff,
  color_palette,
  solver_name,
):
  categories = [
    {
      "label": f"Fast (< {fast_cutoff:.2f}s)",
      "color": color_palette[0],
      "min": 0.0,
      "max": fast_cutoff,
      "encodings": [],
      "times": [],
    },
    {
      "label": f"Medium ({fast_cutoff:.2f}s – {medium_cutoff:.2f}s)",
      "color": color_palette[1],
      "min": fast_cutoff,
      "max": medium_cutoff,
      "encodings": [],
      "times": [],
    },
    {
      "label": f"Slow (≥ {medium_cutoff:.2f}s)",
      "color": color_palette[2],
      "min": medium_cutoff,
      "max": float("inf"),
      "encodings": [],
      "times": [],
    },
  ]

  skipped_timeouts = skipped_infeasible = skipped_missing = 0

  for sample in points:
    if "ilp_encoding_num_vars" not in sample:
      raise KeyError(f"{solver_name} encoding time scatter requires 'ilp_encoding_num_vars' in every sample")
    if timeout_key not in sample:
      raise KeyError(f"{solver_name} encoding time scatter requires '{timeout_key}' in every sample")
    if sample[timeout_key]:
      skipped_timeouts += 1
      continue
    if infeasible_key not in sample:
      raise KeyError(f"{solver_name} encoding time scatter requires '{infeasible_key}' in every sample")
    if sample[infeasible_key]:
      skipped_infeasible += 1
      continue
    if solved_time_key not in sample:
      raise KeyError(f"{solver_name} encoding time scatter requires '{solved_time_key}' in every sample")
    duration = sample[solved_time_key]
    if duration is None:
      skipped_missing += 1
      continue

    solve_time = duration_to_seconds(duration)
    if solve_time <= 0:
      raise ValueError(f"Encountered non-positive {solver_name} solve time when building encoding scatter")
    encoding_size = sample["ilp_encoding_num_vars"]
    if encoding_size <= 0:
      raise ValueError(f"Encountered non-positive ILP encoding size when building {solver_name} encoding scatter")

    for category in categories:
      if category["min"] <= solve_time < category["max"]:
        category["encodings"].append(encoding_size)
        category["times"].append(solve_time)
        break

  return categories, skipped_timeouts, skipped_infeasible, skipped_missing


def _render_encoding_time_scatter(categories, solver_label, fast_cutoff, medium_cutoff, output):
  total_points = sum(len(cat["times"]) for cat in categories)

  if total_points == 0:
    return False

  fig, (ax_overview, ax_zoom) = plt.subplots(
    2,
    1,
    sharex=True,
    figsize=(10, 10),
    gridspec_kw={"height_ratios": [2.0, 1.0], "hspace": 0.08},
  )

  marker_style = dict(alpha=0.65, edgecolors='black', linewidths=0.4, s=36)

  for category in categories:
    if not category["times"]:
      continue
    label = f"{category['label']} (n={len(category['times'])})"
    ax_overview.scatter(
      category["encodings"],
      category["times"],
      color=category["color"],
      label=label,
      **marker_style,
    )
    ax_zoom.scatter(
      category["encodings"],
      category["times"],
      color=category["color"],
      label='_nolegend_',
      **marker_style,
    )

  ax_overview.set_title(f'{solver_label} ILP Solve Time vs Encoding Size', fontsize=28)
  ax_overview.set_ylabel('ILP Solve Time (Seconds)', fontsize=24)
  ax_overview.set_yscale('log')
  ax_overview.set_xscale('log')
  ax_overview.tick_params(axis='both', which='major', labelsize=22)
  ax_overview.grid(alpha=0.3, which='both', linestyle='--', linewidth=0.7)

  ax_zoom.set_ylabel('ILP Solve Time (Seconds)', fontsize=24)
  ax_zoom.set_xlabel('ILP Encoding Size (# of Variables)', fontsize=24)
  ax_zoom.set_xscale('log')
  ax_zoom.set_ylim(0, max(medium_cutoff * 2, 0.3))
  ax_zoom.tick_params(axis='both', which='major', labelsize=22)
  ax_zoom.grid(alpha=0.3, linestyle='--', linewidth=0.7)

  for threshold in (fast_cutoff, medium_cutoff):
    ax_zoom.axhline(threshold, color='gray', linestyle=':', linewidth=1.0, alpha=0.6)
    ax_overview.axhline(threshold, color='gray', linestyle=':', linewidth=1.0, alpha=0.3)

  handles, labels = ax_overview.get_legend_handles_labels()
  if handles:
    ax_overview.legend(handles, labels, loc='lower right', fontsize=20)

  fig.tight_layout(rect=[0.0, 0.0, 1.0, 0.98])
  fig.savefig(output)
  return True


def make_ilp_encoding_time_scatter(
  data,
  output,
  run_method="eggcc-tiger-ILP-COMPARISON",
  fast_cutoff=0.08,
  medium_cutoff=0.15,
):
  benchmarks = dedup([b["benchmark"] for b in data if "benchmark" in b])
  points = all_region_extract_points(run_method, data, benchmarks)

  categories, skipped_timeouts, skipped_infeasible, skipped_missing = _collect_encoding_time_categories(
    points,
    "ilp_extract_time",
    "ilp_timed_out",
    "ilp_infeasible",
    fast_cutoff,
    medium_cutoff,
    ["#2ca02c", "#ff7f0e", "#d62728"],
    "Gurobi",
  )

  if sum(len(cat["times"]) for cat in categories) == 0:
    print(
      "WARNING: No solved ILP timing samples available for encoding/time scatter "
      f"(skipped {skipped_timeouts} timeouts, {skipped_infeasible} infeasible, {skipped_missing} missing durations)."
    )
    return

  if skipped_timeouts or skipped_infeasible or skipped_missing:
    print(
      "INFO: make_ilp_encoding_time_scatter skipped"
      f" {skipped_timeouts} timeouts, {skipped_infeasible} infeasible regions,"
      f" {skipped_missing} samples without solve times."
    )

  _render_encoding_time_scatter(
    categories,
    "Gurobi",
    fast_cutoff,
    medium_cutoff,
    output,
  )


def make_cbc_encoding_time_scatter(
  data,
  output,
  run_method="eggcc-tiger-ILP-COMPARISON",
  fast_cutoff=0.2,
  medium_cutoff=0.6,
):
  benchmarks = dedup([b["benchmark"] for b in data if "benchmark" in b])
  points = all_region_extract_points(run_method, data, benchmarks)

  categories, skipped_timeouts, skipped_infeasible, skipped_missing = _collect_encoding_time_categories(
    points,
    "cbc_ilp_extract_time",
    "cbc_ilp_timed_out",
    "cbc_ilp_infeasible",
    fast_cutoff,
    medium_cutoff,
    ["#1f77b4", "#9467bd", "#8c564b"],
    "CBC",
  )

  total_points = sum(len(cat["times"]) for cat in categories)
  if total_points == 0:
    print(
      "WARNING: No CBC ILP timing samples available for encoding/time scatter "
      f"(skipped {skipped_timeouts} timeouts, {skipped_infeasible} infeasible, {skipped_missing} missing durations)."
    )
    return

  if skipped_timeouts or skipped_infeasible or skipped_missing:
    print(
      "INFO: make_cbc_encoding_time_scatter skipped"
      f" {skipped_timeouts} timeouts, {skipped_infeasible} infeasible regions,"
      f" {skipped_missing} samples without solve times."
    )

  _render_encoding_time_scatter(
    categories,
    "CBC",
    fast_cutoff,
    medium_cutoff,
    output,
  )
