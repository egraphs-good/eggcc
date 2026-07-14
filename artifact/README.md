# eggcc Artifact — "Efficient Extraction for Effectful E-Graphs"

## Quick start

In the VM, open a terminal and run:

```bash
./reproduce.sh smoke     # ~5–10 min : sanity check (3 benchmarks)
./reproduce.sh full      # ~3–4 h    : the whole benchmark suite
```

Your figures land in this home folder; the authors' pre-generated versions are in
`~/reference/` to compare against. The authors' figures are already there, so you can open
`~/reference/` right now to see the expected results. The rest of this file is the full,
claim-by-claim guide (VM notes, optional Gurobi, paper-scale runs, reusability).

---

This artifact supports the paper *Efficient Extraction for Effectful E-Graphs*. The paper
introduces **Statewalk DP**, an extraction algorithm for effectful e-graphs that enforces
effect ordering **without an external ILP solver**. It is implemented in **eggcc**, an
e-graph-based compiler for (imperative) Bril programs. (In the source, the Statewalk DP
extractor is the `tiger` component.)

The evaluation (paper §8) answers three research questions, each backed by a figure this
artifact reproduces:

| RQ | Claim | Figure (paper) | Artifact figure |
|----|-------|----------------|-----------------|
| **RQ1** | Statewalk DP is far faster than ILP extraction (paper: **520× vs Gurobi**; ILP times out on all 30 PolyBench and 17 Bril benchmarks) | Fig. 7 | `extraction-time-cdf.pdf` |
| **RQ2** | eggcc's extracted code is comparable to LLVM (and to ILP-extracted code) | Fig. 10 (Bril), Fig. 11 (PolyBench) | `normalized-binary-perf-chart-{under3,over3}-bril.pdf`, `-polybench.pdf` |
| **RQ3** | Statewalk DP enables domain-specific effectful optimizations (Hacker's-Delight `lowbit` on a Fenwick tree: **6.52× vs LLVM-O3-O3**) | §8.3 | `fenwick-cycles-bar-chart.pdf` |

The artifact uses the free **CBC** ILP solver out of the box — no license needed. The
paper's headline 520× number is specifically **vs Gurobi**, which is commercial; installing
a Gurobi license (§2) reproduces that comparison. Without one, CBC already shows ILP's
infeasibility (CBC also times out on every PolyBench benchmark).

## The VM

- **Host:** VirtualBox 7.2+ (free; macOS/ARM and Windows/ARM supported). If you haven't
  imported the VM yet, see the host-side setup guide shipped next to `eggcc-artifact.ova`.
- **Guest:** Ubuntu 22.04 Desktop, 4 vCPUs / 8 GB RAM. Login `eggcc` / `eggcc`.
- **In your home folder** you'll find `README.md` (this guide, quick start at the top),
  `reproduce.sh`, the source in `eggcc/`, and `reference/` — the authors' pre-generated
  figures, ready to open right away. Running `reproduce.sh` writes *your* figures into the
  home folder next to `reference/`, so you can compare against it (see below).

> **Scale.** The paper ran on a 100+-core AMD EPYC (512 GB), timing **100 % of regions**
> with a **5-minute** per-region ILP timeout. That is not feasible on a small VM, so the
> VM's `full` run samples **1 % of regions** with a **30-second** timeout. This reproduces
> the *shape* of every claim (the order-of-magnitude gap, ILP timing out, the speedups) but
> not the exact paper numbers; the figures state the timeout they used. §2 explains how to
> run the full paper configuration on a large machine.

## 1. Sanity check (~5–10 min)

Open a terminal and run:

```bash
./reproduce.sh smoke
```

This profiles three small benchmarks with CBC and writes `~/extraction-time-cdf-smoke.pdf`;
it opens automatically. **Statewalk DP** is bunched at the far left and **CBC** far to the
right — Statewalk DP is dramatically faster. Three benchmarks make a coarse plot; the full
version ships in `~/reference/extraction-time-cdf.pdf`.

## 2. Full reproduction (~3–4 h)

```bash
./reproduce.sh full
```

Runs the whole benchmark suite (64 Bril + 30 PolyBench + a raytracer, as in the paper) and
writes these figures into your home folder:

- `extraction-time-cdf.pdf` — **RQ1**, paper Fig. 7. Statewalk DP is sub-millisecond;
  CBC/ILP is orders of magnitude slower with a run of points at the timeout.
- `normalized-binary-perf-chart-{under3,over3}-bril.pdf` — **RQ2**, paper Fig. 10 (split in
  two by the 5 outlier benchmarks, as in the paper).
- `normalized-binary-perf-chart-polybench.pdf` — **RQ2**, paper Fig. 11.
- `fenwick-cycles-bar-chart.pdf` — **RQ3**, the Hacker's-Delight Fenwick-tree case study.

These land in your home folder, next to the authors' pre-generated set in `~/reference/`, so
open the two side by side to compare — e.g. `~/extraction-time-cdf.pdf` against
`~/reference/extraction-time-cdf.pdf`. The shapes should match (see §3); exact numbers will
not, since both are 1 %-of-regions samples with their own random draw.

(The other paper figures — ILP encoding size (Fig. 6), statewalk-width distribution
(Fig. 8), and runtime-vs-statewalk-width scaling (Fig. 9) — are also written to
`eggcc/nightly/output/paper/`.)

Exact times vary with the host and the random region sample; the order-of-magnitude gap is
stable. ILP timeouts are expected, not failures.

**Optional — Gurobi (the paper's solver).** Install a (free academic) license, then re-run:

```bash
eggcc/infra/setup_gurobi.sh /path/to/gurobi.lic
./reproduce.sh full
```

`full` auto-detects the license and adds a **Gurobi** curve to the CDF (Statewalk DP vs
Gurobi vs CBC) and switches the RQ2 charts' ILP treatment to Gurobi (EQCC-GUROBI), matching
the paper. It stays VM-scale, so it runs on a few cores.

**Paper-scale runs.** For the paper's exact configuration (100 % of regions, 5-minute
timeout) you need a large machine: `cd eggcc && bash infra/nightly.sh benchmarks/passing
--local --paper` (~100 cores; many hours).

## 3. Reproducibility

| Claim | Reproduce with | Figure — expected result |
|-------|----------------|--------------------------|
| RQ1: Statewalk DP ≫ faster than ILP | `./reproduce.sh full` | `extraction-time-cdf.pdf` — Statewalk DP curve far left of the CBC/ILP curve; ILP hits the timeout |
| RQ1 vs Gurobi (paper's 520×) | install Gurobi, `./reproduce.sh full` | CDF gains a Gurobi curve, also far right of Statewalk DP |
| RQ2: output at par with ILP, comparable to LLVM | `./reproduce.sh full` | `normalized-binary-perf-chart-*.pdf` — EQCC-DP tracks EQCC-GUROBI and is close to the LLVM-O3-O0 baseline (1.0) |
| RQ3: domain-specific optimization | `./reproduce.sh full` | `fenwick-cycles-bar-chart.pdf` — EQCC-DP (with Hacker's-Delight rules) well below the LLVM and no-hacker bars |

A run is valid if it completes and writes the PDFs; a crash/panic or an empty figure is a
bug. Each expected result above is also shipped as a graph in `~/reference/`, so you can
compare your freshly generated figure against the authors' side by side.

## 4. Reusability

- **Run on your own program:** `cd eggcc && cargo run --release -- <file.bril> --run-mode
  optimize` (`--tiger-ilp` uses the ILP extractor instead of Statewalk DP, `--ilp-solver
  cbc|gurobi`, `--ilp-timeout-seconds N`).
- **Add benchmarks:** put `.bril`/`.rs` files under `eggcc/benchmarks/passing/<suite>/`.
- **Key code:** the Statewalk DP extractor is `eggcc/dag_in_context/src/tiger/`; the
  compiler pipeline is `eggcc/dag_in_context/` and `eggcc/src/`; the measurement/plotting
  harness is `eggcc/infra/`.
- **License:** open source (`eggcc/LICENSE`). **Requires LLVM 18.**

## 5. Layout

```
~/
├── README.md                     this guide (quick start at the top; = eggcc/artifact/README.md)
├── reproduce.sh                  smoke | full   →   your figures land here
├── extraction-time-cdf.pdf, normalized-binary-perf-chart-*.pdf, fenwick-cycles-bar-chart.pdf
│                                 (YOUR figures — appear here after you run reproduce.sh)
├── reference/                    the authors' pre-generated figures, to compare against
└── eggcc/                        source
    ├── artifact/                 reproduce.sh, README.md (this file)
    ├── infra/                    nightly.py, graphs.py, plot_cdf.py, setup_gurobi.sh,
    │                             build_vm.sh, provision.sh, BUILDING.md
    ├── dag_in_context/           egglog pipeline + Statewalk DP extractor (src/tiger/)
    └── benchmarks/passing/       benchmark suite (Bril, PolyBench, raytracer, Fenwick)
```
(Toolchain dirs — `.cargo`, `.rustup`, `.eggcc-venv` — are hidden.)

## Rebuilding the VM

Authors build and ship the VM with `infra/build_vm.sh` on a macOS host (VirtualBox 7.2+).
See **`infra/BUILDING.md`** for the full build/ship steps and the manual fallback.
