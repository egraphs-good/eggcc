// Top-level load function for the main index page.
function load() {
  // data should be in the format of Array<{runMethod: String, benchmark: String, results: {total_dyn_inst: Int, hyperfine: Array<{results: **hyperfine `--json` results**}>}}>
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
          let results = b.results.hyperfine.results[0];
          toParse["Executions "].data.push({
            name: b.runMethod,
            "# Instructions": b.results.total_dyn_inst,
            min: tryRound(results.min),
            max: tryRound(results.max),
            mean: tryRound(results.mean),
            median: tryRound(results.median),
            stddev: tryRound(results.stddev),
          });
        });

        parsed.push(toParse);
      });

  container.innerHTML = ConvertJsonToTable(parsed);
}
