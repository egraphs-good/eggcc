const COLORS = {
  rvsdg_roundtrip: "red",
  "cranelift-O0": "orange",
  "cranelift-O0-eggcc": "yellow",
  "cranelift-O3": "green",
  "cranelift-O3-eggcc": "blue",
  "llvm-peep": "purple",
  "llvm-peep-eggcc": "pink",
  "llvm-O3": "gray",
  "llvm-O3-eggcc": "brown",
};

const BASELINE_MODE = "llvm-peep";

function getEntry(benchmark, runMode) {
  const entries = GLOBAL_DATA.currentRun.filter(
    (entry) => entry.benchmark === benchmark && entry.runMethod === runMode,
  );
  if (entries.length === 0) {
    addWarning(`no data for ${benchmark} ${runMode}`);
  } else if (entries.length > 1) {
    throw new Error(
      `duplicate entries for ${benchmark} ${runMode} (this probably shouldn't happen)`,
    );
  } else {
    return entries[0];
  }
}

function getValue(entry) {
  if (GLOBAL_DATA.chartMode === "absolute") {
    return entry.hyperfine.results[0].mean;
  } else if (GLOBAL_DATA.chartMode === "speedup") {
    const baseline = getEntry(entry.benchmark, BASELINE_MODE);
    if (!baseline) {
      addWarning(`No speedup baseline for ${benchmark}`);
    }
    return baseline.hyperfine.results[0].mean / entry.hyperfine.results[0].mean;
  } else {
    throw new Error(`unknown chart mode ${GLOBAL_DATA.chartMode}`);
  }
}

function getError(entry) {
  if (GLOBAL_DATA.chartMode === "absolute") {
    return entry.hyperfine.results[0].stddev;
  } else {
    return undefined;
  }
}

function parseDataForChart(sortByMode) {
  const benchmarks = GLOBAL_DATA.enabledBenchmarks;
  let sortedBenchmarks = Array.from(benchmarks).sort();

  const data = {};
  GLOBAL_DATA.enabledModes.forEach((mode) => {
    data[mode] = {};
    benchmarks.forEach((benchmark) => {
      const entry = getEntry(benchmark, mode);
      if (entry) {
        data[mode][benchmark] = {
          mode: mode,
          benchmark: benchmark,
          value: getValue(entry),
          error: getError(entry),
        };
      }
    });
    if (mode === sortByMode) {
      sortedBenchmarks = Object.values(data[mode])
        .sort((a, b) => b.value - a.value)
        .map((x) => x.benchmark);
    }
  });
  const datasets = {};
  GLOBAL_DATA.enabledModes.forEach((mode) => {
    datasets[mode] = {
      label: mode,
      backgroundColor: COLORS[mode],
      data: Array(benchmarks.length).fill(0),
      borderWidth: 1,
      errorBars: {},
    };
    Object.values(data[mode]).forEach((point) => {
      const idx = sortedBenchmarks.indexOf(point.benchmark);

      datasets[mode].data[idx] = point.value;
      if (point.error) {
        datasets[mode].errorBars[point.benchmark] = {
          plus: point.error,
          minus: point.error,
        };
      }
    });
  });
  return {
    labels: Array.from(sortedBenchmarks),
    datasets: Object.values(datasets),
  };
}

function initializeChart() {
  const ctx = document.getElementById("chart");

  const chartData = parseDataForChart();

  GLOBAL_DATA.chart = new Chart(ctx, {
    type: "bar",
    data: chartData,
    options: {
      scales: {
        y: {
          beginAtZero: true,
        },
      },
      legend: {
        onClick: (_, item) => refreshChart(item.text),
      },
      plugins: {
        chartJsPluginErrorBars: {
          color: "black",
        },
      },
    },
  });
}

// Seems important for the charting library to change the data but not
// create a new chart to avoid some weird rendering flicekrs.
function refreshChart(sortByMode) {
  if (!GLOBAL_DATA.chart) {
    return;
  }
  GLOBAL_DATA.chart.data = parseDataForChart(sortByMode);
  GLOBAL_DATA.chart.update();
}
