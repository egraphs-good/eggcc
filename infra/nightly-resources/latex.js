// format a number to 2 decimal places as a string
// but only if it is a number
function formatNumber(num) {
  if (typeof num === "number") {
    return num.toFixed(2);
  }
  return num;
}

// convert a javascript array object to a latex table
function jsonToLatexTable(json) {
  var res = "\\begin{tabular}{|";
  var headers = [];
  var header = "";
  var rows = json.length;
  var cols = 0;

  // get the headers
  for (var i = 0; i < rows; i++) {
    for (var key in json[i]) {
      if (headers.indexOf(key) == -1) {
        headers.push(key);
      }
    }
  }

  // create the header
  cols = headers.length;
  for (var i = 0; i < cols; i++) {
    header += "r|";
  }

  res += header + "}\\hline\n";

  // create the header row
  for (var i = 0; i < cols; i++) {
    res += headers[i];
    if (i < cols - 1) {
      res += " & ";
    }
  }

  res += " \\\\ \\hline\n";

  // create the body
  for (var i = 0; i < rows; i++) {
    for (var j = 0; j < cols; j++) {
      var value = json[i][headers[j]];
      res += formatNumber(value);
      if (j < cols - 1) {
        res += " & ";
      }
    }
    res += " \\\\ \\hline\n";
  }

  res += "\\end{tabular}";

  return res;
}
