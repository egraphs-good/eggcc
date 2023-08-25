// Top-level load function for the main index page.
function load() {
  console.log(data);
  let container = document.getElementById("profile");
  let parsed = [];
  data.forEach((obj) => {
    let names = Object.keys(obj);
    names.forEach((name) => {
      let results = obj[name].hyperfine.results[0];
      parsed.push({
        name,
        "# Instructions": obj[name]["total_dyn_inst:"],
        min: tryRound(results.min),
        max: tryRound(results.max),
        mean: tryRound(results.mean),
        median: tryRound(results.median),
        stddev: tryRound(results.stddev),
      });
    });
  });
  container.innerHTML = ConvertJsonToTable(parsed);
}
