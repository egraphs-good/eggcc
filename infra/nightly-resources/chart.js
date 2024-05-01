const COLORS = {
  rvsdg_roundtrip: "red",
  egglog_noopt_brilift_noopt: "orange",
  egglog_noopt_brilift_opt: "yellow",
  egglog_opt_brilift_noopt: "green",
  egglog_opt_brilift_opt: "blue",
  egglog_noopt_bril_llvm_noopt: "purple",
  egglog_noopt_bril_llvm_opt: "pink",
  egglog_opt_bril_llvm_noopt: "gray",
  egglog_opt_bril_llvm_opt: "brown",
};

function parseDataForChart() {
  const benchmarks = Array.from(GLOBAL_DATA.enabledBenchmarks);
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
      const benchDataForMode = GLOBAL_DATA.currentRun[benchName][mode];
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
