from collections import Counter
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker
from matplotlib.patches import Patch
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset
import numpy as np
import os
from graph_helpers import *


# note: use ["..."] for indexing samples instead of .get(...) to fail fast on missing keys

# given a profile.json, list of suite paths, and an output file
def make_macros(profile, benchmark_suites, output_file):
  with open(output_file, 'a') as out:
    benchmarks = dedup([row["benchmark"] for row in profile])
    benchmark_regions = {benchmark: 0 for benchmark in benchmarks}
    suite_region_counts = {}
    benchmark_suite_map = {}

    region_points = all_region_extract_points("eggcc-tiger-ILP-COMPARISON", profile, benchmarks)

    for benchmark in benchmarks:
      row = get_row(profile, benchmark, "eggcc-tiger-ILP-COMPARISON")
      timings = row["extractRegionTimings"]
      benchmark_suite_map[benchmark] = row["suite"]
      out.write(
        format_latex_macro(
          f"NumSubregions{convert_string_to_valid_latex_var(benchmark)}",
          len(timings),
        )
      )
      benchmark_regions[benchmark] = len(timings)
      if row["suite"] not in suite_region_counts:
        suite_region_counts[row["suite"]] = []
      suite_region_counts[row["suite"]].append(benchmark_regions[benchmark])

    region_counts = list(benchmark_regions.values())
    out.write(
      format_latex_macro(
        "AvgNumSubregionsPerBenchmark",
        f"{mean(region_counts):.2f}",
      )
    )
    out.write(
      format_latex_macro(
        "MaxNumSubregionsPerBenchmark",
        max(region_counts),
      )
    )

    if "polybench" not in suite_region_counts:
      raise ValueError("No polybench suite benchmarks found when computing regionalized e-graphs per benchmark")
    out.write(
      format_latex_macro(
        "AvgPolybenchRegionalizedEgraphsPerBenchmark",
        f"{mean(suite_region_counts['polybench']):.2f}",
      )
    )

    # report number of benchmarks in each benchmark suite
    for suite in benchmark_suites:
      out.write(
        format_latex_macro(
          f"Num{os.path.basename(suite)}Benchmarks",
          len(benchmarks_in_folder(suite)),
        )
      )

    # report the number of benchmarks in the profile
    out.write(format_latex_macro("NumBenchmarksAllSuites", len(benchmarks)))

    bril_benchmarks = [b for b in benchmarks if benchmark_suite_map.get(b) == "bril"]
    if not bril_benchmarks:
      print("WARNING: No bril benchmarks found in profile; skipping Bril performance macro")
    else:
      bril_better_count = 0
      for benchmark in bril_benchmarks:
        eggcc_cycles = get_cycles(profile, benchmark, "eggcc-tiger-O0-O0")
        llvm_cycles = get_cycles(profile, benchmark, "llvm-O3-O0")

        eggcc_mean = mean(eggcc_cycles)
        llvm_mean = mean(llvm_cycles)

        if eggcc_mean < llvm_mean:
          bril_better_count += 1

      out.write(
        format_latex_macro(
          "NumBrilBenchmarksEggcctigerO0O0BetterThanLlvmO3O0",
          bril_better_count,
        )
      )

    out.write(
      format_latex_macro(
        "NumeggcctigerILPGurobiRegionTimeoutBenchmarks",
        len(timeout_benchmarks_for_run(profile, 'eggcc-tiger-ILP-O0-O0')),
      )
    )

    out.write(
      format_latex_macro(
        "NumeggcctigerILPGurobiRegionTimeoutBenchmarksBril",
        len(timeout_benchmarks_for_run(profile, 'eggcc-tiger-ILP-O0-O0', suite='bril')),
      )
    )

    ilp_gurobi_solved_benchmarks = {
      row["benchmark"]
      for row in profile
      if row["runMethod"] == 'eggcc-tiger-ILP-O0-O0' and not row["ILPRegionTimeOut"]
    }
    if None in ilp_gurobi_solved_benchmarks:
      raise ValueError("Found benchmark with name None among ILP Gurobi solved benchmarks")
    out.write(format_latex_macro("NumILPGurobiSolvedBenchmarks", len(ilp_gurobi_solved_benchmarks)))

    tiger_times_on_gurobi_solved = []
    for benchmark in ilp_gurobi_solved_benchmarks:
      tiger_row = get_row(profile, benchmark, 'eggcc-tiger-O0-O0')
      extraction_time = tiger_row["eggccExtractionTimeSecs"]
      if extraction_time is False or extraction_time is None:
        raise ValueError(
          f"Missing eggccExtractionTimeSecs for benchmark {benchmark}; cannot compute average extraction time"
        )
      tiger_times_on_gurobi_solved.append(extraction_time)

    if not tiger_times_on_gurobi_solved:
      raise ValueError("No eggcc-tiger-O0-O0 extraction times available for Gurobi-solved benchmarks")
    out.write(
      format_latex_macro(
        "AvgEggcctigerO0O0ExtractionTimeSecsOnILPGurobiSolvedBenchmarks",
        f"{mean(tiger_times_on_gurobi_solved):.3f}",
      )
    )

    out.write(
      format_latex_macro(
        "NumeggcctigerILPCBCRegionTimeoutBenchmarks",
        len(timeout_benchmarks_for_run(profile, 'eggcc-tiger-ILP-CBC-O0-O0')),
      )
    )

    out.write(
      format_latex_macro(
        "NumeggcctigerILPCBCRegionTimeoutBenchmarksBril",
        len(timeout_benchmarks_for_run(profile, 'eggcc-tiger-ILP-CBC-O0-O0', suite='bril')),
      )
    )

    raytrace_row = get_row(profile, "raytrace", "eggcc-tiger-ILP-COMPARISON")
    raytrace_timings = raytrace_row["extractRegionTimings"]
    out.write(
      format_latex_macro("NumRaytraceRegionalizedEgraphs", len(raytrace_timings))
    )
    out.write(
      format_latex_macro(
        "MaxRaytraceRegionalizedEgraphTerms",
        f"{max(sample["egraph_size"] for sample in raytrace_timings):.4f}",
      )
    )
    out.write(
      format_latex_macro(
        "MaxRaytraceTigerExtractionTimeSecs",
        f"{max(
          duration_to_seconds(sample['extract_time_liveon_satelliteon'])
          for sample in raytrace_timings
        ):.6f}",
      )
    )
    out.write(
      format_latex_macro(
        "NumRaytraceILPRegionalizedEgraphTimeouts",
        sum(1 for sample in raytrace_timings if sample["ilp_timed_out"]),
      )
    )

    total_regionalized_egraphs = len(region_points)
    if total_regionalized_egraphs == 0:
      print("WARNING: No regionalized e-graphs found; skipping ILP macro generation")
      return
    out.write(format_latex_macro("NumRegionalizedEgraphs", total_regionalized_egraphs))

    ilp_region_timeout_count = sum(1 for sample in region_points if sample["ilp_timed_out"])
    out.write(format_latex_macro("NumILPGurobiRegionTimeouts", ilp_region_timeout_count))
    out.write(
      format_latex_macro_percent(
        "PercentILPGurobiRegionTimeouts",
        ilp_region_timeout_count / total_regionalized_egraphs,
      )
    )

    ilp_infeasible_count = sum(1 for sample in region_points if sample.get("ilp_infeasible"))
    out.write(format_latex_macro("NumILPGurobiInfeasibleRegions", ilp_infeasible_count))
    out.write(
      format_latex_macro_percent(
        "PercentILPGurobiInfeasibleRegions",
        ilp_infeasible_count / total_regionalized_egraphs,
      )
    )

    ilp_region_times = [
      duration_to_seconds(sample["ilp_extract_time"])
      for sample in region_points
      if (not sample["ilp_timed_out"]) and (not sample["ilp_infeasible"])
    ]
    if not ilp_region_times:
      print("WARNING: No ILP extract times available; skipping AvgILPGurobiRegionExtractTimeSecs macro")
      return
    out.write(
      format_latex_macro(
        "AvgILPGurobiRegionExtractTimeSecs",
        f"{mean(ilp_region_times):.3f}",
      )
    )

    tiger_region_times = [
      duration_to_seconds(sample["extract_time_liveon_satelliteon"])
      for sample in region_points
    ]
    if not tiger_region_times:
      print("WARNING: No tiger extract times available; skipping AvgTigerLiveOnSatelliteOnRegionExtractTimeSecs macro")
      return
    out.write(
      format_latex_macro(
        "AvgTigerLiveOnSatelliteOnRegionExtractTimeSecs",
        f"{mean(tiger_region_times):.6f}",
      )
    )
    out.write(
      format_latex_macro(
        "MaxTigerLiveOnSatelliteOnRegionExtractTimeSecs",
        f"{max(tiger_region_times):.4f}",
      )
    )

    statewalk_widths = [
      sample["statewalk_width_liveoff_satelliteoff_max"]
      for sample in region_points
    ]
    total_regions = len(statewalk_widths)

    if total_regions == 0:
      print("WARNING: No statewalk width data available; skipping statewalk width macros")
      return

    max_statewalk_width = max(statewalk_widths)
    if isinstance(max_statewalk_width, float) and max_statewalk_width.is_integer():
      max_statewalk_width = int(max_statewalk_width)
    out.write(
      format_latex_macro(
        "MaxStatewalkWidthAllBenchmarks",
        max_statewalk_width,
      )
    )
    for threshold in range(1, 31):
      out.write(
        format_latex_macro_percent(
          f"PercentRegionsStatewalkWidthUnder{threshold}",
          sum(1 for width in statewalk_widths if width < threshold) / total_regions,
        )
      )

    top_widths = sorted(set(statewalk_widths), reverse=True)[:20]
    for width_value in top_widths:
      out.write(
        format_latex_macro(
          f"NumRegionsStatewalkWidthAbove{width_value}",
          sum(1 for width in statewalk_widths if width >= width_value),
        )
      )

    heat3d_width = None
    if "heat-3d" in benchmarks:
      heat3d_row = get_row(profile, "heat-3d", "eggcc-tiger-ILP-COMPARISON")
      heat3d_timings = heat3d_row["extractRegionTimings"]
      width_key = "statewalk_width_liveon_satelliteon_max"
      values = [sample.get(width_key) for sample in heat3d_timings if sample.get(width_key) is not None]
      if values:
        heat3d_width = max(values)

    if heat3d_width is None:
      print("WARNING: No statewalk width data for heat-3d live-on satellite-on; skipping macro")
    else:
      if isinstance(heat3d_width, float) and heat3d_width.is_integer():
        heat3d_width = int(heat3d_width)
      out.write(
        format_latex_macro(
          "StatewalkWidthHeatThreeDLiveOnSatelliteOn",
          heat3d_width,
        )
      )

    geometric_speedup = compute_geometric_mean_tiger_speedup_vs_gurobi(region_points)
    out.write(
      format_latex_macro(
        "GeometricMeanTigerSpeedupVsGurobiWithTimeouts",
        f"{geometric_speedup:.2f}",
      )
    )

    encoding_geo_mean, encoding_max = compute_encoding_vars_per_egraph_stats(region_points)
    out.write(
      format_latex_macro(
        "GeometricMeanILPEncodingVarsPerEgraphSize",
        round(encoding_geo_mean),
      )
    )
    out.write(
      format_latex_macro(
        "MaxILPEncodingVarsPerEgraphSize",
        f"{encoding_max:.2f}",
      )
    )


