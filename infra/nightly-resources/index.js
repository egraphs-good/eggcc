// copied from profile.py
const treatments = [
  "rvsdg_roundtrip",

  "cranelift-O0",
  "cranelift-O0-eggcc",
  "cranelift-O3",
  "cranelift-O3-eggcc",

  "llvm-peep",
  "llvm-peep-eggcc",
  "llvm-O3",
  "llvm-O3-eggcc",
];

const GLOBAL_DATA = {
  enabledModes: new Set(),
  enabledBenchmarks: new Set(),
  warnings: new Set(),
  currentRun: {},
  baselineRun: undefined,
  chart: undefined,
};

function addWarning(warning) {
  GLOBAL_DATA.warnings.add(warning);
}

function clearWarnings() {
  GLOBAL_DATA.warnings.clear();
}

async function getPreviousRuns() {
  const req = await fetch(
    "https://nightly.cs.washington.edu/reports-json/eggcc/",
  );
  const files = await req.json();

  // map files into objects of the shape:
  // {
  //   branch: <git branch:string>,
  //   commit: <git commit:string>,
  //   date: <unix timestamp:int>,
  //   url: <absolute url to nightly page:string>
  // }
  const comparisons = [];
  // start at i=1 because / is the first file
  for (let i = 1; i < files.length; i++) {
    // file name is of the format <date>:"nightly":<branch>:<commit>
    const [date, _, branch, commit] = files[i].name.split(":");

    const run = {
      branch: branch,
      commit: commit,
      // type coerce a unix timestamp that is a string into an int with a `+`
      date: +date,
      // The file server only hands us back the directory names,
      // but we want to make sure that we only use absolute URLs so that we can run this page
      // in any environment (local or otherwise)
      url: `https://nightly.cs.washington.edu/reports/eggcc/${files[i].name}`,
    };

    comparisons.push(run);
  }

  // sort runs in descending order
  comparisons.sort((l, r) => {
    if (l.date < r.date) {
      return 1;
    }
    if (l.date > r.date) {
      return -1;
    }
    return 0;
  });

  return comparisons.slice(
    0,
    comparisons.length < 30 ? comparisons.length : 30,
  );
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

// findBenchToCompare will find the last benchmark run on the main branch that is not itself
function findBenchToCompareIdx(benchRuns) {
  // Determine what benchmark run we are based on the browser's URL
  // This is likely the best way to do this without embedding a bunch of data into our profile.js file
  // or our profile.json file, which although tempting, is not backwards compatible
  const path = window.location.pathname;
  const parts = path.split("/");

  // URLs should have trailing slashes leaving `parts` with a blank last element,
  // so we should index into `parts` at its length-2
  // Just in case the URL somehow doesn't have a trailing slash and `parts` doesn't
  // have a blank last element, do a quick check and adjust the index accordingly
  const idx =
    path[path.length - 1] === "/" ? parts.length - 2 : parts.length - 1;

  const [date, _, branch, commit] = parts[idx].split("%3A");
  for (let i = 0; i < benchRuns.length; i++) {
    const run = benchRuns[i];
    if (run.branch === "main") {
      // If we are comparing a run on a main branch, to previous main branch we need to make sure
      // it is not the same branch.
      // I did mean `==` here, not `===`. `curComparison.date` is an int, and `date` is a string
      if (branch === "main" && run.commit === commit && run.date == date) {
        continue; // skip, we're on the same branch
      }

      // the branch is now either the latest main run, or if on main the previous main run
      // return it
      return i;
    }
  }
  throw new Error("Couldn't find a benchmark run from main for comparison");
}

async function getBench(url) {
  const resp = await fetch(url + "data/profile.json");
  const benchData = await resp.json();
  return groupByBenchmark(benchData);
}

// benchList should be in the format of
// Array<{
//     runMethod: String,
//     benchmark: String,
//     total_dyn_inst: Int,
//     hyperfine: Array<{results: **hyperfine `--json` results**}>
// }>
function groupByBenchmark(benchList) {
  const groupedBy = {};
  benchList.forEach((obj) => {
    if (!groupedBy[obj.benchmark]) {
      groupedBy[obj.benchmark] = {};
    }
    groupedBy[obj.benchmark][obj.runMethod] = obj;
  });
  return groupedBy;
}

// Outputs current_number - baseline_number in a human-readable format
// If baseline_number is undefined, it will return N/A
function getDifference(current, baseline) {
  // if b is undefined, return a
  if (baseline === undefined) {
    return { class: "bad", value: "N/A" };
  } else {
    var difference = current - baseline;
    // if the difference is negative it will already have a "-"
    var sign = difference < 0 ? "" : "+";
    var cssClass = "";
    if (difference < 0) {
      cssClass = "good";
    } else if (difference > 0) {
      cssClass = "bad";
    }
    // put the difference in parens after a
    return { class: cssClass, value: `${sign}${tryRound(difference)}` };
  }
}

// compare two objects at a particular attribute
function diffAttribute(results, baseline, attribute) {
  const current = results[attribute];
  const baselineNum = baseline?.[attribute];
  return getDifference(current, baselineNum);
}

// baseline may be undefined
function buildEntry(benchName, baseline, current) {
  const results = current.hyperfine.results[0];
  const baselineResults = baseline?.hyperfine.results[0];

  var name = current.runMethod;
  if (current.llvm_ir) {
    name = `<a target="_blank" rel="noopener noreferrer" href="llvm.html?benchmark=${benchName}&runmode=${current.runMethod}">${current.runMethod}</a>`;
  }

  const result = {
    name: name,
    mean: { class: "", value: tryRound(results.mean) },
    meanVsBaseline: diffAttribute(results, baselineResults, "mean"),
    min: { class: "", value: tryRound(results.min) },
    minVsBaseline: diffAttribute(results, baselineResults, "min"),
    max: { class: "", value: tryRound(results.max) },
    maxVsBaseline: diffAttribute(results, baselineResults, "max"),
    median: { class: "", value: tryRound(results.median) },
    medianVsBaseline: diffAttribute(results, baselineResults, "median"),
    stddev: { class: "", value: tryRound(results.stddev) },
  };

  return result;
}

function refreshView() {
  const parsed = Array.from(GLOBAL_DATA.enabledBenchmarks).map((benchName) => {
    if (!GLOBAL_DATA.baselineRun) {
      addWarning("no baseline to compare to");
    }
    const executions = Object.keys(GLOBAL_DATA.currentRun[benchName])
      .map((runMode) => {
        // if the mode is not enabled, skip it
        if (!GLOBAL_DATA.enabledModes.has(runMode)) {
          return undefined;
        }
        // prevRun may be undefined
        const baselineBench = GLOBAL_DATA.baselineRun?.[benchName];
        if (GLOBAL_DATA.baselineRun && !baselineBench) {
          addWarning(`Baseline doesn't have ${benchName} benchmark`);
        }
        const baselineRunForMethod = baselineBench?.[runMode];
        if (baselineBench && !baselineRunForMethod) {
          addWarning(
            `Baseline doesn't have run mode ${runMode} for ${benchName}`,
          );
        }

        return buildEntry(
          benchName,
          baselineRunForMethod,
          GLOBAL_DATA.currentRun[benchName][runMode],
        );
      })
      .filter((e) => e !== undefined);
    if (executions.length > 1) {
      const cols = ["mean", "min", "max", "median"];
      cols.forEach((col) => {
        const sorted = executions
          .map((e) => e[col])
          .sort((a, b) => a.value - b.value);
        const min = sorted[0].value;
        const max = sorted[sorted.length - 1].value;
        sorted.forEach((item) => {
          if (item.value === min) {
            item.class = "good";
          }
          if (item.value === max) {
            item.class = "bad";
          }
        });
      });
    }

    return {
      name: benchName,
      executions: { data: executions },
    };
  });

  parsed.sort((l, r) => {
    if (l.name < r.name) {
      return -1;
    }
    if (l.name > r.name) {
      return 1;
    }
    return 0;
  });

  document.getElementById("profile").innerHTML = ConvertJsonToTable(parsed);

  renderWarnings();
  refreshChart();
}

function renderWarnings() {
  const toggle = document.getElementById("warnings-toggle");
  toggle.innerText = `Show ${GLOBAL_DATA.warnings.size} Warnings`;

  const warningContainer = document.getElementById("warnings");
  warningContainer.innerHTML = "";
  GLOBAL_DATA.warnings.forEach((warning) => {
    const warningElement = document.createElement("p");
    warningElement.innerText = warning;
    warningContainer.appendChild(warningElement);
  });
}

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

  const benchmarks = Object.keys(GLOBAL_DATA.currentRun).sort();
  benchmarks.forEach((benchmark) => {
    const checkbox = makeCheckbox(
      document.getElementById("benchmarkCheckboxes"),
      benchmark,
    );
    checkbox.onchange = () =>
      toggleCheckbox(benchmark, GLOBAL_DATA.enabledBenchmarks);
  });
}
