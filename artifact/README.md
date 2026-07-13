# eggcc Artifact — "Efficient Extraction for Effectful E-Graphs"

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

## The image

The artifact is a **Docker image** (`eggcc-artifact`), built from Ubuntu 22.04 with eggcc and
all its dependencies. Load the shipped tarball or pull it from the registry named in the
submission:

```bash
docker load < eggcc-artifact-image.tar.gz          # offline tarball, or:
docker pull <registry>/eggcc-artifact:oopsla26     # registry reference
```

- **Architecture.** The reference image is `linux/amd64` (the paper's architecture). It runs
  natively on x86-64 hosts and on Apple-Silicon / ARM hosts through Docker's emulation. (If a
  multi-arch image was published, ARM hosts run the `arm64` variant natively.)
- **Headless.** The container has no desktop; each run writes its figures to a directory you
  bind-mount at `/out`, which you open with your host's normal PDF viewer.

> **Scale.** The paper ran on a 100+-core AMD EPYC (512 GB), timing **100 % of regions** with
> a **5-minute** per-region ILP timeout. That is not feasible on a laptop, so the `full` run
> here samples **1 % of regions** with a **30-second** timeout. This reproduces the *shape* of
> every claim (the order-of-magnitude gap, ILP timing out, the speedups) but not the exact
> paper numbers; the figures state the timeout they used. §2 explains how to run the full
> paper configuration on a large machine.

## 1. Sanity check (~5–10 min)

```bash
mkdir -p out
docker run --rm -it -v "$PWD/out:/out" eggcc-artifact ./reproduce.sh smoke
```

This profiles three small benchmarks with CBC and writes `out/extraction-time-cdf-smoke.pdf`
(a separate file, so the full figures aren't affected). Open it from `./out` on your host:
**Statewalk DP** is bunched at the far left and **CBC** far to the right — Statewalk DP is
dramatically faster. Three benchmarks make a coarse plot; `full` (below) produces the real one.

## 2. Full reproduction (~3–4 h)

```bash
docker run --rm -it -v "$PWD/out:/out" eggcc-artifact ./reproduce.sh full
```

Runs the whole benchmark suite (64 Bril + 30 PolyBench + a raytracer, as in the paper) and
writes these figures to `./out`:

- `extraction-time-cdf.pdf` — **RQ1**, paper Fig. 7. Statewalk DP is sub-millisecond;
  CBC/ILP is orders of magnitude slower with a run of points at the timeout.
- `normalized-binary-perf-chart-{under3,over3}-bril.pdf` — **RQ2**, paper Fig. 10 (split in
  two by the 5 outlier benchmarks, as in the paper).
- `normalized-binary-perf-chart-polybench.pdf` — **RQ2**, paper Fig. 11.
- `fenwick-cycles-bar-chart.pdf` — **RQ3**, the Hacker's-Delight Fenwick-tree case study.

(The other paper figures — ILP encoding size (Fig. 6), statewalk-width distribution
(Fig. 8), and runtime-vs-statewalk-width scaling (Fig. 9) — are also written to `./out`.)

Exact times vary with the host and the random region sample; the order-of-magnitude gap is
stable. ILP timeouts are expected, not failures.

**Optional — Gurobi (the paper's solver).** With a (free academic) license, mount it and
re-run `full`:

```bash
docker run --rm -it -v "$PWD/out:/out" -v "$PWD/gurobi.lic:/root/gurobi.lic" \
  eggcc-artifact bash -c "infra/setup_gurobi.sh /root/gurobi.lic && ./reproduce.sh full"
```

`full` auto-detects the license and adds a **Gurobi** curve to the CDF (Statewalk DP vs
Gurobi vs CBC) and switches the RQ2 charts' ILP treatment to Gurobi (EQCC-GUROBI), matching
the paper. It stays at the sampled scale, so it runs on a few cores.

**Paper-scale runs.** For the paper's exact configuration (100 % of regions, 5-minute
timeout) you need a large machine:

```bash
docker run --rm -it -v "$PWD/out:/out" -w /root/eggcc eggcc-artifact \
  bash -c "bash infra/nightly.sh benchmarks/passing --local --paper"   # ~100 cores; many hours
```

## 3. Reproducibility

Each command is a `docker run ... ./reproduce.sh full` as in §1–2.

| Claim | Reproduce with | Figure — expected result |
|-------|----------------|--------------------------|
| RQ1: Statewalk DP ≫ faster than ILP | `reproduce.sh full` | `extraction-time-cdf.pdf` — Statewalk DP curve far left of the CBC/ILP curve; ILP hits the timeout |
| RQ1 vs Gurobi (paper's 520×) | install Gurobi, `reproduce.sh full` | CDF gains a Gurobi curve, also far right of Statewalk DP |
| RQ2: output at par with ILP, comparable to LLVM | `reproduce.sh full` | `normalized-binary-perf-chart-*.pdf` — EQCC-DP tracks EQCC-GUROBI and is close to the LLVM-O3-O0 baseline (1.0) |
| RQ3: domain-specific optimization | `reproduce.sh full` | `fenwick-cycles-bar-chart.pdf` — EQCC-DP (with Hacker's-Delight rules) well below the LLVM and no-hacker bars |

A run is valid if it completes and writes the PDFs; a crash/panic or an empty figure is a
bug.

## 4. Reusability

- **Run on your own program:** mount a directory holding your Bril file and invoke eggcc
  inside the container:
  ```bash
  docker run --rm -it -v "$PWD:/work" -w /root/eggcc eggcc-artifact \
    cargo run --release -- /work/<file.bril> --run-mode optimize
  ```
  (`--tiger-ilp` uses the ILP extractor instead of Statewalk DP; `--ilp-solver cbc|gurobi`;
  `--ilp-timeout-seconds N`.)
- **Add benchmarks:** put `.bril`/`.rs` files under `eggcc/benchmarks/passing/<suite>/`
  (`/root/eggcc/benchmarks/passing/<suite>/` in the image).
- **Key code:** the Statewalk DP extractor is `eggcc/dag_in_context/src/tiger/`; the
  compiler pipeline is `eggcc/dag_in_context/` and `eggcc/src/`; the measurement/plotting
  harness is `eggcc/infra/`.
- **License:** open source (`eggcc/LICENSE`). **Requires LLVM 18.**

## 5. Layout

Inside the container (working directory `/root`):

```
/root/
├── reproduce.sh                 smoke | full   →   figures written to /out
└── eggcc/                       source
    ├── artifact/                reproduce.sh, README.md (this file)
    ├── infra/                   nightly.py, graphs.py, plot_cdf.py, setup_gurobi.sh,
    │                            Dockerfile, build_docker.sh, BUILDING.md
    ├── dag_in_context/          egglog pipeline + Statewalk DP extractor (src/tiger/)
    └── benchmarks/passing/      benchmark suite (Bril, PolyBench, raytracer, Fenwick)
/out/                            figures land here (bind-mount a host directory)
```
(Toolchain dirs — `.cargo`, `.rustup`, `.eggcc-venv` — live under `/root` as hidden dirs.)

## Building the image

Authors build and publish the image with `infra/build_docker.sh` (see `infra/Dockerfile`).
See **`infra/BUILDING.md`** for the full build, multi-platform, and publish steps.
