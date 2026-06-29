import os
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

    `caption` is escaped, but `label` is emitted verbatim into `\\label{...}`,
    so it must be a bare LaTeX identifier (no special characters).
    """
    if not table:
        return ""
    ncols = len(table[0])
    col_spec = align if align is not None else ("l" + "r" * (ncols - 1))

    def render_unit_suffix(s):
        # Convert a trailing " us" unit to LaTeX "$\mu$s"; pass " ms" through.
        if s.endswith(" us"):
            return f"{latex_escape(s[:-3])} $\\mu$s"
        return latex_escape(s)

    def render_cell(c):
        s = str(c)
        # Render "X +- Y" as math with \pm so it typesets nicely.
        if " +- " in s:
            left, right = s.split(" +- ", 1)
            return f"{render_unit_suffix(left)} $\\pm$ {render_unit_suffix(right)}"
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
    return format_cycles_with_stddev(mean(row["cycles"]), stddev_cycles(row["cycles"]))


COMPACT_METHODS = ["llvm-O0-O0", "eggcc-tiger-O0-O0", "llvm-O3-O0"]


def _make_compact_data_for_benchmarks(data, benchmarks):
    from graphs import to_paper_names_treatment
    header = [""] + [to_paper_names_treatment(m) for m in COMPACT_METHODS]
    res = [header]
    for benchmark in benchmarks:
        row = [benchmark]
        for method in COMPACT_METHODS:
            row.append(summarize_row(get_row(data, benchmark, method)))
        res.append(row)
    return res


def suites_in(data):
    return dedup([b.get('suite') for b in data])


def make_compact_data_for_suite(data, suite):
    suite_data = [b for b in data if b.get('suite') == suite]
    benchmarks = dedup([b['benchmark'] for b in suite_data])
    return _make_compact_data_for_benchmarks(suite_data, benchmarks)


def write_compact_table_latex(data, paper_dir):
    """Write one LaTeX fragment per suite into `paper_dir`.

    Each suite gets its own `longtable` fragment at
    `<paper_dir>/compact_table_<suite>.tex` (no caption, no preamble), so it
    can be `\\input{compact_table_<suite>}` from the paper. The paper's
    preamble must load `booktabs` and `longtable`.
    """
    paper_dir = str(paper_dir)
    os.makedirs(paper_dir, exist_ok=True)
    out_paths = []
    for suite in suites_in(data):
        table = make_compact_data_for_suite(data, suite)
        tex = to_latex(table, caption=None, label=None, longtable=True)
        out_path = os.path.join(paper_dir, f"compact_table_{suite}.tex")
        with open(out_path, "w") as f:
            f.write(tex + "\n")
        print(f"Wrote compact table LaTeX for suite '{suite}' to {out_path}")
        out_paths.append(out_path)
    return out_paths