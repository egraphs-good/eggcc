#!/usr/bin/env python3
import json
import os
from collections import defaultdict

def _build_benchtable(profiles):
    keys = ["name", "mean", "max", "min", "stddev"]

    sizing = "|".join(list(map(lambda k: "p{4.5cm}", keys)))
    header = " & ".join(keys)
    
    preamble = """\\begin{tabular}{ |%s| }
\hline
%s \\\\
\hline
""" % (sizing, header)
    for profile in sorted(profiles, key=lambda p: p["runMethod"]):
        escaped_name = profile['runMethod'].replace('_', '\\_')
        preamble += f"{escaped_name} & "\
                    + " & ".join(list(map(lambda k: "{:.3f}".format(profile["hyperfine"]["results"][0][k]), keys[1:]))) + "\\\\\n"
    preamble += "\end{tabular}\n"
    return preamble

def gen_nightly_table(nightly_data):
    preamble = """\\begin{tabular}{ |p{4cm}|p{20cm}| }
\hline
\multicolumn{2}{|c|}{Benchmarks} \\\\
\hline
Name & Executions\\\\
\hline"""

    benches = defaultdict(list)

    for profile in nightly_data:
        benches[profile["benchmark"]].append(profile)

    for profile_key in sorted(benches.keys())[:1]:
        preamble += ("\hline\n%s & %s \\\\\n" % (profile_key, _build_benchtable(benches[profile_key])))
    preamble += "\\end{tabular}"
    return preamble
