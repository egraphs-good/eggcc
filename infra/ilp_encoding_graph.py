import matplotlib.pyplot as plt
from graph_helpers import *


def make_ilp_encoding_scatter(data, output):
  benchmarks = dedup([b["benchmark"] for b in data if "benchmark" in b])
  points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", data, benchmarks)

  solved_sizes = []
  solved_encodings = []
  timeout_count = 0
  infeasible_count = 0

  for sample in points:
    if "egraph_size" not in sample:
      raise KeyError("ILP encoding scatter requires 'egraph_size' in every sample")
    if "ilp_encoding_num_vars" not in sample:
      raise KeyError("ILP encoding scatter requires 'ilp_encoding_num_vars' in every sample")
    egraph_size = sample["egraph_size"]
    encoding_size = sample["ilp_encoding_num_vars"]
    if egraph_size <= 0 or encoding_size <= 0:
      raise ValueError("ILP encoding scatter received non-positive egraph or encoding size")

    if sample["ilp_infeasible"]:
      infeasible_count += 1
      continue
    if sample["ilp_timed_out"]:
      timeout_count += 1
      continue
    else:
      solved_sizes.append(egraph_size)
      solved_encodings.append(encoding_size)

  if not solved_sizes:
    if timeout_count or infeasible_count:
      print(
        "WARNING: No solved ILP samples for encoding scatter; all candidates "
        f"timed out ({timeout_count}) or were infeasible ({infeasible_count})."
      )
    else:
      print("WARNING: No ILP encoding data available for scatter plot")
    return

  plt.figure(figsize=(10, 6))

  plt.scatter(
    solved_sizes,
    solved_encodings,
    color='green',
    label='ILP Encoding (Solved)',
    alpha=0.7,
    edgecolors='black',
    linewidths=0.5,
    s=60,
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
