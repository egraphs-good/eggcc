
async function getNightlyLinks(element) {
    const req = await fetch("../");
    const html = await req.text();

    const parser = new DOMParser();
    const htmlDoc = parser.parseFromString(html, 'text/html');

    const allLinks = htmlDoc.getElementsByTagName("a");

    const comparisons = [];
    for (let i = 1; i < allLinks.length; i++) {
        const hrefText= allLinks[i].getAttribute("href");

        const [date, __, branch, commit] = hrefText.split("%3A");
        console.log(date)
        comparisons.push({branch: branch, commit: commit.slice(0, -1), date: +date});
    }

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

function loadBenchmarks() {
    // data should be in the format of Array<{runMethod: String, benchmark: String, total_dyn_inst: Int, hyperfine: Array<{results: **hyperfine `--json` results**}>}>
    console.log(data);

    let container = document.getElementById("profile");
    let parsed = [];

    // aggregate benchmark runs into a list by their "benchmark" key
    let groupedBy = {};
    data.forEach((obj) => {
        if(!groupedBy[obj.benchmark]) {
            groupedBy[obj.benchmark] = [];
        }
        groupedBy[obj.benchmark].push(obj);
    });
    let benchmarkNames = Object.keys(groupedBy);

    // for each benchmark, add a table with a name, and a subt-able of each "run method" execution
    benchmarkNames
        .forEach((benchName) => {
            let benchmark = groupedBy[benchName]
            let toParse = {name: benchName, "Executions ": {data: []}};
            benchmark.forEach((b) => {
                let results = b.hyperfine.results[0];
                toParse["Executions "].data.push({
                    name: b.runMethod,
                    "# Instructions": b.total_dyn_inst,
                    min: tryRound(results.min),
                    max: tryRound(results.max),
                    mean: tryRound(results.mean),
                    median: tryRound(results.median),
                    stddev: tryRound(results.stddev),
                });
            });

            parsed.push(toParse);
        });

    parsed.sort((l, r) => {
        if (l.name < r.name) { return -1; }
        if (l.name > r.name) { return 1; }
        return 0;
    });
    container.innerHTML = ConvertJsonToTable(parsed);
}


// Top-level load function for the main index page.
function load() {
    loadBenchmarks();
    getNightlyLinks("comparison")
}
