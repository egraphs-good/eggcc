async function getComparisons() {
    const req = await fetch("../");
    const html = await req.text();

    const parser = new DOMParser();
    const htmlDoc = parser.parseFromString(html, 'text/html');

    const allLinks = htmlDoc.getElementsByTagName("a");

    const comparisons = [];
    for (let i = 1; i < allLinks.length; i++) {
        const hrefText = allLinks[i].getAttribute("href");

        const [date, _, branch, commit] = hrefText.split("%3A");
        comparisons.push({branch: branch, commit: commit.slice(0, -1), date: +date, url: allLinks[i].href});
    }

    comparisons.sort((l, r) => {
        if (l.date < r.date) {
            return -1;
        }
        if (l.date > r.date) {
            return 1;
        }
        return 0;
    });

    return comparisons;
}

async function buildNightlyDropdown(element, comparisons) {
    const select = document.getElementById(element);
    for (let i = 0; i < comparisons.length; i++) {
        const nightly = comparisons[i];
        console.log(nightly);

        const option = document.createElement("option");
        option.value = nightly.commit;
        option.innerText = `${nightly.branch} - ${nightly.commit}`
        select.appendChild(option);
    }
}

// getLastBench will find the last main benchmark that is not itself
function findBenchToCompare(comparisons) {
    // Determine what benchmark run we are actually running based on our URL
    // This is likely the best way to do this without embedding a bunch of data into our profile.js file
    // or our profile.json file, which although tempting, is not backwards compatible
    const path = window.location.pathname;
    const parts = path.split("/");

    // URLs should have trailing slashes leaving `parts` with a blank last element,
    // so we should index into `parts` at its length-2
    // Just in case the URL somehow doesn't have a trailing slash and `parts` doesn't
    // have a blank last element, do a quick check and adjust the index accordingly
    const idx = path[path.length - 1] === "/" ? parts.length - 1 : parts.length - 2;


    const [date, _, branch, commit] = parts[idx].split("%3A");
    for (let i = comparisons.length; i >= 0; i--) {
        const curComparison = comparisons[i];
        if (curComparison === "main") {
            // yes, I did mean `==` here. `curComparison.date` is an int, and `date` is a string
            if (curComparison.commit === commit && curComparison.date == date) {
                continue;
            }
            return curComparison;
        }
    }
    throw new Error("Didn't find a candidate benchmark");
}


async function getBenchToCompare(comparisons) {
    const bench = findBenchToCompare(comparisons);
    const resp = await fetch(bench.url + "data/profile.json");
    const benchData = await resp.json();
    return indexBenchmarks(benchData)
}

function groupByBenchmark(benchList) {
    const groupedBy = {};
    benchList.forEach((obj) => {
        if (!groupedBy[obj.benchmark]) {
            groupedBy[obj.benchmark] = [];
        }
        groupedBy[obj.benchmark].push(obj);
    });
    return groupedBy;
}

function indexBenchmarks(benchmarkList) {
    const runsByBench = groupByBenchmark(benchmarkList);
    const benchNames = Object.keys(runsByBench);
    return benchNames.reduce(
        (acc, benchName) => {
            return {
                ...acc,
                [benchName]: runsByBench[benchName].reduce(
                    (obj, run) => {
                        return {
                            ...obj, [run.runMethod]: run
                        }
                    }, {})
            }
        }, {});
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

function buildTableText(prevRun) {
    return (run) => {
        const entry = buildEntry(run)
        if (!prevRun) { return entry; }

        const prevEntry= buildEntry(prevRun);
        compareKeys.forEach((key) => {
            const diff = Math.abs(entry[key] - prevEntry[key]);
            if (diff === 0) {
                return;
            }
            
            const sign = entry[key] < prevEntry[key] ? "-" : "+"
            entry[key] = `${entry[key]} (${sign}${diff})`
        })
    }
}

async function loadBenchmarks(comparisons) {
    // data should be in the format of Array<{runMethod: String, benchmark: String, total_dyn_inst: Int, hyperfine: Array<{results: **hyperfine `--json` results**}>}>
    console.log(data);

    let container = document.getElementById("profile");

    // aggregate benchmark runs into a list by their "benchmark" key
    const currentRun = indexBenchmarks(data);
    const previousRun = await getBenchToCompare(comparisons);

    const benchmarkNames = Object.keys(currentRun);

    const parsed = benchmarkNames.map((benchName) => {
        return {
            name: benchName,
            "Executions ": {
                data: currentRun[benchName].map(buildTableText(previousRun[benchName]))
            }
        }
    });

    parsed.sort((l, r) => {
        if (l.name < r.name) { return -1; }
        if (l.name > r.name) { return 1; }
        return 0;
    });
    
    container.innerHTML = ConvertJsonToTable(parsed);
}


// Top-level load function for the main index page.
async function load() {
    const comparisons = await getComparisons();

    buildNightlyDropdown("comparison", comparisons)
    loadBenchmarks(comparisons);
}
