const COLORS = {
  "rvsdg-round-trip-to-executable": "red",
  "llvm-O0-O0": "black",
  "llvm-O1-O0": "green",
  "llvm-O2-O0": "orange",
  "llvm-O3-O0": "purple",
  "llvm-O3-O3": "gold",
  "llvm-eggcc-O0-O0": "blue",
  "llvm-eggcc-sequential-O0-O0": "pink",
  "llvm-eggcc-O3-O0": "brown",
  "llvm-eggcc-O3-O3": "lightblue",
  "llvm-eggcc-ablation-O0-O0": "blue",
  "llvm-eggcc-ablation-O3-O0": "green",
  "llvm-eggcc-ablation-O3-O3": "orange",
  "eggcc-ILP-O0-O0": "red",
  "llvm-eggcc-tiger-O0-O0": "cyan",
  "llvm-eggcc-tiger-WL-O0-O0": "magenta",
  "llvm-eggcc-tiger-ILP-O0-O0": "green",
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

function variance(cycles) {
  const mean = cycles.reduce((a, b) => a + b, 0) / (cycles.length - 1);
  const squared_diffs = cycles.map((c) => (c - mean) ** 2);
  // TODO kevin said we might want to use bessel's correction here, but we don't currently
  const res =
    squared_diffs.reduce((a, b) => a + b, 0) / (squared_diffs.length - 1);
  return res;
}

function stddev(cycles) {
  return Math.sqrt(variance(cycles));
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

function normalized(entry, baseline) {
  const baseV = mean(baseline["cycles"]);
  const expV = mean(entry["cycles"]);
  // If you change this, also change the displayed formula in index.html
  return expV / baseV;
}

function getValue(entry) {
  if (GLOBAL_DATA.chart.mode === "absolute") {
    return mean(entry["cycles"]);
  } else if (GLOBAL_DATA.chart.mode === "normalized") {
    const baseline = getEntry(entry.benchmark, BASELINE_MODE);
    if (!baseline) {
      addWarning(`No normalized baseline for ${benchmark}`);
    }
    return normalized(entry, baseline);
  } else {
    throw new Error(`unknown chart mode ${GLOBAL_DATA.chart.mode}`);
  }
}

// get error bars for the bar chart
function getError(entry) {
  if (GLOBAL_DATA.chart.mode === "absolute") {
    return stddev(entry["cycles"]);
  } else {
    // when normalized, normalize the values then take the stddev
    const baseline = getEntry(entry.benchmark, BASELINE_MODE);
    if (!baseline) {
      addWarning(`No normalized baseline for ${benchmark}`);
    }
    const baseline_mean = mean(baseline["cycles"]);
    const normalized = entry["cycles"].map((c) => c / baseline_mean);

    return stddev(normalized);
  }
}

// Register a plugin to draw hatch pattern over timeout bars without affecting legend colors.
(function registerTimeoutPatternPlugin() {
  if (typeof Chart === "undefined") return; // safety
  if (Chart._timeoutPatternPluginRegistered) return;
  const plugin = {
    afterDatasetsDraw: function (chart) {
      const ctx = chart.ctx;
      chart.config.data.datasets.forEach((ds, dsIndex) => {
        if (!ds._timeoutFlags) return;
        const meta = chart.getDatasetMeta(dsIndex);
        meta.data.forEach((bar, i) => {
          if (!ds._timeoutFlags[i]) return;
          const model = bar._model || bar; // Chart.js v2 vs potential future
          const left = model.x - model.width / 2;
          const right = model.x + model.width / 2;
          const top = model.y;
          const bottom = model.base;
          ctx.save();
          ctx.beginPath();
          ctx.rect(left, top, right - left, bottom - top);
          ctx.clip();
          ctx.globalAlpha = 0.35;
          ctx.fillStyle = ds.backgroundColor || COLORS[ds.label] || "gray";
          ctx.fillRect(left, top, right - left, bottom - top);
          ctx.globalAlpha = 1.0;
          ctx.strokeStyle = "white";
          ctx.lineWidth = 2;
          // diagonal stripes
          const step = 8;
          for (
            let x = left - (bottom - top);
            x < right + (bottom - top);
            x += step
          ) {
            ctx.beginPath();
            ctx.moveTo(x, bottom);
            ctx.lineTo(x + (bottom - top), top);
            ctx.stroke();
          }
          ctx.restore();
        });
      });
    },
  };
  Chart.plugins.register(plugin); // Chart.js 2 style
  Chart._timeoutPatternPluginRegistered = true;
})();

function parseDataForChart() {
  const benchmarks = enabledBenchmarks();
  const sortByMode = GLOBAL_DATA.chart.sortBy;
  let sortedBenchmarks = Array.from(benchmarks).sort();

  const data = {};
  GLOBAL_DATA.checkedModes.forEach((mode) => {
    data[mode] = {};
    benchmarks.forEach((benchmark) => {
      const entry = getEntry(benchmark, mode);
      if (!entry) return;
      const didFail = entry.failed || !Array.isArray(entry.cycles);
      let value = 0;
      let error = 0;
      if (!didFail) {
        value = getValue(entry);
        error = getError(entry);
      }
      data[mode][benchmark] = {
        mode,
        benchmark,
        value,
        error,
        failed: didFail,
      };
    });
    if (mode === sortByMode) {
      sortedBenchmarks = Object.values(data[mode])
        .sort((a, b) => b.value - a.value)
        .map((x) => x.benchmark);
    }
  });

  // For each mode lift timeout bars to max non-timeout height (works for absolute & normalized)
  GLOBAL_DATA.checkedModes.forEach((mode) => {
    const points = Object.values(data[mode]);
    const nonTimeout = points.filter((p) => !p.failed).map((p) => p.value);
    const maxValue = nonTimeout.length ? Math.max(...nonTimeout) : 1;
    points.forEach((p) => {
      if (p.failed) {
        p.value = maxValue;
        p.error = 0;
      }
    });
  });

  const datasets = {};
  GLOBAL_DATA.checkedModes.forEach((mode) => {
    const points = Object.values(data[mode]);
    const dsData = Array(sortedBenchmarks.length).fill(0);
    const timeoutFlags = Array(sortedBenchmarks.length).fill(false);
    const errorBars = {};
    points.forEach((point) => {
      const idx = sortedBenchmarks.indexOf(point.benchmark);
      if (idx === -1) return;
      dsData[idx] = point.value;
      if (point.failed) {
        timeoutFlags[idx] = true;
        errorBars[point.benchmark] = { plus: 0, minus: 0, failed: true };
      } else if (point.error) {
        errorBars[point.benchmark] = { plus: point.error, minus: point.error };
      }
    });
    datasets[mode] = {
      label: mode,
      backgroundColor: COLORS[mode], // keep solid color for legend
      data: dsData,
      borderWidth: 1,
      errorBars,
      _timeoutFlags: timeoutFlags,
    };
  });

  if (GLOBAL_DATA.chart.mode === "normalized") {
    datasets[BASELINE_MODE] = {
      label: BASELINE_MODE,
      data: Array(sortedBenchmarks.length + 1).fill(1),
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
