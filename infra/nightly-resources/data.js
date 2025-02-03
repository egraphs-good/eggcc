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

function getOverallStatistics() {
  // generate one row per treatment...
  const result = [];
  for (const treatment of treatments) {
    const normalized_cycles = [];
    // for each benchmark, calculate the normalized cycles
    for (const benchmark of GLOBAL_DATA.enabledBenchmarks) {
      const row = getRow(benchmark, treatment);
      const baseline = getRow(benchmark, BASELINE_MODE);
      if (row && baseline) {
        normalized_cycles.push(normalized(row, baseline));
      }
    }

    const eggcc_compile_times = [];
    const llvm_compile_times = [];
    for (const benchmark of GLOBAL_DATA.enabledBenchmarks) {
      const row = getRow(benchmark, treatment);
      eggcc_compile_times.push(row.eggccCompileTimeSecs);
      llvm_compile_times.push(row.llvmCompileTimeSecs);
    }

    result.push({
      runMethod: treatment,
      geoMeanNormalized: tryRound(geometricMean(normalized_cycles)),
      meanEggccCompileTimeSecs: tryRound(mean(eggcc_compile_times)),
      meanLlvmCompileTimeSecs: tryRound(mean(llvm_compile_times)),
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
      const cycles = row["cycles"];
      const rowData = {
        runMethod: row.runMethod,
        mean: { class: "", value: tryRound(mean(cycles)) },
        meanVsBaseline: getDifference(cycles, comparisonCycles, mean),
        min: { class: "", value: tryRound(min_cycles(cycles)) },
        minVsBaseline: getDifference(cycles, comparisonCycles, min_cycles),
        max: { class: "", value: tryRound(max_cycles(cycles)) },
        maxVsBaseline: getDifference(cycles, comparisonCycles, max_cycles),
        median: { class: "", value: tryRound(median_cycles(cycles)) },
        medianVsBaseline: getDifference(
          cycles,
          comparisonCycles,
          median_cycles,
        ),
        stddev: { class: "", value: tryRound(stddev(cycles)) },
        eggccCompileTimeSecs: {
          class: "",
          value: tryRound(row.eggccCompileTimeSecs),
        },
        llvmCompileTimeSecs: {
          class: "",
          value: tryRound(row.llvmCompileTimeSecs),
        },
        normalized: { class: "", value: tryRound(normalized(row, baseline)) },
      };
      if (shouldHaveLlvm(row.runMethod)) {
        rowData.runMethod = `<a target="_blank" rel="noopener noreferrer" href="llvm.html?benchmark=${benchmark}&runmode=${row.runMethod}">${row.runMethod}</a>`;
      }
      return rowData;
    });

  if (executions.length > 1) {
    const cols = ["mean", "min", "max", "median"];
    cols.forEach((col) => {
      const sorted = executions
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
