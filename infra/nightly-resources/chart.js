const COLORS = {
  "rvsdg-round-trip-to-executable": "red",
  "llvm-O0-O0": "purple",
  "llvm-O1-O0": "green",
  "llvm-O2-O0": "orange",
  "llvm-O3-O0": "gray",
  "llvm-O3-O3": "gold",
  "llvm-eggcc-O0-O0": "pink",
  "llvm-eggcc-sequential-O0-O0": "blue",
  "llvm-eggcc-O3-O0": "brown",
};

const BASELINE_MODE = "llvm-O0-O0";

// TODO these functions (mean, median, ect) are duplicated in generate_line_counts.py
// we could move the computation of the latex table to js to solve this problem

// Given a list of integers, compute the mean
// number of cycles
function mean(cycles) {
  return cycles.reduce((a, b) => a + b, 0) / cycles.length;
}

function median_cycles(cycles) {
  const sorted = cycles.sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 0) {
    return (sorted[mid - 1] + sorted[mid]) / 2;
  } else {
    return sorted[mid];
  }
}

function max_cycles(cycles) {
  return Math.max(...cycles);
}

function min_cycles(cycles) {
  return Math.min(...cycles);
}

function stddev_cycles(cycles) {
  const mean = cycles.reduce((a, b) => a + b, 0) / (cycles.length - 1);
  const squared_diffs = cycles.map((c) => (c - mean) ** 2);
  // TODO kevin said we might want to use bessel's correction here
  const bessels_corrected =
    squared_diffs.reduce((a, b) => a + b, 0) / squared_diffs.length;
  return Math.sqrt(bessels_corrected);
}

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

function speedup(entry, baseline) {
  const baseV = mean(baseline["cycles"]);
  const expV = mean(entry["cycles"]);
  // If you change this, also change the displayed formula in index.html
  return baseV / expV;
}

function getValue(entry) {
  if (GLOBAL_DATA.chart.mode === "absolute") {
    return mean(entry["cycles"]);
  } else if (GLOBAL_DATA.chart.mode === "speedup") {
    const baseline = getEntry(entry.benchmark, BASELINE_MODE);
    if (!baseline) {
      addWarning(`No speedup baseline for ${benchmark}`);
    }
    return speedup(entry, baseline);
  } else {
    throw new Error(`unknown chart mode ${GLOBAL_DATA.chart.mode}`);
  }
}

function getError(entry) {
  if (GLOBAL_DATA.chart.mode === "absolute") {
    return stddev_cycles(entry["cycles"]);
  } else {
    // Error is given using propagation of error formula for two variables
    // f = baseV / expV
    const baseline = getEntry(entry.benchmark, BASELINE_MODE);
    if (!baseline) {
      addWarning(`No speedup baseline for ${benchmark}`);
    }

    const baseV = mean(baseline["cycles"]);
    const expV = mean(entry["cycles"]);
    const baseStd = stddev_cycles(baseline["cycles"]);
    const expStd = stddev_cycles(entry["cycles"]);

    // Speedup calculation
    const speedup = baseV / expV;

    // Error propagation
    const relativeBaseError = baseStd / baseV;
    const relativeExpError = expStd / expV;

    const speedupError =
      speedup * Math.sqrt(relativeBaseError ** 2 + relativeExpError ** 2);

    return speedupError;
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
      type: "line",
      borderColor: COLORS[BASELINE_MODE],
      fill: false,
      borderWidth: 5,
      borderDash: [5, 5],
      pointRadius: 0,
      order: 1,
    };
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
          color: "gray",
          lineWidth: 1,
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
