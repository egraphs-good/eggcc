# Reproducing the OOPSLA camera-ready figures

This directory pins the data and the recipe for re-rendering the figures in the
OOPSLA paper. It exists because regenerating them is **not** just "run the
nightly": the paper's figures come from two runs, they were rendered by two
different matplotlib versions, and the treatment naming has changed since.

## Data

| File | What it is |
|---|---|
| `eggcc-submission-combined.json` | The submission run. Same bytes as the file in the `eggcc-submission-pldi-2026-data` tag. Source for every figure except the Fenwick chart. |
| `fenwick-mar17-profile.json` | A later Fenwick-only run (19 rows, `fenwick_tree`). Source for the Fenwick chart **only**. |

The Fenwick chart needs the second file because the submission run does not
contain `eggcc-tiger-WITHCTX-O0-O0` / `eggcc-tiger-nohacker-WITHCTX-O0-O0` for
`fenwick_tree` at the values the paper reports. The Mar 17 numbers reproduce all
four Fenwick speedup macros in the paper exactly (6.52, 23.01, 40.74, 40.71).

A dedicated ILP-comparison run also exists locally
(`nightlyjustILPComparisonno13-2025`), but it is **not** needed: its 96
`eggcc-tiger-ILP-COMPARISON` rows are byte-identical to the ones already in
`eggcc-submission-combined.json`.

## The treatment-name trap

Submission-era runs carry Statewalk DP as its own treatment,
`eggcc-tiger-O0-O0`. Current code defaults to `eggcc-O0-O0`. **Both names exist
in the old data with different numbers**, so plotting an old profile with the
default silently charts the wrong rows instead of failing. The tell is the
bril under3/over3 split: the paper's is 60/5, the wrong treatment gives 62/3.

`graphs.py` now reads `EGGCC_DP_TREATMENT` (default `eggcc-O0-O0`, so normal
nightly runs are unaffected). `simple_table.py` already had `--dp-treatment`.

## Recipe

Set `pdf.fonttype = 42` for every figure — the paper must not ship Type 3 fonts.
This is provably geometry-neutral: rendering the same figure with fonttype 3 vs
42 gives byte-identical path geometry.

    export EGGCC_DP_TREATMENT=eggcc-tiger-O0-O0

Use the matplotlib version that matches the figure's PDF `Producer` string:

| matplotlib | Figures |
|---|---|
| **3.9.2** | `ilp-encoding-size-scatter`, `peggy-extraction-time-ratio`, `normalized-binary-perf-chart-over3-bril`, `statewalk-width-histogram` |
| **3.11.1** | `extraction-time-cdf`, `statewalk-width-vs-tiger-time-{optoff,opton}`, `statewalk-width-vs-ilp-time`, `normalized-binary-perf-chart-{under3-bril,polybench}`, `eggcc-extraction-time-ratio`, `fenwick-cycles-bar-chart` |

Driver: call `graphs.make_graphs(out, graphs_dir, profile, "benchmarks/passing",
NightlyConfig(paper_mode=True, use_gurobi=True))` with `matplotlib.use("Agg")`
and `rcParams["pdf.fonttype"]=42`, from the repo root so `./infra/peggy_data.csv`
resolves. Render the Fenwick chart separately against
`fenwick-mar17-profile.json`.

## Verifying a regeneration

Do not compare PDFs byte-wise, and do not compare drawing ops positionally — a
single inserted op makes every later op look changed and produces a meaningless
"hundreds differ" count.

Compare **markers**: extract single-point path ops with `mutool draw -F trace`,
group by colour, sort, and compare the y-coordinates. y encodes the measured
value; x encodes layout. Every figure above reproduces with **max |Δy| = 0.000pt**
except `eggcc-extraction-time-ratio` (0.100pt, sub-point layout drift).

Expect x to differ by up to ~1.33pt on a 720pt page (≤0.19%). That drift is
environmental, not caused by the font change, and is invisible in print.

## Note on `statewalk-width-vs-ilp-time.pdf`

The generator writes this name in lowercase; an older copy in the paper repo was
`statewalk-width-vs-ILP-time.pdf` while the LaTeX asked for lowercase. That built
only because TeX Live defaults to `texmf_casefold_search=1`. The paper now uses
the lowercase name the generator produces.
