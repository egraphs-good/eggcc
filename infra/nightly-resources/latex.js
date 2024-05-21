function construct(pre, rows, post) {
  return String.raw`
  ${pre}
  ${rows.join("\n")}
  ${post}`;
}

function generateLineCountTable(latexData) {
  const writtenEggTotal = Object.values(latexData.written_egg).reduce(
    (acc, v) => acc + v,
  );

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
  `;
}

function generateDetailedLineCountTable(latexData) {
  const pre = String.raw`
  \begin{tabular}{ |l|l| }
  \hline
  \multicolumn{2}{|c|}{Egglog Line Counts} \\
  \hline
  File & \# Lines  \\`;
  const rows = Object.keys(latexData.written_egg).map((filename) => {
    const escapedFilename = filename.replaceAll("_", "\\_");
    const lineCount = latexData.written_egg[filename];
    return String.raw`\hline ${escapedFilename} & ${lineCount} \\`;
  });
  const post = String.raw`
  \hline
  \end{tabular}`;
  return construct(pre, rows, post);
}

function generateRowsForBench(benchmark) {
  const dataForBench = GLOBAL_DATA.currentRun.filter(x => x.benchmark === benchmark);
  const rows = dataForBench.map((entry, idx) => {
    const {mean, max, min, stddev} = entry.hyperfine.results[0];
    const fstCol = idx === 0 ? String.raw`\multirow{${dataForBench.length}}{*}{${benchmark}}` : '';
    return [
      String.raw`\multicolumn{1}{|l|}{${fstCol}} &`,
      String.raw`\multicolumn{1}{l|}{${entry.runMethod}}  &`,
      String.raw`\multicolumn{1}{l|}{${tryRound(mean)}} &`,
      String.raw`\multicolumn{1}{l|}{${tryRound(max)}} &`,
      String.raw`\multicolumn{1}{l|}{${tryRound(min)}} &`,
      String.raw`${tryRound(stddev)} \\`
    ].join(" ");
  });
  return construct("", rows, String.raw` \hline`);
}

function generateBenchmarksTable(latexData) {
  const pre = String.raw`
  \begin{table}[]
  \resizebox{\textwidth}{!}{%
  \begin{tabular}{|llllll|}
  \hline
  \multicolumn{6}{|c|}{Benchmarks} \\ \hline
  \multicolumn{1}{|l|}{Name} & \multicolumn{5}{c|}{Executions} \\ \hline
  \multicolumn{1}{|l|}{} & \multicolumn{1}{l|}{Run Method}  & \multicolumn{1}{l|}{Mean} & \multicolumn{1}{l|}{Max} & \multicolumn{1}{l|}{Min} & Std Dev \\ \hline`

  const benchmarks = [...new Set(GLOBAL_DATA.currentRun.map(x => x.benchmark))].sort();
  const rows = benchmarks.map(bench => generateRowsForBench(bench));

  const post = String.raw`\end{tabular}%
  }
  \end{table}`
  return construct(pre, rows, post);
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
