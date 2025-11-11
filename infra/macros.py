from collections import Counter
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from matplotlib.patches import Patch
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset
import numpy as np
import os
from graph_helpers import *


# note: use ["..."] for indexing samples instead of .get(...) to fail fast on missing keys
# never hide errors about missing keys or anything like that
# if data is empty so you can't compute a number, don't default to 0, abort instead with a warning

# given a profile.json, list of suite paths, and an output file
def make_macros(profile, benchmark_suites, output_file):
  with open(output_file, 'a') as out:
    benchmarks = dedup([row["benchmark"] for row in profile])
    benchmark_regions = {benchmark: 0 for benchmark in benchmarks}

    region_points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", profile, benchmarks)

    for sample in region_points:
      benchmark_name = sample.get("benchmark")
      if benchmark_name is None:
        print("WARNING: region point missing benchmark name; skipping benchmark macro generation")
        return
      if benchmark_name not in benchmark_regions:
        print(f"WARNING: Unexpected benchmark {benchmark_name} in region data; aborting macro generation")
        return
      benchmark_regions[benchmark_name] += 1

    for benchmark, count in benchmark_regions.items():
      macro_name = f"NumSubregions{convert_string_to_valid_latex_var(benchmark)}"
      out.write(format_latex_macro(macro_name, count))

    region_counts = list(benchmark_regions.values())
    avg_regions = mean(region_counts)
    out.write(format_latex_macro("AvgNumSubregionsPerBenchmark", f"{avg_regions:.2f}"))
    max_regions = max(region_counts)
    out.write(format_latex_macro("MaxNumSubregionsPerBenchmark", max_regions))

    # report number of benchmarks in each benchmark suite
    for suite in benchmark_suites:
      suite_name = os.path.basename(suite)
      suite_benchmarks = benchmarks_in_folder(suite)
      macro_name = f"Num{suite_name}Benchmarks"
      out.write(format_latex_macro(macro_name, len(suite_benchmarks)))

    # report the number of benchmarks in the profile
    out.write(format_latex_macro("NumBenchmarksAllSuites", len(benchmarks)))

    ilp_timeout_benchmarks = {
      row["benchmark"]
      for row in profile
      if row["runMethod"] == 'eggcc-tiger-ILP-O0-O0' and row["ILPRegionTimeOut"]
    }
    if None in ilp_timeout_benchmarks:
      ilp_timeout_benchmarks.discard(None)
    out.write(format_latex_macro("NumeggcctigerILPGurobiRegionTimeoutBenchmarks", len(ilp_timeout_benchmarks)))

    ilp_cbc_timeout_benchmarks = {
      row["benchmark"]
      for row in profile
      if row["runMethod"] == 'eggcc-tiger-ILP-CBC-O0-O0' and row["ILPRegionTimeOut"]
    }
    if None in ilp_cbc_timeout_benchmarks:
      ilp_cbc_timeout_benchmarks.discard(None)
    out.write(format_latex_macro("NumeggcctigerILPCBCRegionTimeoutBenchmarks", len(ilp_cbc_timeout_benchmarks)))

    total_regionalized_egraphs = len(region_points)
    if total_regionalized_egraphs == 0:
      print("WARNING: No regionalized e-graphs found; skipping ILP macro generation")
      return
    out.write(format_latex_macro("NumRegionalizedEgraphs", total_regionalized_egraphs))

    ilp_region_timeout_count = sum(1 for sample in region_points if sample["ilp_timed_out"])
    out.write(format_latex_macro("NumILPGurobiRegionTimeouts", ilp_region_timeout_count))
    timeout_ratio = ilp_region_timeout_count / total_regionalized_egraphs
    out.write(format_latex_macro_percent("PercentILPGurobiRegionTimeouts", timeout_ratio))

    ilp_infeasible_count = sum(1 for sample in region_points if sample.get("ilp_infeasible"))
    out.write(format_latex_macro("NumILPGurobiInfeasibleRegions", ilp_infeasible_count))
    infeasible_ratio = ilp_infeasible_count / total_regionalized_egraphs
    out.write(format_latex_macro_percent("PercentILPGurobiInfeasibleRegions", infeasible_ratio))

    ilp_region_times = [
      duration_to_seconds(sample["ilp_extract_time"])
      for sample in region_points
      if (not sample["ilp_timed_out"]) and (not sample["ilp_infeasible"])
    ]
    if not ilp_region_times:
      print("WARNING: No ILP extract times available; skipping AvgILPGurobiRegionExtractTimeSecs macro")
      return
    avg_ilp_region_time = mean(ilp_region_times)
    out.write(format_latex_macro("AvgILPGurobiRegionExtractTimeSecs", f"{avg_ilp_region_time:.6f}"))

    tiger_region_times = [
      duration_to_seconds(sample["extract_time_liveon_satelliteon"])
      for sample in region_points
    ]
    if not tiger_region_times:
      print("WARNING: No tiger extract times available; skipping AvgTigerLiveOnSatelliteOnRegionExtractTimeSecs macro")
      return
    avg_tiger_region_time = mean(tiger_region_times)
    out.write(format_latex_macro("AvgTigerLiveOnSatelliteOnRegionExtractTimeSecs", f"{avg_tiger_region_time:.6f}"))
    max_tiger_region_time = max(tiger_region_times)
    out.write(format_latex_macro("MaxTigerLiveOnSatelliteOnRegionExtractTimeSecs", f"{max_tiger_region_time:.6f}"))

    width_field = "statewalk_width_liveon_satelliteon_max"
    statewalk_widths = [sample[width_field] for sample in region_points]
    total_regions = len(statewalk_widths)

    if total_regions == 0:
      print("WARNING: No statewalk width data available; skipping statewalk width macros")
      return
    for threshold in range(1, 31):
      under_count = sum(1 for width in statewalk_widths if width < threshold)
      percent_under = under_count / total_regions
      macro_name = f"PercentRegionsStatewalkWidthUnder{threshold}"
      out.write(format_latex_macro_percent(macro_name, percent_under))

    top_widths = sorted(set(statewalk_widths), reverse=True)[:20]
    for width_value in top_widths:
      count_at_or_above = sum(1 for width in statewalk_widths if width >= width_value)
      macro_name = f"NumRegionsStatewalkWidthAbove{width_value}"
      out.write(format_latex_macro(macro_name, count_at_or_above))
