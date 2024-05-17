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

function getDataForBenchmarkRunMode(benchmark, runMode) {
  const entries = GLOBAL_DATA.currentRun.filter(
    (entry) => entry.benchmark === benchmark && entry.runMethod === runMode,
  );
  if (entries.length === 0) {
    console.warn(`no data for ${benchmark} ${runMode}`);
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
    return entry.mean;
  } else if (GLOBAL_DATA.chartMode === "speedup") {
    return entry.speedup;
  } else {
    throw new Error(`unknown chart mode ${GLOBAL_DATA.chartMode}`);
  }
}

function parseDataForChart(sortByMode) {
  const BASELINE_MODE = "llvm-peep";
  let sortedModes = Array.from(GLOBAL_DATA.enabledModes).filter(
    (x) => x !== BASELINE_MODE,
  );
  sortedModes.unshift(BASELINE_MODE);

  const benchmarks = GLOBAL_DATA.enabledBenchmarks;
  let sortedBenchmarks = Array.from(benchmarks).sort();

  const data = {};
  sortedModes.forEach((mode) => {
    data[mode] = {};
    benchmarks.forEach((benchmark) => {
      const entry = getDataForBenchmarkRunMode(benchmark, mode);
      if (entry) {
        data[mode][benchmark] = {
          mode: mode,
          benchmark: benchmark,
          mean: entry.hyperfine.results[0].mean,
          stddev: entry.hyperfine.results[0].stddev,
        };
        if (mode === BASELINE_MODE) {
          data[mode][benchmark].speedup = 1;
        } else {
          const baseline = data[BASELINE_MODE][benchmark];
          if (baseline) {
            data[mode][benchmark].speedup =
              baseline.mean / data[mode][benchmark].mean;
          } else {
            console.warn(`No speedup baseline for ${benchmark}`);
          }
        }
      }
    });
    if (mode === sortByMode) {
      sortedBenchmarks = Object.values(data[mode])
        .sort((a, b) => getValue(b) - getValue(a))
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

      datasets[mode].data[idx] = getValue(point);
      if (GLOBAL_DATA.chartMode === "absolute") {
        datasets[mode].errorBars[point.benchmark] = {
          plus: point.stddev,
          minus: point.stddev,
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
