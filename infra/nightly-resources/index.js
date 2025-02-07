// copied from profile.py
const treatments = [
  "rvsdg-round-trip-to-executable",
  "llvm-O0-O0",
  "llvm-O1-O0",
  "llvm-O2-O0",
  "llvm-eggcc-O0-O0",
  "llvm-eggcc-sequential-O0-O0",
  "llvm-O3-O0",
  "llvm-O3-O3",
  "llvm-eggcc-O3-O0",
  "llvm-eggcc-O3-O3",
];

const GLOBAL_DATA = {
  checkedModes: new Set(),
  checkedSuites: new Set(),
  checkedBenchmarks: new Set(),
  warnings: new Set(),
  currentRun: [],
  baselineRun: [],
  chart: {
    chart: undefined,
    mode: "absolute",
    sortBy: undefined,
  },
};

// filter to all the benchmark names that are enabled
// using checkedSuites and checkedBenchmarks
function enabledBenchmarks() {
  return Array.from(GLOBAL_DATA.checkedBenchmarks).filter((benchmark) =>
    GLOBAL_DATA.checkedSuites.has(getRow(benchmark, BASELINE_MODE).suite))
}

// filter current run for enabled benchmarks
// and by checked run modes
function enabledSubsetOfCurrentRun() {
  const benchmarks = enabledBenchmarks();
  return GLOBAL_DATA.currentRun.filter((entry) =>
    GLOBAL_DATA.checkedModes.has(entry.runMethod) && benchmarks.includes(entry),
  );
}

function addWarning(warning) {
  GLOBAL_DATA.warnings.add(warning);
}

function clearWarnings() {
  GLOBAL_DATA.warnings.clear();
}

function addTableTo(element, data, title) {
  // add a h2 element with the title
  const h2 = document.createElement("h2");
  h2.innerText = title;
  element.appendChild(h2);

  // add a button that copies latex for table
  const copyButton = document.createElement("button");
  copyButton.innerText = "Copy Latex";
  copyButton.onclick = () => {
    const table = jsonToLatexTable(data);
    navigator.clipboard.writeText(table);
  };

  // add a button that copies latex macros for table
  const copyMacrosButton = document.createElement("button");
  copyMacrosButton.innerText = "Copy Latex Macros";
  copyMacrosButton.onclick = () => {
    const macros = nestedJsonToLatexMacros(data, "name", "executions", "runMethod");
    console.log("macros");
    console.log(macros);
    navigator.clipboard.writeText(macros);
  };

  element.appendChild(copyButton);
  element.appendChild(copyMacrosButton);

  // add a new div for the table
  const tableDiv = document.createElement("div");
  tableDiv.innerHTML = ConvertJsonToTable(data);
  element.appendChild(tableDiv);
}

function tableForSuite(suite) {
  const byBench = {};
  Array.from(GLOBAL_DATA.checkedBenchmarks)
  .filter((benchmark) => getRow(benchmark, BASELINE_MODE).suite === suite)
  .forEach((benchmark) => {
    byBench[benchmark] = getDataForBenchmark(benchmark);
  });
  const tableData = Object.keys(byBench).map((bench) => ({
    name: `<a target="_blank" rel="noopener noreferrer" href="https://github.com/egraphs-good/eggcc/tree/main/${getBrilPathForBenchmark(
      bench,
    )}">${bench}</a>`,
    executions: { data: byBench[bench] },
  }));
  tableData.sort((l, r) => l.name - r.name);
  return tableData;
}


function dedup(arr) {
  return Array.from(new Set(arr));
}

function getSuites() {
  return dedup(GLOBAL_DATA.currentRun.map((benchmark) => benchmark.suite));
}


function refreshView() {
  if (!GLOBAL_DATA.baselineRun) {
    addWarning("no baseline to compare to");
  }

  // clear the tables element
  while (document.getElementById("tables").firstChild) {
    document.getElementById("tables").removeChild(document.getElementById("tables").firstChild);
  }

  // fill in the overall stats table
  const overallStats = getOverallStatistics();

  console.log("here");
  addTableTo(document.getElementById("tables"), overallStats, "Overall Stats");


  for (const suite of getSuites()) {
    const tableData = tableForSuite(suite);
    addTableTo(document.getElementById("tables"), tableData, suite + " Stats");
  }


  

  renderWarnings();
  refreshChart();
  refreshLatexMacros();
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
    checkbox.onchange = () => toggleCheckbox(mode, GLOBAL_DATA.checkedModes);
  });

  const suites = getSuites();
  suites.forEach((suite) => {
    const checkbox = makeCheckbox(
      document.getElementById("suiteCheckboxes"),
      suite,
    );
    checkbox.onchange = () => toggleCheckbox(suite, GLOBAL_DATA.checkedSuites);
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
      toggleCheckbox(benchmark, GLOBAL_DATA.checkedBenchmarks);
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

async function refreshLatexMacros() {
  const latexMacrosTextArea = document.getElementById("latex-macros-text");
  const latexMacros = await fetch("nightlymacros.tex").then((r) => r.text());
  latexMacrosTextArea.value = latexMacros;
}

function addGraphs() {
  console.log("addgraphs");
  var prevElement = document.getElementById("plots");
  // for each plot in graphs folder, add button to show plot
  fetch("graphs.json")
    .then((r) => r.json())
    .then((data) => {
      data.forEach((plot) => {
        const button = document.createElement("button");
        button.id = plot;
        button.onclick = function () {
          toggle(button, `\u25B6 Show ${plot}`, `\u25BC Hide ${plot}`);
        };
        button.innerText = `\u25B6 Show ${plot}`;

        // insert right after plots element
        prevElement.insertAdjacentElement("afterend", button);
        prevElement = button;

        // create div for plot
        const plotDiv = document.createElement("div");
        plotDiv.classList.add("content");
        plotDiv.classList.add("collapsed");
        plotDiv.id = `${plot}-content`;
        prevElement.insertAdjacentElement("afterend", plotDiv);
        prevElement = plotDiv;

        // create img for plot
        const img = document.createElement("img");
        img.src = `graphs/${plot}`;
        plotDiv.appendChild(img);
      });
    });
}

// on page load, add graphs
window.addEventListener("load", addGraphs);
