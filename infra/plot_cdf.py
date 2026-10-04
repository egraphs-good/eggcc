#!/usr/bin/env python3
"""Generate only the ILP-vs-tiger extraction-time CDF from a profile.json.

Used by artifact/reproduce.sh for a fast reproduction that works on any subset of
benchmarks: it plots just the CDF and so does not depend on the full nightly graph
set (some of which require specific benchmarks, e.g. fenwick_tree).

The CDF compares the tiger (greedy statewalk-DP) extraction time against the ILP
solver times recorded by the eggcc-tiger-ILP-COMPARISON treatment (CBC, plus Gurobi
when a Gurobi run is present in the data).
"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from graphs import make_extraction_time_cdf
from graph_helpers import set_ilp_timeout_seconds, has_gurobi_ilp_data


def main():
    ap = argparse.ArgumentParser(description="Plot the ILP-vs-tiger extraction-time CDF.")
    ap.add_argument("profile_json", help="profile.json produced by the profiler")
    ap.add_argument("output_pdf", help="where to write the CDF pdf")
    ap.add_argument(
        "--ilp-timeout-seconds",
        type=int,
        default=30,
        help="per-region ILP timeout the data was generated with (used for the legend)",
    )
    args = ap.parse_args()

    set_ilp_timeout_seconds(args.ilp_timeout_seconds)
    with open(args.profile_json) as f:
        data = json.load(f)
    os.makedirs(os.path.dirname(os.path.abspath(args.output_pdf)), exist_ok=True)
    make_extraction_time_cdf(
        data,
        args.output_pdf,
        use_log_x=True,
        use_exp_y=False,
        include_gurobi=has_gurobi_ilp_data(data),
    )
    print(f"Wrote CDF to {args.output_pdf}")


if __name__ == "__main__":
    main()
