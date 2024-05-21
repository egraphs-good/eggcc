function generateLineCountTable(latexData) {
  const writtenEggTotal = Object.values(latexData.written_egg).reduce((acc, v) => acc + v);

  return String.raw`
  \begin{tabular}{ |l|l| }
  \hline
  \multicolumn{2}{|c|}{Line Counts} \\
  \hline
  Language & \# Lines  \\
  \hline
  Rust & ${latexData.rust} \\
  Written Egg & ${writtenEggTotal} \\
  Generated EGG & ${latexData.gen_egg} \\
  \hline
  \end{tabular}
  `
}

function generateDetailedLineCountTable(latexData) {
  const pre = String.raw`
  \begin{tabular}{ |l|l| }
  \hline
  \multicolumn{2}{|c|}{Egglog Line Counts} \\
  \hline
  File & \# Lines  \\`;
  const rows = Object.keys(latexData.written_egg).map(filename => {
    const escapedFilename = filename.replaceAll("_", "\\_");
    const lineCount = latexData.written_egg[filename];
    return String.raw`\hline ${escapedFilename} & ${lineCount} \\`
  });
  const post = String.raw`
  \hline
  \end{tabular}`;
  return String.raw`
  ${pre}
  ${rows.join('\n')}
  ${post}`;
}

function generateBenchmarksTable(latexData) {
  console.log(latexData)
}

function generateLatex(latexData, tableName) {
  switch (tableName) {
    case "linecount":
      return generateLineCountTable(latexData);
    case "detailed-linecount":
      return generateDetailedLineCountTable(latexData);
    case "benchmarks":
      return generateBenchmarksTable(latexData);
    default:
      console.error(`unknown table type: ${tableName}`);
  }
}