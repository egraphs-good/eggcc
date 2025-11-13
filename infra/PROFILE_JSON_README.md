# profile.json Data Layout

This document specifies the schema and invariants for the nightly profiling output file: `profile.json`.

Each element of the top-level JSON array represents a (benchmark, runMethod) pair.

## Top-Level Array
```
[
  { <Entry> },
  { <Entry> },
  ...
]
```

## Check `failed` first. If true, ignore all other metric fields (they will be `false`).

## Example (Non-timeout)
```
{
  "benchmark": "matmul",
  "runMethod": "eggcc-O3-O3",
  "suite": "polybench",
  "path": "benchmarks/passing/polybench/matmul.bril",
  "cycles": [120345, 120100, 120512],
  "eggccCompileTimeSecs": 1.234,
  "eggccSerializationTimeSecs": 0.045,
  "eggccExtractionTimeSecs": 5.678,
  "llvmCompileTimeSecs": 0.890,
  "extractRegionTimings": [],
  "failed": false,
  "ILPRegionTimeOut": false
}
```

## Example (Timeout)
```
{
  "benchmark": "matmul",
  "runMethod": "eggcc-tiger-O0-O0",
  "suite": "polybench",
  "path": "benchmarks/passing/polybench/matmul.bril",
  "cycles": false,
  "eggccCompileTimeSecs": false,
  "eggccSerializationTimeSecs": false,
  "eggccExtractionTimeSecs": false,
  "llvmCompileTimeSecs": false,
  "extractRegionTimings": false,
  "failed": true,
  "ILPRegionTimeOut": true
}
```
