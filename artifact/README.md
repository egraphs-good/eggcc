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
claim-by-claim guide (VM notes, optional Gurobi, paper-scale runs).

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
| **RQ1** | Statewalk DP is far faster than ILP extraction (paper: **520× vs Gurobi**, with ILP routinely hitting the extraction timeout) | Fig. 7 | `extraction-time-cdf.pdf` |
| **RQ2** | eggcc's extracted code is comparable to LLVM (and to ILP-extracted code) | Fig. 10 (Bril), Fig. 11 (PolyBench) | `normalized-binary-perf-chart-{under3,over3}-bril.pdf`, `-polybench.pdf` |
| **RQ3** | Statewalk DP enables domain-specific effectful optimizations (Hacker's-Delight `lowbit` on a Fenwick tree: **6.52× vs LLVM-O3-O3**) | §8.3 | `fenwick-cycles-bar-chart.pdf` |

The artifact uses the free **CBC** ILP solver out of the box — no license needed. The
paper's headline 520× number is specifically **vs Gurobi**, which is commercial; installing
a Gurobi license (§2) reproduces that comparison. Without one, CBC already shows ILP's
infeasibility (CBC also times out on every PolyBench benchmark).

## The VM

- **Host:** an ARM machine (Apple-Silicon Mac or Windows on ARM) with VirtualBox 7.2+ (7.2 is
  the first to run VMs on ARM). This is an arm64 VM — it won't import on Intel/x86 hosts. If you
  haven't imported the VM yet, see the host-side setup guide shipped next to `eggcc-artifact.ova`.
- **Guest:** Ubuntu 24.04 Desktop, 4 vCPUs / 8 GB RAM. Login `artifact` / `artifact`.
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

- `extraction-time-cdf.pdf` — **RQ1**, paper Fig. 7. Confirm the shape: the Statewalk DP
  curve sits at the far left (sub-millisecond) while the CBC/ILP curve is orders of magnitude
  to the right, with a run of points pinned at the timeout (ILP failing to extract in time).
- `normalized-binary-perf-chart-{under3,over3}-bril.pdf` — **RQ2**, paper Fig. 10 (split in
  two by the 5 outlier benchmarks, as in the paper).
- `normalized-binary-perf-chart-polybench.pdf` — **RQ2**, paper Fig. 11.
- `fenwick-cycles-bar-chart.pdf` — **RQ3**, the Hacker's-Delight Fenwick-tree case study.

These land in your home folder, next to the authors' pre-generated CBC run in `~/reference/`,
so open the two side by side to compare — e.g. `~/extraction-time-cdf.pdf` against
`~/reference/extraction-time-cdf.pdf`. The shapes should match (see §3); exact numbers will
not, since both are 1 %-of-regions samples with their own random draw. (If you ran with
Gurobi, compare against `~/reference/gurobi/` instead — see below.)

(The other paper figures — ILP encoding size (Fig. 6), statewalk-width distribution
(Fig. 8), and runtime-vs-statewalk-width scaling (Fig. 9) — are also written to
`eggcc/nightly/output/paper/`.)

Exact times vary with the host and the random region sample; the order-of-magnitude gap is
stable. ILP timeouts are expected, not failures.

**Optional — Gurobi (the paper's solver).** The 520× headline is specifically vs Gurobi,
which is commercial but **free for academics**. The Gurobi solver is already installed in the
VM — you only need to add a license, and you do it with a one-line command (no browser needed
in the VM):

1. **Get a license key.** On your *host* machine's browser, sign in at
   <https://portal.gurobi.com/>, request a free **academic** license, and copy the
   `grbgetkey <KEY>` command it shows you.
2. **Activate it in the VM.** To avoid retyping the key, SSH into the VM from your host
   terminal (where paste works — the VM's own clipboard is unreliable) and run it there:
   ```bash
   ssh -p 2222 artifact@localhost      # from your HOST terminal; password: artifact
   grbgetkey <YOUR-KEY>                 # paste it in this ssh session; writes ~/gurobi.lic
   ```
   (Or just type `grbgetkey <YOUR-KEY>` directly in the VM's own terminal.)
3. **Verify and run:**
   ```bash
   eggcc/infra/setup_gurobi.sh
   ./reproduce.sh full
   ```

`full` auto-detects the license and adds a **Gurobi** curve to the CDF (Statewalk DP vs Gurobi
vs CBC) and switches the RQ2 charts' ILP treatment to Gurobi (EQCC-GUROBI), matching the
paper. Compare this run against the authors' Gurobi set in `~/reference/gurobi/` (the default
`~/reference/` is the CBC run). The Gurobi run also produces a couple of extra figures the CBC
run doesn't (e.g. `egraph-size-vs-ILP-time.pdf`).

**If your license is a `gurobi.lic` file instead of a key** (e.g. a WLS license): copy it into
the VM from your host with `scp -P 2222 gurobi.lic artifact@localhost:~/`, then run
`eggcc/infra/setup_gurobi.sh ~/gurobi.lic`. (This VM has no web browser, by design — the
desktop ships without one; that's why the key/file comes in over SSH.)

If `grbgetkey` fails to validate (academic licenses check your network), see
<https://support.gurobi.com/hc/en-us/articles/360040113232>.

**Paper-scale runs.** For the paper's exact configuration (100 % of regions, 5-minute
timeout) you need a large machine: `cd eggcc && bash infra/nightly.sh benchmarks/passing
--local --paper` (~100 cores; many hours).

## 3. Reproducibility

| Claim | Reproduce with | Figure — expected result |
|-------|----------------|--------------------------|
| RQ1: Statewalk DP ≫ faster than ILP | `./reproduce.sh full` | `extraction-time-cdf.pdf` — Statewalk DP curve far left of the CBC/ILP curve; ILP hits the timeout |
| RQ1 vs Gurobi (paper's 520×) | install Gurobi, `./reproduce.sh full` | CDF gains a Gurobi curve, also far right of Statewalk DP (compare against `~/reference/gurobi/`) |
| RQ2: output at par with ILP, comparable to LLVM | `./reproduce.sh full` | `normalized-binary-perf-chart-*.pdf` — EQCC-DP is close to the LLVM-O3-O0 baseline (1.0) and tracks the ILP-extracted bar (CBC by default; EQCC-GUROBI with a license) |
| RQ3: domain-specific optimization | `./reproduce.sh full` | `fenwick-cycles-bar-chart.pdf` — EQCC-DP (with Hacker's-Delight rules) well below the LLVM and no-hacker bars |

A run is valid if it completes and writes the PDFs; a crash/panic or an empty figure is a
bug. Each expected result above is also shipped as a graph in `~/reference/`, so you can
compare your freshly generated figure against the authors' side by side.

## 4. Layout

```
~/
├── README.md                     this guide (quick start at the top; = eggcc/artifact/README.md)
├── reproduce.sh                  smoke | full   →   your figures land here
├── extraction-time-cdf.pdf, normalized-binary-perf-chart-*.pdf, fenwick-cycles-bar-chart.pdf
│                                 (YOUR figures — appear here after you run reproduce.sh)
├── reference/                    the authors' pre-generated CBC run (matches the default run)
│   └── gurobi/                   the authors' Gurobi run (matches a Gurobi run; a few extra figures)
└── eggcc/                        source
    ├── artifact/                 reproduce.sh, README.md (this file)
    ├── infra/                    nightly.py, graphs.py, plot_cdf.py, setup_gurobi.sh,
    │                             provision.sh, BUILDING.md
    ├── dag_in_context/           egglog pipeline + Statewalk DP extractor (src/tiger/)
    └── benchmarks/passing/       benchmark suite (Bril, PolyBench, raytracer, Fenwick)
```
(Toolchain dirs — `.cargo`, `.rustup`, `.eggcc-venv` — are hidden.)

## Rebuilding the VM

Authors build the VM by hand on an Apple-Silicon Mac (VirtualBox 7.2+, arm64 Ubuntu 24.04).
See **`infra/BUILDING.md`** for the full build/ship steps.
