import profile
import os
import math
from graph_helpers import *
from generate_line_counts import stddev_cycles


_LATEX_ESCAPES = {
    "\\": r"\textbackslash{}",
    "&": r"\&",
    "%": r"\%",
    "$": r"\$",
    "#": r"\#",
    "_": r"\_",
    "{": r"\{",
    "}": r"\}",
    "~": r"\textasciitilde{}",
    "^": r"\textasciicircum{}",
}


def latex_escape(s):
    s = str(s)
    out = []
    for ch in s:
        out.append(_LATEX_ESCAPES.get(ch, ch))
    return "".join(out)


def to_latex(table, caption=None, label=None, longtable=True, align=None):
    """Render a 2D list (header row + data rows) as a LaTeX table.

    `+-` in cells is rendered as $\\pm$. Cells are otherwise escaped.
    With longtable=True the body uses the `longtable` package so it pages.
    """
    if not table:
        return ""
    ncols = len(table[0])
    col_spec = align if align is not None else ("l" + "r" * (ncols - 1))

    def render_cell(c):
        s = str(c)
        # Render "X +- Y" as math with \pm so it typesets nicely.
        if " +- " in s:
            left, right = s.split(" +- ", 1)
            return f"{latex_escape(left)} $\\pm$ {latex_escape(right)}"
        return latex_escape(s)

    header = " & ".join(render_cell(c) for c in table[0]) + r" \\"
    body_rows = [
        " & ".join(render_cell(c) for c in row) + r" \\"
        for row in table[1:]
    ]

    lines = []
    if longtable:
        lines.append(r"\begin{longtable}{" + col_spec + "}")
        if caption is not None:
            cap = r"\caption{" + latex_escape(caption) + "}"
            if label is not None:
                cap += r"\label{" + label + "}"
            lines.append(cap + r" \\")
        lines.append(r"\toprule")
        lines.append(header)
        lines.append(r"\midrule")
        lines.append(r"\endfirsthead")
        lines.append(r"\toprule")
        lines.append(header)
        lines.append(r"\midrule")
        lines.append(r"\endhead")
        lines.append(r"\midrule \multicolumn{" + str(ncols) + r"}{r}{\textit{continued on next page}} \\")
        lines.append(r"\endfoot")
        lines.append(r"\bottomrule")
        lines.append(r"\endlastfoot")
        lines.extend(body_rows)
        lines.append(r"\end{longtable}")
    else:
        lines.append(r"\begin{table}[h]")
        lines.append(r"\centering")
        lines.append(r"\begin{tabular}{" + col_spec + "}")
        lines.append(r"\toprule")
        lines.append(header)
        lines.append(r"\midrule")
        lines.extend(body_rows)
        lines.append(r"\bottomrule")
        lines.append(r"\end{tabular}")
        if caption is not None:
            cap = r"\caption{" + latex_escape(caption) + "}"
            if label is not None:
                cap += r"\label{" + label + "}"
            lines.append(cap)
        lines.append(r"\end{table}")
    return "\n".join(lines)


def summarize_row(row):
    mean_cycles = mean(row["cycles"])
    mean_ms = str(cycles_to_ms(mean_cycles)) + " ms"
    std_dev = str(cycles_to_ms(stddev_cycles(row["cycles"])))
    return mean_ms + " +- " + str(std_dev)


def make_compact_data(data):
    res = []
    methods = ["llvm-O0-O0", "eggcc-tiger-O0-O0", "llvm-O3-O0"]
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