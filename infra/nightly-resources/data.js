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

function getBaselineCycles(benchmark, runMethod) {
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
      return baseline[0]["cycles"];
    }
  }
}

function shouldHaveLlvm(runMethod) {
  return [
    "rvsdg-round-trip-to-executable",
    "llvm-O0",
    "llvm-O1",
    "llvm-O2",
    "llvm-O0-eggcc",
    "llvm-O3",
    "llvm-O3-eggcc",
  ].includes(runMethod);
}

function getBrilPathForBenchmark(benchmark) {
  const o = GLOBAL_DATA.currentRun.find((o) => o.benchmark === benchmark);
  if (!o) {
    console.error(
      `couldn't find entry for ${benchmark} (this shouldn't happen)`,
    );
  }
  return o.metadata.path;
}

function getDataForBenchmark(benchmark) {
  const executions = GLOBAL_DATA.currentRun
    ?.filter((row) => row.benchmark === benchmark)
    .map((row) => {
      const baselineCycles = getBaselineCycles(row.benchmark, row.runMethod);
      const cycles = row["cycles"];
      const rowData = {
        runMethod: row.runMethod,
        mean: { class: "", value: tryRound(mean_cycles(cycles)) },
        meanVsBaseline: getDifference(cycles, baselineCycles, mean_cycles),
        min: { class: "", value: tryRound(min_cycles(cycles)) },
        minVsBaseline: getDifference(cycles, baselineCycles, min_cycles),
        max: { class: "", value: tryRound(max_cycles(cycles)) },
        maxVsBaseline: getDifference(cycles, baselineCycles, max_cycles),
        median: { class: "", value: tryRound(median_cycles(cycles)) },
        medianVsBaseline: getDifference(cycles, baselineCycles, median_cycles),
        stddev: { class: "", value: tryRound(median_cycles(cycles)) },
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
