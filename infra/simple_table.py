import profile
import os
import math
from graph_helpers import *
from generate_line_counts import stddev_cycles


def summarize_row(row):
    mean_cycles = mean(row["cycles"])
    mean_ms = str(cycles_to_ms(mean_cycles)) + " ms"
    std_dev = str(cycles_to_ms(stddev_cycles(row["cycles"])))
    return mean_ms + " +- " + str(std_dev)


def make_compact_data(data):
    res = []
    methods = ["eggcc-tiger-O0-O0", "llvm-O0-O0", "llvm-O3-O0"]
    benchmarks = dedup([b.get('benchmark') for b in data])
    header = [""] + methods
    res = res + [header]
    for benchmark in benchmarks:
        row = [benchmark]
        for method in methods:
            row = row + [summarize_row(get_row(data, benchmark, method))]
        res = res + [row]
    return res

    

def raytrace_total_region_extract_time(data):
    res = 0.0
    row = get_row(data, "raytrace", "eggcc-tiger-ILP-COMPARISON")
    timings = row["extractRegionTimings"]
    for timing in timings:
        res += duration_to_seconds(timing["extract_time_liveon_satelliteon"])
    print("here it is")
    print(res)