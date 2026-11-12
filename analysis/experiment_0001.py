"""Independently score exported colored outputs and summarize registered trials."""

import argparse
import csv
import hashlib
import itertools
import json
from pathlib import Path


def colored_errors(actual: bytes, target: bytes) -> tuple[float, float, float]:
    """Score the full original target, counting missing occupied colors as errors."""
    if len(actual) != 1024 or len(target) != 1024:
        raise ValueError("expected 1,024 modules per pattern")
    if any(value > 4 for pattern in (actual, target) for value in pattern):
        raise ValueError("invalid palette value")
    pairs = list(zip(actual, target, strict=True))
    occupied = sum(value != 0 for value in target)
    return (
        sum(value != expected for value, expected in pairs) / 1024,
        sum((value != 0) != (expected != 0) for value, expected in pairs) / 1024,
        sum(value != expected for value, expected in pairs if expected != 0) / occupied
        if occupied
        else 0.0,
    )


def read_csv(path: Path) -> list[dict[str, str]]:
    """Read exported table rows."""
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def summarize(folder: Path) -> dict:
    """Validate raw-output scoring, masks of changed neurons, and trial decisions."""
    if not (folder / "COMPLETE").is_file():
        raise ValueError("incomplete run: COMPLETE marker missing")
    parameters = json.loads((folder / "parameters.json").read_text())
    metrics = read_csv(folder / "metrics.csv")
    verdicts = read_csv(folder / "verdicts.csv")
    configurations = read_csv(folder / "changes.csv")
    outputs = (folder / "outputs.bin").read_bytes()
    if len(metrics) != 168 or len(verdicts) != 24 or len(outputs) != 1024 * len(metrics):
        raise ValueError("registered row/output count mismatch")
    expected_trials = set(
        itertools.product(
            map(str, parameters["seeds"]),
            ("local", "ripple"),
            ("normal", "frozen", "color", "shape"),
        )
    )
    actual_trials = {(row["seed"], row["engine"], row["variant"]) for row in verdicts}
    if actual_trials != expected_trials:
        raise ValueError("missing or unexpected trial identity")
    by_trial: dict[tuple, dict] = {}
    for index, row in enumerate(metrics):
        key = row["seed"], row["engine"], row["variant"]
        target_name = "d" if row["condition"].endswith("cd") else "b"
        target = (folder / f"seed-{row['seed']}-stimulus-{target_name}.bin").read_bytes()
        errors = colored_errors(outputs[index * 1024 : (index + 1) * 1024], target)
        for name, value in zip(
            ("palette_error", "shape_error", "occupied_color_error"), errors, strict=True
        ):
            if abs(float(row[name]) - value) > 1e-9:
                raise ValueError(f"raw output/metric mismatch: {key} {row['condition']} {name}")
        conditions = by_trial.setdefault(key, {})
        if row["condition"] in conditions:
            raise ValueError("duplicate condition")
        conditions[row["condition"]] = errors[0]
    for row in verdicts:
        key = row["seed"], row["engine"], row["variant"]
        conditions = by_trial[key]
        if set(conditions) != {
            "untrained_ab",
            "untrained_cd",
            "trained_ab",
            "unrelated_ab",
            "retained_ab",
            "trained_cd",
            "unrelated_cd",
        }:
            raise ValueError("missing or unexpected condition")
        for pair in ("ab", "cd"):
            trained = conditions[f"trained_{pair}"]
            passes = (
                trained <= 0.10
                and conditions[f"untrained_{pair}"] - trained >= 0.15
                and conditions[f"unrelated_{pair}"] - trained >= 0.15
            )
            if (row[f"{pair}_pass"] == "true") != passes:
                raise ValueError("incorrect scientific verdict")
        retention = (
            row["ab_pass"] == "true"
            and conditions["retained_ab"] - conditions["trained_ab"] <= 0.05
        )
        if (row["retention_pass"] == "true") != retention:
            raise ValueError("incorrect retention verdict")
        stem = f"seed-{key[0]}-{key[1]}-{key[2]}"
        snapshots = [
            (folder / f"{stem}-{stage}.bin").read_bytes() for stage in ("initial", "ab", "cd")
        ]
        if any(len(snapshot) != 262144 or max(snapshot) > 4 for snapshot in snapshots):
            raise ValueError("invalid persistent snapshot")
        for stage, before, after in zip(("ab", "cd"), snapshots[:-1], snapshots[1:], strict=True):
            expected_changes = {}
            for neuron in range(4096):
                start = neuron * 64
                changed = sum(
                    a != b
                    for a, b in zip(
                        before[start : start + 64], after[start : start + 64], strict=True
                    )
                )
                if changed:
                    expected_changes[neuron] = changed
            reported = {
                int(change["neuron"]): int(change["changed_modules"])
                for change in configurations
                if (change["seed"], change["engine"], change["variant"], change["stage"])
                == (*key, stage)
            }
            if expected_changes != reported:
                raise ValueError("snapshot/change-location mismatch")
            if len(reported) != int(row[f"{stage}_neurons"]) or sum(reported.values()) != int(
                row[f"{stage}_modules"]
            ):
                raise ValueError("snapshot/change-count mismatch")
    normal = [row for row in metrics if row["variant"] == "normal"]
    return {
        "trials": len(verdicts),
        "inferences": len(metrics),
        "raw_metrics_verified": True,
        "persistent_changes_verified": True,
        "ab_passes": sum(row["ab_pass"] == "true" for row in verdicts),
        "cd_passes": sum(row["cd_pass"] == "true" for row in verdicts),
        "retention_passes": sum(row["retention_pass"] == "true" for row in verdicts),
        "stability": json.loads((folder / "stability.json").read_text()),
        "normal_metrics": normal,
        "raw_outputs_sha256": hashlib.sha256(outputs).hexdigest(),
    }


def main() -> None:
    """Validate a run and emit its summary as JSON."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    args = parser.parse_args()
    summary = summarize(args.folder)
    destination = args.folder / "analysis-summary.json"
    destination.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(
        json.dumps(
            {key: value for key, value in summary.items() if key != "normal_metrics"}, indent=2
        )
    )


if __name__ == "__main__":
    main()
