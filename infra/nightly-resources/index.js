// copied from profile.py
const treatments = [
  "rvsdg-round-trip-to-executable",
  "llvm-O0",
  "llvm-O1",
  "llvm-O2",
  "llvm-O0-eggcc",
  "llvm-O0-eggcc-sequential",
  "llvm-O3",
  "llvm-O3-eggcc",
];

const GLOBAL_DATA = {
  enabledModes: new Set(),
  enabledBenchmarks: new Set(),
  warnings: new Set(),
  currentRun: [],
  baselineRun: [],
  chart: {
    chart: undefined,
    mode: "absolute",
    sortBy: undefined,
  },
};

function addWarning(warning) {
  GLOBAL_DATA.warnings.add(warning);
}

function clearWarnings() {
  GLOBAL_DATA.warnings.clear();
}

function refreshView() {
  if (!GLOBAL_DATA.baselineRun) {
    addWarning("no baseline to compare to");
  }

  const byBench = {};
  GLOBAL_DATA.enabledBenchmarks.forEach((benchmark) => {
    byBench[benchmark] = getDataForBenchmark(benchmark);
  });
  const tableData = Object.keys(byBench).map((bench) => ({
    name: `<a target="_blank" rel="noopener noreferrer" href="https://github.com/egraphs-good/eggcc/tree/main/${getBrilPathForBenchmark(bench)}">${bench}</a>`,
    executions: { data: byBench[bench] },
  }));
  tableData.sort((l, r) => l.name - r.name);

  document.getElementById("profile").innerHTML = ConvertJsonToTable(tableData);

  // fill in the overall stats table
  const overallStats = getOverallStatistics();
  const overallTable = document.getElementById("overall-stats-table");
  overallTable.innerHTML = ConvertJsonToTable(overallStats);

  renderWarnings();
  refreshChart();
}

function renderWarnings() {
  const toggle = document.getElementById("warnings-toggle");
  toggle.innerText = `\u25B6 Show ${GLOBAL_DATA.warnings.size} Warnings`;

  const warningContainer = document.getElementById("warnings");
  warningContainer.innerHTML = "";
  GLOBAL_DATA.warnings.forEach((warning) => {
    const warningElement = document.createElement("p");
    warningElement.innerText = warning;
    warningContainer.appendChild(warningElement);
  });
}

/// Manipulating UI elements:

function makeCheckbox(parent, mode) {
  // make a check box for enabling this mode
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.id = mode;
  checkbox.checked = true;
  parent.appendChild(checkbox);
  // make a label for the checkbox
  const label = document.createElement("label");
  label.htmlFor = mode;
  label.innerText = mode;
  parent.appendChild(label);
  // make a line break
  parent.appendChild(document.createElement("br"));
  return checkbox;
}

function makeSelectors() {
  treatments.forEach((mode) => {
    const checkbox = makeCheckbox(
      document.getElementById("modeCheckboxes"),
      mode,
    );
    checkbox.onchange = () => toggleCheckbox(mode, GLOBAL_DATA.enabledModes);
  });

  const benchmarks = Array.from(
    new Set(GLOBAL_DATA.currentRun.map((o) => o.benchmark)),
  ).sort();
  benchmarks.forEach((benchmark) => {
    const checkbox = makeCheckbox(
      document.getElementById("benchmarkCheckboxes"),
      benchmark,
    );
    checkbox.onchange = () =>
      toggleCheckbox(benchmark, GLOBAL_DATA.enabledBenchmarks);
  });
}

async function buildNightlyDropdown(element, previousRuns, initialIdx) {
  const select = document.getElementById(element);

  const formatRun = (run) =>
    `${run.branch} - ${run.commit} - ${new Date(
      run.date * 1000,
    ).toDateString()}`;

  previousRuns.forEach((nightly) => {
    const option = document.createElement("option");
    option.innerText = formatRun(nightly);
    select.appendChild(option);
  });

  select.onchange = () => loadBaseline(previousRuns[select.selectedIndex].url);

  select.selectedIndex = initialIdx;
  select.value = formatRun(previousRuns[initialIdx]);
}
