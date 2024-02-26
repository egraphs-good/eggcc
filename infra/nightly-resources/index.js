async function getPreviousRuns() {
    const req = await fetch("https://nightly.cs.washington.edu/reports-json/eggcc/");
    const files = await req.json();
    
    // map files into objects of the shape:
    // {branch: <git branch:string>, commit: <git commit:string>, date: <unix timestamp:int>, url: <absolute url to nightly page:string>}
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
        }
        
        comparisons.push(run);
    }

    // sort runs in descending order
    comparisons.sort((l, r) => {
        if (l.date < r.date) { return 1; }
        if (l.date > r.date) { return -1; }
        return 0;
    });

    return comparisons.slice(0, comparisons.length < 30 ? comparisons.length : 30);
}

async function buildNightlyDropdown(element, previousRuns, initialIdx) {
    const select = document.getElementById(element);
    
    const formatRun = (run) => `${run.branch} - ${run.commit} - ${(new Date(run.date * 1000)).toDateString()}`
    
    
    previousRuns.forEach((nightly) => {
        console.log(nightly);

        const option = document.createElement("option");
        option.innerText = formatRun(nightly);
        select.appendChild(option);
    });
    
    select.onchange = () => {
        const compareTo = previousRuns[select.selectedIndex];
        loadBenchmarks(compareTo);
    }

    select.selectedIndex = initialIdx;
    select.value = formatRun(previousRuns[initialIdx]);
    select.onchange();
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
    const idx = path[path.length - 1] === "/" ? parts.length - 2 : parts.length - 1;
    
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
    return groupByBenchmark(benchData)
}

// benchList should be in the format of Array<{runMethod: String, benchmark: String, total_dyn_inst: Int, hyperfine: Array<{results: **hyperfine `--json` results**}>}>
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

const compareKeys = ["# Instructions"];
function buildEntry(run) {
    const results = run.hyperfine.results[0];
    return {
        name: run.runMethod,
        "# Instructions": run.total_dyn_inst,
        min: tryRound(results.min),
        max: tryRound(results.max),
        mean: tryRound(results.mean),
        median: tryRound(results.median),
        stddev: tryRound(results.stddev),
    }
}

function buildTableText(prevRun, run) {
    const entry = buildEntry(run)
    if (!prevRun) {
        return entry;
    }

    const prevEntry = buildEntry(prevRun);
    compareKeys.forEach((key) => {
        const diff = Math.abs(entry[key] - prevEntry[key]);
        if (diff === 0) {
            return;
        }

        const sign = entry[key] < prevEntry[key] ? "-" : "+"
        entry[key] = `${entry[key]} (${sign}${diff})`
    })
    return entry;
}

async function loadBenchmarks(compareTo) {
    const currentRun = await getBench("../");
    let previousRun = undefined;
    try {
        previousRun = await getBench(compareTo.url+"/");
    } catch (e) {}

    const benchmarkNames = Object.keys(currentRun);

    const parsed = benchmarkNames.map((benchName) => {
        return {
            name: benchName,
            "Executions ": {
                data: Object
                    .keys(currentRun[benchName])
                    .map((runMethod) => {
                        const prevBenchmark = previousRun ? previousRun[benchName] : undefined;
                        const prevRun = prevBenchmark ? prevBenchmark[runMethod] : undefined;
                        
                        return buildTableText(prevRun, currentRun[benchName][runMethod]) 
                    })
            }
        }
    });

    parsed.sort((l, r) => {
        if (l.name < r.name) { return -1; }
        if (l.name > r.name) { return 1; }
        return 0;
    });

    let container = document.getElementById("profile");
    container.innerHTML = ConvertJsonToTable(parsed);
}

// Top-level load function for the main index page.
async function load() {
    const previousRuns = await getPreviousRuns();
    const initialRunIdx = findBenchToCompareIdx(previousRuns);
    
    buildNightlyDropdown("comparison", previousRuns, initialRunIdx)
}
