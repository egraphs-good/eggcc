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
      label='ILP Encoding (Solved)',
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
      label='ILP Encoding (Timeout)',
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
      label='ILP Encoding (Infeasible)',
      alpha=0.9,
      edgecolors='black',
      linewidths=0.5,
      s=65,
      marker='^',
    )

  plt.xlabel('E-graph Size')
  plt.ylabel('ILP Encoding Size (Edge Variables)')
  plt.title('ILP Encoding Size vs E-graph Size')

  ax = plt.gca()
  ax.set_xscale('log')
  ax.set_yscale('log')

  plt.grid(alpha=0.3)
  plt.legend(loc='lower right')
  plt.tight_layout()
  plt.savefig(output)
