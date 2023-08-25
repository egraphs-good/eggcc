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