def compute_geometric_mean_tiger_speedup_vs_gurobi(region_points):
  if not region_points:
    raise ValueError("No regionalized e-graphs provided when computing tiger speedup macro")

  ratios = []

  for sample in region_points:
    if "extract_time_liveon_satelliteon" not in sample:
      raise KeyError("Missing extract_time_liveon_satelliteon when computing tiger speedup macro")
    tiger_duration = sample["extract_time_liveon_satelliteon"]
    tiger_time = duration_to_seconds(tiger_duration)

    if "ilp_timed_out" not in sample:
      raise KeyError("Missing ilp_timed_out when computing tiger speedup macro")
    if "ilp_infeasible" not in sample:
      raise KeyError("Missing ilp_infeasible when computing tiger speedup macro")

    if sample["ilp_timed_out"] or sample["ilp_infeasible"]:
      gurobi_time = ILP_TIMEOUT_SECONDS
    else:
      if "ilp_extract_time" not in sample:
        raise KeyError("Missing ilp_extract_time when computing tiger speedup macro")
      gurobi_duration = sample["ilp_extract_time"]
      gurobi_time = ILP_TIMEOUT_SECONDS if gurobi_duration is None else duration_to_seconds(gurobi_duration)

    if tiger_time <= 0:
      raise ValueError("Non-positive tiger time encountered when computing tiger speedup macro")
    if gurobi_time <= 0:
      gurobi_time = ILP_TIMEOUT_SECONDS

    ratio = gurobi_time / tiger_time
    if ratio <= 0:
      raise ValueError("Non-positive speedup ratio encountered when computing tiger speedup macro")

    ratios.append(ratio)

  if not ratios:
    raise ValueError("No valid tiger/Gurobi timing pairs for speedup macro")

  return geometric_mean(ratios)


def compute_encoding_vars_per_egraph_stats(region_points):
  if not region_points:
    raise ValueError("No regionalized e-graphs provided when computing encoding size ratios")

  ratios = []

  for sample in region_points:
    num_vars = sample["ilp_encoding_num_vars"]
    egraph_size = sample["egraph_size"]

    if num_vars is None or num_vars <= 0:
      raise ValueError("Non-positive ilp_encoding_num_vars encountered when computing encoding size ratios")
    if egraph_size is None or egraph_size <= 0:
      raise ValueError("Non-positive egraph_size encountered when computing encoding size ratios")

    ratio = num_vars / egraph_size
    if ratio <= 0:
      raise ValueError("Non-positive encoding ratio encountered when computing encoding size ratios")

    ratios.append(ratio)

  if not ratios:
    raise ValueError("No ILP encoding variable data available when computing encoding size ratios")

  return geometric_mean(ratios), max(ratios)

