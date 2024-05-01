// Top-level load function for the main index page.
async function load() {
  GLOBAL_DATA.currentRun = await getBench("./");
  makeModeSelectors();

  const previousRuns = await getPreviousRuns();
  const initialRunIdx = findBenchToCompareIdx(previousRuns);
  GLOBAL_DATA.baselineRun = await getBench(
    previousRuns[initialRunIdx].url + "/"
  );

  buildNightlyDropdown("comparison", previousRuns, initialRunIdx);

  refreshView();
}

function selectAllModes(enabled) {
  const checkboxContainer = document.getElementById("modeCheckboxes");
  checkboxContainer.childNodes.forEach((checkbox) => {
    checkbox.checked = enabled;
    enabled
      ? GLOBAL_DATA.enabledModes.add(checkbox.id)
      : GLOBAL_DATA.enabledModes.delete(checkbox.id);
  });
  refreshView();
}

function toggleCheckbox(mode) {
  if (GLOBAL_DATA.enabledModes.has(mode)) {
    GLOBAL_DATA.enabledModes.delete(mode);
  } else {
    GLOBAL_DATA.enabledModes.add(mode);
  }
  refreshView();
}
