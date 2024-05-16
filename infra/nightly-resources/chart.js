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
  const entries = GLOBAL_DATA.currentRun.filter(entry => entry.benchmark === benchmark && entry.runMethod === runMode);
  if (entries.length === 0) {
    console.warn(`no data for ${benchmark} ${runMode}`);
  } else if (entries.length > 1) {
    throw new Error(`duplicate entries for ${benchmark} ${runMode} (this probably shouldn't happen)`)
  } else {
    return entries[0];
  }
}

function parseDataForChart() {
  const benchmarks = Array.from(GLOBAL_DATA.enabledBenchmarks).sort();
  const datasets = {};
  GLOBAL_DATA.enabledModes.forEach((mode) => {
    datasets[mode] = {
      label: mode,
      backgroundColor: COLORS[mode],
      data: Array(benchmarks.length).fill(0),
      borderWidth: 1,
      errorBars: {},
    };
  });
  benchmarks.forEach((benchName, idx) => {
    GLOBAL_DATA.enabledModes.forEach((mode) => {
      const benchDataForMode = getDataForBenchmarkRunMode(benchName, mode);
      if (benchDataForMode) {
        const mean = benchDataForMode.hyperfine.results[0].mean;
        const stddev = benchDataForMode.hyperfine.results[0].stddev;
        datasets[mode].data[idx] = mean;
        datasets[mode].errorBars[benchName] = { plus: stddev, minus: stddev };
      } else {
        console.warn(`no data for ${benchName} ${mode}`);
      }
    });
  });
  return { labels: benchmarks, datasets: Object.values(datasets) };
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
  if (!GLOBAL_DATA.chart) {
    return;
  }
  GLOBAL_DATA.chart.data = parseDataForChart();
  GLOBAL_DATA.chart.update();
}
