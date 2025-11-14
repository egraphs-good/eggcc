#!/usr/bin/env python3
"""Merge two profile.json files using (benchmark, treatment) as the key.

The script takes a base profile JSON file and an overlay profile JSON file.
All rows from the overlay are written into the output: if a matching
(benchmark, treatment) row exists in the base it is replaced, and if it does
not exist it is appended. The combined result is written to the requested
output path.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Tuple


@dataclass(frozen=True)
class ProfileKey:
  benchmark: str
  treatment: str

  @classmethod
  def from_entry(cls, entry: dict) -> "ProfileKey":
    benchmark = entry.get("benchmark")
    if benchmark is None:
      raise ValueError("Entry missing required field 'benchmark'")

    treatment = entry.get("runMethod")
    if treatment is None:
      raise ValueError("Entry missing required field 'treatment'")

    return cls(str(benchmark), str(treatment))

  def format(self) -> str:
    return f"(benchmark={self.benchmark}, treatment={self.treatment})"


def load_entries(path: Path) -> List[dict]:
  try:
    with path.open() as f:
      data = json.load(f)
  except OSError as exc:
    raise RuntimeError(f"Failed to read '{path}': {exc}") from exc
  except json.JSONDecodeError as exc:
    raise RuntimeError(f"File '{path}' is not valid JSON: {exc}") from exc

  if not isinstance(data, list):
    raise ValueError(f"Expected top-level list in '{path}', got {type(data).__name__}")

  return data


def ensure_unique_keys(entries: Iterable[dict], *, source: str) -> Dict[ProfileKey, dict]:
  seen: Dict[ProfileKey, dict] = {}
  for entry in entries:
    key = ProfileKey.from_entry(entry)
    if key in seen:
      raise ValueError(
        f"Duplicate entry for {key.format()} found in {source}."
      )
    seen[key] = entry
  return seen


def merge_profiles(base_entries: List[dict], overlay_entries: List[dict]) -> Tuple[List[dict], List[ProfileKey]]:
  base_keyed = ensure_unique_keys(base_entries, source="base profile")
  overlay_keyed = ensure_unique_keys(overlay_entries, source="overlay profile")

  merged: List[dict] = []
  # Track which overlay keys we have applied so that we can append leftovers later.
  applied_overlay_keys: Dict[ProfileKey, dict] = {}

  for entry in base_entries:
    key = ProfileKey.from_entry(entry)
    if key in overlay_keyed:
      replacement = overlay_keyed[key]
      merged.append(replacement)
      applied_overlay_keys[key] = replacement
    else:
      merged.append(entry)

  # append any overlay entries that were missing from the base
  missing_from_base: List[ProfileKey] = []
  for key, entry in overlay_keyed.items():
    if key not in applied_overlay_keys:
      merged.append(entry)
      missing_from_base.append(key)

  return merged, missing_from_base


def write_output(entries: List[dict], path: Path, indent: int) -> None:
  try:
    with path.open("w") as f:
      json.dump(entries, f, indent=indent)
      f.write("\n")
  except OSError as exc:
    raise RuntimeError(f"Failed to write '{path}': {exc}") from exc


def parse_args(argv: List[str]) -> argparse.Namespace:
  parser = argparse.ArgumentParser(
    description=(
      "Overwrite rows in a profile.json file using entries from another "
      "profile.json. Rows are matched by (benchmark, treatment)."
    )
  )
  parser.add_argument("base", type=Path, help="Path to the base profile JSON file")
  parser.add_argument("overlay", type=Path, help="Path to the overlay profile JSON file")
  parser.add_argument("output", type=Path, help="Path to write the merged profile JSON file")
  parser.add_argument(
    "--indent",
    type=int,
    default=2,
    help="Number of spaces to use for JSON indentation (default: 2)",
  )
  return parser.parse_args(argv)


def main(argv: List[str] | None = None) -> int:
  args = parse_args(argv or sys.argv[1:])

  base_entries = load_entries(args.base)
  overlay_entries = load_entries(args.overlay)

  merged_entries, missing_from_base = merge_profiles(base_entries, overlay_entries)

  if missing_from_base:
    missing_summary = ", ".join(key.format() for key in missing_from_base)
    print(
      "Added entries that were missing from the base profile: "
      f"{missing_summary}",
      file=sys.stderr,
    )

  write_output(merged_entries, args.output, args.indent)

  return 0


if __name__ == "__main__":
  raise SystemExit(main())
