// Load data that both the index page and llvm page need
async function loadCommonData() {
  GLOBAL_DATA.currentRun = await fetchDataJson(".");
}

// Top-level load function for the main index page.
async function load_index() {
  await loadCommonData();
  makeSelectors();

  // Everything selected by default
  selectAllModes(true);
  selectAllBenchmarks(true);

  const previousRuns = await getPreviousRuns();
  const initialRunIdx = findBenchToCompareIdx(previousRuns);
  loadBaseline(previousRuns[initialRunIdx].url);

  buildNightlyDropdown("comparison", previousRuns, initialRunIdx);

  refreshView();
  initializeChart();
}

// Top-level load function for the llvm page
async function load_llvm() {
  await loadCommonData();
  const params = new URLSearchParams(window.location.search);
  const benchmark = params.get("benchmark");
  const runMode = params.get("runmode");
  if (!benchmark || !runMode) {
    console.error("missing query params, this probably shouldn't happen");
  }
  const entry = GLOBAL_DATA.currentRun.filter(
    (entry) => entry.benchmark === benchmark && entry.runMethod === runMode,
  );
  if (entry.length !== 1) {
    console.error(
      `missing or duplicate entries for ${benchmark} and ${runMode}, this probably shouldn't happen`,
    );
  }
  document.getElementById("llvm").innerText = entry[0].llvm_ir;
}

function selectAllModes(enabled) {
  const checkboxContainer = document.getElementById("modeCheckboxes");
  Array.from(checkboxContainer.getElementsByTagName("input")).forEach(
    (checkbox) => {
      checkbox.checked = enabled;
      enabled
        ? GLOBAL_DATA.enabledModes.add(checkbox.id)
        : GLOBAL_DATA.enabledModes.delete(checkbox.id);
    },
  );
  refreshView();
}

function selectAllBenchmarks(enabled) {
  const checkboxContainer = document.getElementById("benchmarkCheckboxes");
  Array.from(checkboxContainer.getElementsByTagName("input")).forEach(
    (checkbox) => {
      checkbox.checked = enabled;
      enabled
        ? GLOBAL_DATA.enabledBenchmarks.add(checkbox.id)
        : GLOBAL_DATA.enabledBenchmarks.delete(checkbox.id);
    },
  );
  refreshView();
}

function toggleCheckbox(mode, set) {
  if (set.has(mode)) {
    set.delete(mode);
  } else {
    set.add(mode);
  }
  refreshView();
}

async function loadBaseline(url) {
  clearWarnings();
  GLOBAL_DATA.baselineRun = await fetchDataJson(url);

  const baselineBenchmarks = new Set(
    GLOBAL_DATA.baselineRun.map((o) => o.benchmark),
  );
  const currentBenchmarks = new Set(
    GLOBAL_DATA.currentRun.map((o) => o.benchmark),
  );
  baselineBenchmarks.difference(currentBenchmarks).forEach((benchmark) => {
    addWarning(
      `Baseline run had benchmark ${benchmark} that the current run doesn't`,
    );
  });

  refreshView();
}

function toggleWarnings() {
  const elt = document.getElementById("warnings-toggle");
  elt.classList.toggle("active");
  const content = elt.nextElementSibling;
  if (content.style.display === "block") {
    elt.innerText = `Show ${GLOBAL_DATA.warnings.size} Warnings`;
    content.style.display = "none";
  } else {
    elt.innerText = `Hide ${GLOBAL_DATA.warnings.size} Warnings`;
    content.style.display = "block";
  }
}

function toggleChart() {
  const elt = document.getElementById("chart-toggle");
  elt.classList.toggle("active");
  const content = elt.nextElementSibling;
  if (content.style.display === "block") {
    elt.innerText = "Show Chart";
    content.style.display = "none";
  } else {
    elt.innerText = "Hide Chart";
    content.style.display = "block";
  }
}
