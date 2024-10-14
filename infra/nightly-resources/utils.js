// If v is a number, rounds it to the specified precision.
// If v is not a number, returns it unchanged
function tryRound(v, precision) {
  if (typeof v == "number") {
    if (v % 1 == 0) {
      return v;
    } else {
      return v.toFixed(precision || 3);
    }
  } else {
    return v;
  }
}

// Outputs current_number - baseline_number in a human-readable format
// If baseline_number is undefined, it will return N/A
function getDifference(current, baseline, comparison_func) {
  const THRESHOLD = 0.01;
  // if b is undefined, return a
  if (baseline === undefined) {
    return { class: "", value: "N/A" };
  } else {
    var difference = comparison_func(current) - comparison_func(baseline);
    // if the difference is negative it will already have a "-"
    var sign = difference < 0 ? "" : "+";
    var cssClass = "";
    if (difference < -THRESHOLD) {
      cssClass = "good";
    } else if (difference > THRESHOLD) {
      cssClass = "bad";
    }
    // put the difference in parens after a
    return { class: cssClass, value: `${sign}${tryRound(difference)}` };
  }
}

