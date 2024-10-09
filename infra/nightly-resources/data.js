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

function getBaselineHyperfine(benchmark, runMethod) {
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
      return baseline[0].hyperfine.results[0];
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
    ?.filter((o) => o.benchmark === benchmark)
    .map((o) => {
      const baselineHyperfine = getBaselineHyperfine(o.benchmark, o.runMethod);
      const hyperfine = o.hyperfine.results[0];
      const rowData = {
        runMethod: o.runMethod,
        mean: { class: "", value: tryRound(hyperfine.mean) },
        meanVsBaseline: diffAttribute(hyperfine, baselineHyperfine, "mean"),
        min: { class: "", value: tryRound(hyperfine.min) },
        minVsBaseline: diffAttribute(hyperfine, baselineHyperfine, "min"),
        max: { class: "", value: tryRound(hyperfine.max) },
        maxVsBaseline: diffAttribute(hyperfine, baselineHyperfine, "max"),
        median: { class: "", value: tryRound(hyperfine.median) },
        medianVsBaseline: diffAttribute(hyperfine, baselineHyperfine, "median"),
        stddev: { class: "", value: tryRound(hyperfine.stddev) },
      };
      if (shouldHaveLlvm(o.runMethod)) {
        rowData.runMethod = `<a target="_blank" rel="noopener noreferrer" href="llvm.html?benchmark=${benchmark}&runmode=${o.runMethod}">${o.runMethod}</a>`;
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
