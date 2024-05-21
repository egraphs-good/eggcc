const COLORS = {
  rvsdg_roundtrip: "red",
  "cranelift-O3": "blue",
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
  if (GLOBAL_DATA.chart.mode === "absolute") {
    return entry.hyperfine.results[0].mean;
  } else if (GLOBAL_DATA.chart.mode === "speedup") {
    const baseline = getEntry(entry.benchmark, BASELINE_MODE);
    if (!baseline) {
      addWarning(`No speedup baseline for ${benchmark}`);
    }
    const baseV = baseline.hyperfine.results[0].mean;
    const expV = entry.hyperfine.results[0].mean;
    // return (baseV - expV) / expV * 100;
    return baseV / expV;
  } else {
    throw new Error(`unknown chart mode ${GLOBAL_DATA.chart.mode}`);
  }
}

function getError(entry) {
  if (GLOBAL_DATA.chart.mode === "absolute") {
    return entry.hyperfine.results[0].stddev;
  } else {
    return undefined;
  }
}

function parseDataForChart() {
  const benchmarks = GLOBAL_DATA.enabledBenchmarks;
  const sortByMode = GLOBAL_DATA.chart.sortBy;
  let sortedBenchmarks = Array.from(benchmarks).sort();

  const data = {};
  // First, compute value and error for each mode and benchmark
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
    // Then, sort the benchmarks by the specified mode
    if (mode === sortByMode) {
      sortedBenchmarks = Object.values(data[mode])
        .sort((a, b) => b.value - a.value)
        .map((x) => x.benchmark);
    }
  });

  // ChartJS wants the data formatted so that there's an array of values for each mode
  // and a corresponding array of labels (benchmarks)
  const datasets = {};
  GLOBAL_DATA.enabledModes.forEach((mode) => {
    datasets[mode] = {
      label: mode,
      backgroundColor: COLORS[mode],
      data: Array(benchmarks.size).fill(0),
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

  // Show baseline as dotted line at 1x if speedup
  if (GLOBAL_DATA.chart.mode === "speedup") {
    datasets[BASELINE_MODE] = {
      label: BASELINE_MODE,
      data: Array(benchmarks.size + 1).fill(1),
      type: 'line',
      borderColor: "purple",
      fill: false,
      borderWidth: 5,
      borderDash: [5, 5],
      pointRadius: 0,
      order: 1
    }
  }


  return {
    labels: Array.from(sortedBenchmarks),
    datasets: Object.values(datasets),
  };
}

function initializeChart() {
  const ctx = document.getElementById("chart");

  const chartData = parseDataForChart();

  GLOBAL_DATA.chart.chart = new Chart(ctx, {
    type: "bar",
    data: chartData,
    options: {
      legend: {
        onClick: (_, item) => {
          GLOBAL_DATA.chart.sortBy = item.text;
          refreshChart();
        },
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
function refreshChart() {
  if (!GLOBAL_DATA.chart.chart) {
    return;
  }
  GLOBAL_DATA.chart.chart.data = parseDataForChart();
  GLOBAL_DATA.chart.chart.update();
}
