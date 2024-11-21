// Load data that both the index page and llvm page need
async function loadCommonData() {
  GLOBAL_DATA.currentRun = await fetchJson("./data/profile.json");
}

// Top-level load function for the main index page.
async function load_index() {
  await loadCommonData();
  makeSelectors();

  // Everything selected by default
  selectAllModes(true);
  selectBenchmarks("all");

  // Firefox doesn't handle radio buttons correctly on page reload,
  // so manually set to absolute view
  document.getElementById("absolute").checked = true;
  onRadioClick("absolute");

  const previousRuns = await getPreviousRuns();
  const initialRunIdx = findBenchToCompareIdx(previousRuns);
  loadBaseline(previousRuns[initialRunIdx].url);

  buildNightlyDropdown("comparison", previousRuns, initialRunIdx);

  refreshView();
  initializeChart();
}

// Top-level load function for the llvm page
async function load_llvm() {
  const params = new URLSearchParams(window.location.search);
  const benchmark = params.get("benchmark");
  const runMode = params.get("runmode");

  document.title = `${benchmark} | ${runMode}`;
  document.getElementById(
    "llvm-header"
  ).innerText = `Benchmark: ${benchmark} | Run Mode: ${runMode}`;

  if (!benchmark || !runMode) {
    console.error("missing query params, this probably shouldn't happen");
    return;
  }
  showIR(benchmark, runMode);
  showCFGs(benchmark, runMode);
}

async function load_table() {
  await loadCommonData();
  const params = new URLSearchParams(window.location.search);
  const table = params.get("table");
  document.getElementById("table-header").innerText = table;
  const tex = await fetchText(`./data/${table}.tex`);
  document.getElementById("table").innerText = tex;
  document.getElementById("table-pdf").href = `./data/${table}.pdf`;
}

function selectAllModes(enabled) {
  const checkboxContainer = document.getElementById("modeCheckboxes");
  Array.from(checkboxContainer.getElementsByTagName("input")).forEach(
    (checkbox) => {
      checkbox.checked = enabled;
      enabled
        ? GLOBAL_DATA.enabledModes.add(checkbox.id)
        : GLOBAL_DATA.enabledModes.delete(checkbox.id);
    }
  );
  refreshView();
}

function selectBenchmarks(category) {
  const checkboxContainer = document.getElementById("benchmarkCheckboxes");
  let checkboxes = Array.from(checkboxContainer.getElementsByTagName("input"));
  const benches = [...new Set(GLOBAL_DATA.currentRun.map((x) => x.benchmark))];
  // For sorting by time, just take the max of the cycles across all run modes
  const benchmarkSpeeds = benches.map((b) => ({
    bench: b,
    time: Math.max(
      ...GLOBAL_DATA.currentRun
        .filter((x) => x.benchmark === b)
        .map((x) => Math.max(...x.cycles))
    ),
  }));
  switch (category) {
    case "all":
      checkboxes.forEach((c) => (c.checked = true));
      break;
    case "none":
      checkboxes.forEach((c) => (c.checked = false));
      break;
    case "fast":
      const fastest = benchmarkSpeeds
        .sort((a, b) => a.time - b.time)
        .slice(0, 5)
        .map((x) => x.bench);
      checkboxes.forEach((c) => (c.checked = fastest.includes(c.id)));
      break;
    case "slow":
      const slowest = benchmarkSpeeds
        .sort((a, b) => b.time - a.time)
        .slice(0, 5)
        .map((x) => x.bench);
      checkboxes.forEach((c) => (c.checked = slowest.includes(c.id)));
      break;
  }
  checkboxes.forEach((checkbox) => {
    checkbox.checked
      ? GLOBAL_DATA.enabledBenchmarks.add(checkbox.id)
      : GLOBAL_DATA.enabledBenchmarks.delete(checkbox.id);
  });
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
  GLOBAL_DATA.baselineRun = await fetchJson(`${url}/data/profile.json`);

  const baselineBenchmarks = new Set(
    GLOBAL_DATA.baselineRun.map((o) => o.benchmark)
  );
  const currentBenchmarks = new Set(
    GLOBAL_DATA.currentRun.map((o) => o.benchmark)
  );
  baselineBenchmarks.difference(currentBenchmarks).forEach((benchmark) => {
    addWarning(
      `Baseline run had benchmark ${benchmark} that the current run doesn't`
    );
  });

  refreshView();
}

function onRadioClick(value) {
  GLOBAL_DATA.chart.mode = value;
  document.getElementById("speedup-formula").style.visibility =
    value === "speedup" ? "visible" : "hidden";
  refreshChart();
}

function copyToClipboard(eltId) {
  navigator.clipboard.writeText(document.getElementById(eltId).innerText);
}

function expand(elt, labelElt, text) {
  elt.classList.add("expanded");
  elt.classList.remove("collapsed");
  labelElt.innerText = text;
}

function collapse(elt, labelElt, text) {
  elt.classList.add("collapsed");
  elt.classList.remove("expanded");
  labelElt.innerText = text;
}

function toggle(elt, showText, hideText) {
  const content = elt.nextElementSibling;
  if (content.classList.contains("expanded")) {
    collapse(content, elt, showText);
  } else {
    expand(content, elt, hideText);
  }
}

function toggleAllPngs(elt) {
  const btns = Array.from(document.getElementsByClassName("pngToggle"));

  if (elt.innerText == "Expand All") {
    elt.innerText = "Collapse All";
    btns.forEach((btn) => {
      const txt = btn.innerText.replace("\u25B6 Show", "\u25BC Hide");
      const content = btn.nextElementSibling;
      expand(content, btn, txt);
    });
  } else {
    elt.innerText = "Expand All";
    btns.forEach((btn) => {
      const txt = btn.innerText.replace("\u25BC Hide", "\u25B6 Show");
      const content = btn.nextElementSibling;
      collapse(content, btn, txt);
    });
  }
}
