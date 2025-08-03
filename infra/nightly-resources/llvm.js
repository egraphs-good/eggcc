async function showIR(benchmark, runMode) {
  const llvm = await fetchText(
    `./data/llvm/${benchmark}/${runMode}/optimized.ll`,
  );
  document.getElementById("llvm-ir").innerText = llvm;
}

async function showCFGs(benchmark, runMode) {
  let svgs = (
    await fetchText(`./data/llvm/${benchmark}/${runMode}/svg_names.txt`)
  ).split("\n");

  // Move main.svg and _main.svg to top
  const _main = "_main.svg";
  if (svgs.includes(_main)) {
    svgs = svgs.filter((x) => x !== _main);
    svgs.unshift(_main);
  }
  const main = "main.svg";
  if (svgs.includes(main)) {
    svgs = svgs.filter((x) => x !== main);
    svgs.unshift(main);
  }

  const svgContainer = document.getElementById("llvm-cfg");
  svgs.forEach((svg) => {
    const elt = document.createElement("div");

    const btn = document.createElement("button");
    btn.innerText = `\u25B6 Show ${svg}`;
    btn.classList.add("collapsible");
    btn.classList.add("svgToggle");
    btn.onclick = (elt) =>
      toggle(elt.target, `\u25B6 Show ${svg}`, `\u25BC Hide ${svg}`);

    elt.appendChild(btn);

    const img = document.createElement("img");
    img.classList.add("cfg");
    img.classList.add("collapsed");
    img.src = `data/llvm/${benchmark}/${runMode}/${svg}`;
    elt.appendChild(img);

    svgContainer.appendChild(elt);
  });
}
