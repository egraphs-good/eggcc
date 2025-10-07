async function fetchJson(url) {
  const resp = await fetch(url);
  const data = await resp.json();
  return data;
}

async function fetchText(url) {
  const resp = await fetch(url);
  const data = await resp.text();
  return data;
}

function getRow(benchmark, runMethod) {
  return GLOBAL_DATA.currentRun.find(
    (row) => row.benchmark === benchmark && row.runMethod === runMethod,
  );
}

// Get the row for the comparison branch
// for the given benchmark and run method
function getComparison(benchmark, runMethod) {
  const baselineData =
    GLOBAL_DATA.baselineRun?.filter((o) => o.benchmark === benchmark) || [];
  if (baselineData.length === 0) {
    addWarning(`Baseline doesn't have ${benchmark} benchmark`);
  } else {
    const baseline = baselineData.filter((o) => o.runMethod === runMethod);
    if (baseline.length === 0) {
      addWarning(`No baseline data for ${benchmark} ${runMethod}`);
    } else if (baseline.length !== 1) {
      throw new Error(
        `Baseline had multiple entries for ${benchmark} ${runMethod}`,
      );
    } else {
      return baseline[0];
    }
  }
}

function shouldHaveLlvm(runMethod) {
  return [
    "rvsdg-round-trip-to-executable",
    "llvm-O0-O0",
    "llvm-O1-O0",
    "llvm-O2-O0",
    "llvm-eggcc-O0-O0",
    "llvm-eggcc-sequential-O0-O0",
    "llvm-O3-O0",
    "llvm-O3-O3",
    "llvm-eggcc-O3-O0",
    "llvm-eggcc-O3-O3",
  ].includes(runMethod);
}

function getBrilPathForBenchmark(benchmark) {
  const o = GLOBAL_DATA.currentRun.find((o) => o.benchmark === benchmark);
  if (!o) {
    console.error(
      `couldn't find entry for ${benchmark} (this shouldn't happen)`,
    );
  }
  return o.path;
}

// calculates the geometric mean over a list of ratios
function geometricMean(values) {
  return Math.pow(
    values.reduce((a, b) => a * b, 1),
    1 / values.length,
  );
}

// suite can be undefined, in which case it uses all
// enabled benchmarks
function getOverallStatistics(suite) {
  var benchmarks = enabledBenchmarks();
  if (typeof suite !== "undefined") {
    benchmarks = benchmarksInSuite(suite);
  }

  const result = [];
  for (const treatment of treatments()) {
    const normalized_cycles = [];
    for (const benchmark of benchmarks) {
      const row = getRow(benchmark, treatment);
      const baseline = getRow(benchmark, BASELINE_MODE);
      if (row && baseline && !row.failed) {
        normalized_cycles.push(normalized(row, baseline));
      }
    }

    const eggcc_compile_times = [];
    const eggcc_extraction_times = [];
    const eggcc_serialization_times = [];
    const llvm_compile_times = [];
    for (const benchmark of benchmarks) {
      const row = getRow(benchmark, treatment);
      if (!row || row.failed) continue;
      eggcc_compile_times.push(row.eggccCompileTimeSecs);
      eggcc_extraction_times.push(row.eggccExtractionTimeSecs);
      eggcc_serialization_times.push(row.eggccSerializationTimeSecs);
      llvm_compile_times.push(row.llvmCompileTimeSecs);
    }

    result.push({
      Treatment: treatment,
      "Normalized Mean": normalized_cycles.length
        ? tryRound(geometricMean(normalized_cycles))
        : "timeout",
      "Eggcc Compile Time": eggcc_compile_times.length
        ? tryRound(mean(eggcc_compile_times))
        : "timeout",
      "Eggcc Serialization Time": eggcc_serialization_times.length
        ? tryRound(mean(eggcc_serialization_times))
        : "timeout",
      "Eggcc Extraction Time": eggcc_extraction_times.length
        ? tryRound(mean(eggcc_extraction_times))
        : "timeout",
      "LLVM Compile Time": llvm_compile_times.length
        ? tryRound(mean(llvm_compile_times))
        : "timeout",
    });
  }
  return result;
}

function getDataForBenchmark(benchmark) {
  const executions = GLOBAL_DATA.currentRun
    ?.filter((row) => row.benchmark === benchmark)
    .map((row) => {
      const baseline = getRow(benchmark, BASELINE_MODE);
      const comparisonCycles = getComparison(
        row.benchmark,
        row.runMethod,
      )?.cycles;
      const cycles = row.cycles;
      const didFail = row.failed;
      const rowData = {
        runMethod: row.runMethod,
        mean: {
          class: didFail ? "timeout" : "",
          value: didFail ? "timeout" : tryRound(mean(cycles)),
        },
        meanVsOtherBranch: didFail
          ? { class: "timeout", value: "timeout" }
          : getDifference(cycles, comparisonCycles, mean),
        min: {
          class: didFail ? "timeout" : "",
          value: didFail ? "timeout" : tryRound(min_cycles(cycles)),
        },
        max: {
          class: didFail ? "timeout" : "",
          value: didFail ? "timeout" : tryRound(max_cycles(cycles)),
        },
        median: {
          class: didFail ? "timeout" : "",
          value: didFail ? "timeout" : tryRound(median_cycles(cycles)),
        },
        stddev: {
          class: didFail ? "timeout" : "",
          value: didFail ? "timeout" : tryRound(stddev(cycles)),
        },
        eggccCompileTimeSecs: {
          class: didFail ? "timeout" : "",
          value: tryRound(row.eggccCompileTimeSecs),
        },
        eggccSerializationTimeSecs: {
          class: didFail ? "timeout" : "",
          value: tryRound(row.eggccSerializationTimeSecs),
        },
        eggccExtractionTimeSecs: {
          class: didFail ? "timeout" : "",
          value: tryRound(row.eggccExtractionTimeSecs),
        },
        llvmCompileTimeSecs: {
          class: didFail ? "timeout" : "",
          value: tryRound(row.llvmCompileTimeSecs),
        },
        normalized: {
          class: didFail ? "timeout" : "",
          value:
            didFail || !baseline
              ? "timeout"
              : tryRound(normalized(row, baseline)),
        },
        failed: !!row.failed,
      };
      if (shouldHaveLlvm(row.runMethod)) {
        rowData.runMethod = `<a target="_blank" rel="noopener noreferrer" href="llvm.html?benchmark=${benchmark}&runmode=${row.runMethod}">${row.runMethod}</a>`;
      }
      return rowData;
    });

  if (executions.length > 1) {
    const cols = ["mean", "min", "max", "median"];
    cols.forEach((col) => {
      const numeric = executions.filter((e) => e[col].value !== "timeout");
      if (numeric.length === 0) return;
      const sorted = numeric
        .map((e) => e[col])
        .sort((a, b) => a.value - b.value);
      const min = sorted[0].value;
      const max = sorted[sorted.length - 1].value;
      sorted.forEach((item) => {
        if (item.value === min) {
          item.class = "good";
        }
        if (item.value === max) {
          item.class = "bad";
        }
      });
    });
  }

  return executions;
}
