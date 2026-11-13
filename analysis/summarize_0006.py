"""Portable CSV summaries from measured eligibility, palettes and causal receipts."""

import argparse
import csv
import json
from pathlib import Path

import numpy as np

from analysis.experiment_0006 import audit_available, checkpoint, compare, decode, require, rows


def summarize(root):
    output = root / "agent-report"
    output.mkdir(exist_ok=True)
    receipts = audit_available(root)
    with (output / "morphology-summary.csv").open("w") as stream:
        writer = csv.writer(stream)
        writer.writerow(
            (
                "case",
                "seed",
                "module",
                "budget",
                "rule",
                "dt",
                "changed_neurons",
                "changed_positions",
                "source_positions",
                "neighbor_changed_neurons",
                "remote_changed_neurons",
                "positions_0_to_8",
                "global_color_histogram_preserved",
            )
        )
        for result in receipts:
            folder = root / result["id"]
            if not (folder / "scores.bin").exists():
                continue
            params = rows(folder / "parameters.csv")[0]
            before = np.frombuffer((folder / "before.palette").read_bytes(), dtype="u1").reshape(
                4096, 64
            )
            after = np.frombuffer((folder / "after.palette").read_bytes(), dtype="u1").reshape(
                4096, 64
            )
            counts = np.count_nonzero(before != after, axis=1)
            ids = np.arange(4096)
            hops = abs(ids % 16 - 6) + abs(ids // 16 % 16 - 8) + abs(ids // 256 - 8)
            histogram_equal = np.array_equal(
                np.bincount(before.ravel(), minlength=5), np.bincount(after.ravel(), minlength=5)
            )
            writer.writerow(
                (
                    folder.name,
                    params["seed"],
                    params["module"],
                    params["budget"],
                    params["rule"],
                    params["dt"],
                    np.count_nonzero(counts),
                    int(np.sum(counts)),
                    int(counts[2182]),
                    int(np.count_nonzero(counts[hops == 1])),
                    int(np.count_nonzero(counts[hops > 1])),
                    json.dumps(np.bincount(counts, minlength=9).tolist()),
                    bool(histogram_equal),
                )
            )
    with (output / "diagnosis.csv").open("w") as stream:
        writer = csv.writer(stream)
        writer.writerow(
            (
                "case",
                "id",
                "x",
                "y",
                "z",
                "hops_A",
                "injection_module",
                "argmax_V",
                "max_V",
                "V_absolute_count",
                "V_absolute_and_relative_count",
                "S_eligible_count",
                "mean_eligible_internal_distance",
                "max_eligible_internal_distance",
            )
        )
        for folder in sorted(root.glob("d-*")):
            if not (folder / "COMPLETE").exists():
                continue
            params = rows(folder / "parameters.csv")[0]
            module = int(params["module"])
            velocity = np.frombuffer(
                (folder / "velocity-history.bin").read_bytes(), dtype="<f8"
            ).reshape(4096, 64)
            scores = np.frombuffer((folder / "scores.bin").read_bytes(), dtype="<f8").reshape(
                4096, 64
            )
            for ident in range(4096):
                vmax = float(np.max(velocity[ident]))
                smax = float(np.max(scores[ident]))
                chosen = np.flatnonzero((scores[ident] >= 1e-8) & (scores[ident] >= 0.05 * smax))
                distance = abs(chosen // 8 - module // 8) + abs(chosen % 8 - module % 8)
                x, y, z = ident % 16, ident // 16 % 16, ident // 256
                writer.writerow(
                    (
                        folder.name,
                        ident,
                        x,
                        y,
                        z,
                        abs(x - 6) + abs(y - 8) + abs(z - 8),
                        module,
                        int(np.argmax(velocity[ident])),
                        format(vmax, ".17e"),
                        int(np.count_nonzero(velocity[ident] >= 1e-8)),
                        int(
                            np.count_nonzero(
                                (velocity[ident] >= 1e-8) & (velocity[ident] >= 0.05 * vmax)
                            )
                        ),
                        len(chosen),
                        float(np.mean(distance)) if len(distance) else "",
                        int(np.max(distance)) if len(distance) else "",
                    )
                )
    verification = json.loads((root / "verification.json").read_text())
    with (output / "causal-contrasts.csv").open("w") as stream:
        writer = csv.writer(stream)
        writer.writerow(
            (
                "case",
                "seed",
                "module",
                "rule",
                "dt",
                "variant",
                "budget",
                "initial_energy",
                "time",
                "global_relative_l2",
                "global_absolute_rms",
                "source_relative_l2",
                "offsource_relative_l2",
                "offsource_absolute_rms",
                "offsource_vs_source_only_relative_l2",
            )
        )
        for result in verification.get("causal", []):
            for effect in result["effects"]:
                writer.writerow(
                    (
                        result["id"],
                        result["seed"],
                        result["module"],
                        result["rule"],
                        result["dt"],
                        result["kind"],
                        result["budget"],
                        result["initial_energy"],
                        effect["time"],
                        effect["global_effect"]["relative_l2"],
                        effect["global_effect"]["absolute_rms"],
                        effect["source_effect"]["relative_l2"],
                        effect["offsource_effect"]["relative_l2"],
                        effect["offsource_effect"]["absolute_rms"],
                        effect["versus_source_only"]["relative_l2"],
                    )
                )
    with (output / "pair-contrasts.csv").open("w") as stream:
        writer = csv.writer(stream)
        writer.writerow(
            (
                "seed",
                "module",
                "rule",
                "dt",
                "budget",
                "time",
                "global_all_vs_source_relative_l2",
                "global_all_vs_source_absolute_rms",
                "offsource_all_vs_source_relative_l2",
                "global_off_vs_virgin_relative_l2",
                "global_source_vs_virgin_relative_l2",
            )
        )
        for result in verification.get("causal", []):
            if result["kind"] != "ALL":
                continue
            prefix = f"c-s{result['seed']}-m{result['module']}-R{result['rule']}-d{result['dt']}"
            for physical_time in (16, 32):
                all_state = checkpoint(root, result["id"], physical_time)
                source_state = checkpoint(
                    root, prefix + "-SOURCE_ONLY-" + result["budget"], physical_time
                )
                off_state = checkpoint(
                    root, prefix + "-OFF_SOURCE_ONLY-" + result["budget"], physical_time
                )
                virgin = checkpoint(root, prefix + "-VIRGIN-UNIT", physical_time)
                pair = compare(all_state, source_state)
                mask = np.arange(4096) != 2182
                writer.writerow(
                    (
                        result["seed"],
                        result["module"],
                        result["rule"],
                        result["dt"],
                        result["budget"],
                        physical_time,
                        pair["relative_l2"],
                        pair["absolute_rms"],
                        compare(all_state, source_state, mask)["relative_l2"],
                        compare(off_state, virgin)["relative_l2"],
                        compare(source_state, virgin)["relative_l2"],
                    )
                )
    # Structural immutability of all retained checkpoints, including switch states.
    checked = 0
    for result in receipts:
        folder = root / result["id"]
        before, after = (
            (folder / "before.palette").read_bytes(),
            (folder / "after.palette").read_bytes(),
        )
        params = rows(folder / "parameters.csv")[0]
        for file in sorted(folder.glob("*.q6f")):
            frame = decode(file)
            expected = before if frame["tick"] * frame["dt"] < 8 - 1e-9 else after
            require(
                frame["palette"].tobytes() == expected, "actual checkpoint palette immutability"
            )
            checked += 1
        if (folder / "switch").exists():
            require(
                decode(folder / "switch/pre.q6f")["palette"].tobytes() == before,
                "switch before palette",
            )
            require(
                decode(folder / "switch/post.q6f")["palette"].tobytes() == after,
                "switch after palette",
            )
        if params["rule"] == "4":
            reference = root / folder.name.replace("R4", "R0")
            for file in folder.glob("*.q6f"):
                a, b = decode(file), decode(reference / file.name)
                require(
                    all(a[k].tobytes() == b[k].tobytes() for k in ("palette", "u", "v")),
                    "disabled rule matches frozen arrays",
                )
    (output / "additional-verification.json").write_text(
        json.dumps(dict(checkpoint_palettes_checked=checked, disabled_equals_frozen=True), indent=2)
        + "\n"
    )
    print("summary CSVs and checkpoint palette checks", checked)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    summarize(args.root)
