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
  const llvm = await fetchText(`./data/llvm/${benchmark}/${runMode}/llvm.ll`);
  document.getElementById("llvm-ir").innerText = llvm;
  let pngs = (
    await fetchText(`./data/llvm/${benchmark}/${runMode}/png_names.txt`)
  ).split("\n");

  // Move main.png and _main.png to top
  const _main = "_main.png";
  if (pngs.includes(_main)) {
    pngs = pngs.filter((x) => x !== _main);
    pngs.unshift(_main);
  }
  const main = "main.png";
  if (pngs.includes(main)) {
    pngs = pngs.filter((x) => x !== main);
    pngs.unshift(main);
  }

  const pngContainer = document.getElementById("llvm-cfg");
  pngs.forEach((png) => {
    const elt = document.createElement("div");

    const img = document.createElement("img");
    img.className = "cfg";
    img.src = `data/llvm/${benchmark}/${runMode}/${png}`;
    elt.appendChild(img);

    const p = document.createElement("p");
    p.innerText = png;
    elt.appendChild(p);

    pngContainer.appendChild(elt);
  });
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
  GLOBAL_DATA.baselineRun = await fetchJson(`${url}/data/profile.json`);

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

function onRadioClick(elt) {
  GLOBAL_DATA.chart.mode = elt.value;
  document.getElementById("speedup-formula").style.visibility =
    elt.value === "speedup" ? "visible" : "hidden";
  refreshChart();
}

function copyToClipboard(eltId) {
  navigator.clipboard.writeText(document.getElementById(eltId).innerText);
}
