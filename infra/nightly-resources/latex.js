// format a number to 2 decimal places as a string
// but only if it is a number
function formatNumber(num) {
  if (typeof num === "number") {
    return num.toFixed(2);
  }
  return num;
}

function jsonHeaders(json) {
  var headers = [];

  for (var i = 0; i < json.length; i++) {
    for (var key in json[i]) {
      if (headers.indexOf(key) == -1) {
        headers.push(key);
      }
    }
  }

  return headers;
}

function jsonColumns(json, rowIndex) {
  var cols = [];

  for (var i = 0; i < json.length; i++) {
    cols.push(json[i][rowIndex]);

    // throw an error if undefined
    if (json[i][rowIndex] === undefined) {
      throw new Error("undefined value for " + rowIndex + " in row " + i);
    }
  }

  return cols;
}

// convert a javascript array object to a latex table
// each element of the array is a dictionary associating a header with a value
// example element: { geoMeanNormalized : "0.619", meanEggccCompileTimeSecs : "0.001", meanLlvmCompileTimeSecs : "0.447",
// runMethod : "llvm-O1-O0" }
function jsonToLatexTable(json) {
  console.log(json);
  var res = "\\begin{tabular}{|";
  var header = "";
  var rows = json.length;
  var cols = 0;

  var headers = jsonHeaders(json);

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

// change dashes to underscores
function convertStringToValidLatexVar(str) {
  return str.replace(/-/g, "_");
}

// convert a javascript array object storing a table
// to a bunch of latex macros, one for each row, column pair
// each element of the array is a dictionary associating a header with a value
// example element: { geoMeanNormalized : "0.619", meanEggccCompileTimeSecs : "0.001", meanLlvmCompileTimeSecs : "0.447",
// runMethod : "llvm-O1-O0" }
// rowIndex is the name of the header of the left column, in this case "runMethod"
function jsonToLatexMacros(json, rowIndex) {
  var names = [];
  var res = "";

  // get the headers
  var headers = jsonHeaders(json);
  var cols = jsonColumns(json, rowIndex);

  // for each header, col pair create a macro
  for (var i = 0; i < cols.length; i++) {
    for (var j = 0; j < headers.length; j++) {
      var name = `${convertStringToValidLatexVar(cols[i])}${convertStringToValidLatexVar(headers[j])}`;
      res +=
        "\\newcommand{\\" +
        name +
        "}{" +
        formatNumber(json[i][headers[j]]) +
        "}\n";

      names.push(name);
    }
  }

  // if there are duplicate names then throw an error
  var uniqueNames = new Set(names);
  if (uniqueNames.size !== names.length) {
    throw new Error("Duplicate names in jsonToLatexMacros");
  }

  return res;
}
