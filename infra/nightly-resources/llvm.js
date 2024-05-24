async function showIR(benchmark, runMode) {
  const llvm = await fetchText(`./data/llvm/${benchmark}/${runMode}/llvm.ll`);
  document.getElementById("llvm-ir").innerText = llvm;
}

async function showCFGs(benchmark, runMode) {
  let pngs = (
    await fetchText(`./data/llvm/${benchmark}/${runMode}/png_names.txt`)
  ).split("\n");

  let d = await fetchJson(`./data/llvm/${benchmark}/${runMode}/`);
  debugger;

  // Move main.png and _main.png to top
  const _main = "_main.png";
  if (pngs.includes(_main)) {
    pngs = pngs.filter((x) => x !== _main);
    pngs.unshift(_main);
  }
  const main = "main.png";
  if (pngs.includes(main)) {
    pngs = pngs.filter((x) => x !== main);
    pngs.unshift(main);
  }

  const pngContainer = document.getElementById("llvm-cfg");
  pngs.forEach((png) => {
    const elt = document.createElement("div");

    const btn = document.createElement("button");
    btn.innerText = `\u25B6 Show ${png}`;
    btn.classList.add("collapsible");
    btn.classList.add("pngToggle");
    btn.onclick = (elt) =>
      toggle(elt.target, `\u25B6 Show ${png}`, `\u25BC Hide ${png}`);

    elt.appendChild(btn);

    const img = document.createElement("img");
    img.className = "cfg";
    img.src = `data/llvm/${benchmark}/${runMode}/${png}`;
    img.style.display = "none";
    elt.appendChild(img);

    pngContainer.appendChild(elt);
  });
}
