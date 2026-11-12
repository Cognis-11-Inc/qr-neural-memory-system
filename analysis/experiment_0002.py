"""Independently verify Experiment 0002 traces and apply its frozen T/R criteria."""

import argparse
import cmath
import csv
import hashlib
import itertools
import json
import math
import struct
from collections import defaultdict
from pathlib import Path

BINS = (4, 8, 12, 16, 20, 24, 28)
SEEDS = (7, 19, 41)
CLASSES = ("homogeneous", "heterogeneous", "rearranged")


def rows(path: Path) -> list[dict[str, str]]:
    """Read a raw evidence table."""
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def close(actual: float, expected: float) -> None:
    """Reject nonfinite or inconsistent exported numerical measurements."""
    if not math.isfinite(actual) or not math.isclose(
        actual, expected, rel_tol=2e-10, abs_tol=1e-30
    ):
        raise ValueError(f"numeric mismatch: {actual} vs {expected}")


def rms(values: list[float]) -> float:
    """Root mean squared, including zeros in the denominator."""
    return math.sqrt(sum(value * value for value in values) / len(values))


def component(values: list[float], times: list[float], omega: float) -> complex:
    """Unnormalized sine/cosine projection; its argument is sine-relative phase."""
    return complex(
        sum(value * math.sin(omega * time) for value, time in zip(values, times, strict=True)),
        sum(value * math.cos(omega * time) for value, time in zip(values, times, strict=True)),
    )


def peak_test(gains: list[float]) -> dict:
    """Apply the registered interior-peak and off-frequency comparisons."""
    index = max(range(len(gains)), key=gains.__getitem__)
    interior = 0 < index < len(gains) - 1
    resolved = (
        interior
        and gains[index] >= 1.2 * gains[index - 1]
        and gains[index] >= 1.2 * gains[index + 1]
        and gains[index] >= 2 * min(gains)
    )
    return {
        "peak_bin": BINS[index],
        "peak_gain": gains[index],
        "resolved": resolved,
        "interior": interior,
    }


def relative_l2(actual: list, reference: list) -> float:
    """L2 difference divided by the reference norm; zero-reference handled explicitly."""
    numerator = sum(abs(a - b) ** 2 for a, b in zip(actual, reference, strict=True))
    denominator = sum(abs(value) ** 2 for value in reference)
    if denominator == 0:
        return 0.0 if numerator == 0 else math.inf
    return math.sqrt(numerator / denominator)


def expected_keys() -> set:
    """Enumerate all registered identities independently of the Rust ledger."""
    keys = set()
    for seed, channel, source, drive in itertools.product(
        SEEDS, range(1, 5), ("center", "edge", "corner"), ("impulse", "burst")
    ):
        keys.add(("T", seed, "heterogeneous", "full", channel, source, drive, 16, 0.1, 1.0))
    keys.add(("T", 7, "homogeneous", "full", 1, "center", "zero", 16, 0.1, 1.0))
    for seed, config, channel, bin_id in itertools.product(SEEDS, CLASSES, range(1, 5), BINS):
        for mode in ("scalar", "isolated", "patch"):
            keys.add(("R", seed, config, mode, channel, "center", "tone", bin_id, 0.1, 1.0))
        keys.add(("R", seed, config, "isolated", channel, "center", "tone", bin_id, 0.05, 1.0))
    for bin_id in BINS:
        keys.add(("R", 7, "heterogeneous", "full", 1, "center", "tone", bin_id, 0.1, 1.0))
        keys.add(("R", 7, "heterogeneous", "isolated", 1, "center", "tone", bin_id, 0.1, 2.0))
        if bin_id in (8, 16, 24):
            keys.add(("R", 7, "heterogeneous", "full", 1, "center", "tone", bin_id, 0.05, 1.0))
    for drive in ("multisine", "random_phase"):
        keys.add(("R", 7, "heterogeneous", "patch", 1, "center", drive, 16, 0.1, 1.0))
    keys.add(("R", 7, "homogeneous", "isolated", 1, "center", "zero", 16, 0.1, 1.0))
    return keys


def key(trial: dict) -> tuple:
    """Canonical ledger identity."""
    return (
        trial["stage"],
        int(trial["seed"]),
        trial["configuration"],
        trial["mode"],
        int(trial["channel"]),
        trial["source"],
        trial["drive"],
        int(trial["bin"]),
        float(trial["dt"]),
        float(trial["gamma_factor"]),
    )


def evaluate_probe(data: list[tuple], inputs: list[float], trial: dict, exported: dict) -> dict:
    """Recompute every Rust summary field from the exported raw probe trace."""
    dt, omega = float(trial["dt"]), float(trial["omega"])
    steps = int(trial["steps"])
    times = [index * dt for index in range(len(inputs))]
    displacement = [value[0] for value in data]
    velocity = [value[1] for value in data]
    start = 0 if trial["stage"] == "T" else steps // 2 + 1
    steady = displacement[start:]
    force = inputs[start:]
    drive_rms = rms(force)
    projection = component(force, times[start:], omega)
    transfer = component(steady, times[start:], omega) / projection if abs(projection) else 0j
    result = {
        "peak": max(map(abs, displacement)),
        "rms": rms(steady),
        "velocity_rms": rms(velocity[start:]),
        "drive_rms": drive_rms,
        "gain": rms(steady) / drive_rms if drive_rms else 0.0,
        "transfer_re": transfer.real,
        "transfer_im": transfer.imag,
        "phase": cmath.phase(transfer),
    }
    for name, value in result.items():
        if name == "phase":
            if abs(math.remainder(float(exported[name]) - value, math.tau)) > 1e-10:
                raise ValueError("phase mismatch")
        else:
            close(float(exported[name]), value)
    for name, threshold in (("arrival", 1e-6), ("detect_arrival", 1e-14)):
        found = next(
            (
                time
                for time, value in zip(times, displacement, strict=True)
                if abs(value) >= threshold
            ),
            None,
        )
        result[name] = found
        if found is None:
            if exported[name]:
                raise ValueError("incorrect threshold arrival")
        else:
            close(float(exported[name]), found)
    return result


def write_table(path: Path, data: list[dict]) -> None:
    """Export derived observables with explicit headers."""
    with path.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(data[0]))
        writer.writeheader()
        writer.writerows(data)


def verify(folder: Path) -> dict:
    """Verify a complete scheduled suite and derive scientific T/R decisions."""
    if not (folder / "COMPLETE").is_file():
        raise ValueError("suite not complete")
    ledger = rows(folder / "trials.csv")
    if len(ledger) != 1101 or {key(t) for t in ledger} != expected_keys():
        raise ValueError("registered trial coverage mismatch")
    if [int(t["id"]) for t in ledger] != list(range(1101)):
        raise ValueError("trial ID mismatch")
    probes = defaultdict(dict)
    for probe in rows(folder / "probes.csv"):
        probes[probe["id"]][int(probe["probe"])] = probe
    summary = {(r["id"], int(r["probe"])): r for r in rows(folder / "summary.csv")}
    observations, spectra, responses = [], [], []
    energies, numerical_failures = [], []
    zero_ok = True
    phases = {}
    for trial in ledger:
        identity = trial["id"]
        dt, steps = float(trial["dt"]), int(trial["steps"])
        source = int(trial["source_point"])
        raw_inputs = rows(folder / f"inputs/{identity}.csv")
        inputs = [float(r["drive"]) for r in raw_inputs]
        if len(inputs) != steps + 1:
            raise ValueError("truncated input schedule")
        for index, row in enumerate(raw_inputs):
            if int(row["step"]) != index:
                raise ValueError("input order mismatch")
            close(float(row["time"]), index * dt)
        energy = sum(value * value * dt for value in inputs)
        expected = 0.0 if trial["drive"] in ("zero", "impulse") else 1.0
        if abs(energy - expected) > 1e-10:
            raise ValueError("input energy mismatch")
        energies.append(abs(energy - expected))
        diagnostic = rows(folder / f"diagnostics/{identity}.csv")
        valid = all(
            int(r["nonfinite"]) == 0
            and float(r["peak"]) <= 100
            and float(r["velocity_peak"]) <= 100
            for r in diagnostic
        )
        if not valid:
            numerical_failures.append(identity)
        before = (folder / f"state/{identity}-p-before.bin").read_bytes()
        after = (folder / f"state/{identity}-p-after.bin").read_bytes()
        points = int(trial["side"]) ** 2 * 64
        if before != after or len(before) != points or max(before) > 4:
            raise ValueError("persistent structure mismatch")
        if trial["configuration"] == "homogeneous" and any(value != 1 for value in before):
            raise ValueError("homogeneous control is not homogeneous")
        final = list(
            struct.iter_unpack("<dd", (folder / f"state/{identity}-uv-final.bin").read_bytes())
        )
        if len(final) != points:
            raise ValueError("state snapshot length mismatch")
        close(sum(u * u for u, _ in final), float(diagnostic[-1]["sum_u2"]))
        close(sum(v * v for _, v in final), float(diagnostic[-1]["sum_v2"]))
        close(max(abs(u) for u, _ in final), float(diagnostic[-1]["peak"]))
        close(max(abs(v) for _, v in final), float(diagnostic[-1]["velocity_peak"]))
        if trial["drive"] == "zero":
            zero_ok &= all(float(r["sum_u2"]) == 0 and float(r["sum_v2"]) == 0 for r in diagnostic)
        data = defaultdict(list)
        raw_trace = rows(folder / f"traces/{identity}.csv")
        for row in raw_trace:
            probe = int(row["probe"])
            if int(row["step"]) != len(data[probe]):
                raise ValueError("probe trace order mismatch")
            data[probe].append((float(row["u"]), float(row["v"])))
        if set(data) != set(probes[identity]):
            raise ValueError("observation coverage mismatch")
        measured = {}
        for probe, values in data.items():
            exported = summary[identity, probe]
            if (exported["valid"] == "true") != valid:
                raise ValueError("numerical validity mismatch")
            if (
                len(values) != len(diagnostic)
                or int(exported["completed_steps"]) != len(values) - 1
            ):
                raise ValueError("partial trial accounting mismatch")
            if valid and len(values) != steps + 1:
                raise ValueError("valid trial is truncated")
            close(values[-1][0], final[int(probes[identity][probe]["point"])][0])
            close(values[-1][1], final[int(probes[identity][probe]["point"])][1])
            measured[probe] = evaluate_probe(values, inputs[: len(values)], trial, exported)
        for probe, values in data.items():
            result = measured[probe]
            transfer = result["rms"] / measured[0]["rms"] if measured[0]["rms"] else 0.0
            close(float(summary[identity, probe]["source_transfer"]), transfer)
            if trial["stage"] == "T":
                meta = probes[identity][probe]
                if meta["scope"] == "internal":
                    dr, dc = divmod(int(meta["module"]), 8)
                    distance = math.hypot(dr - 3, dc - 3)
                else:
                    side = int(trial["side"])
                    source_row, source_col = divmod(source // 64, side)
                    distance = math.hypot(
                        int(meta["row"]) - source_row, int(meta["col"]) - source_col
                    )
                    close(distance, float(meta["distance"]))
                observations.append(
                    {
                        "id": identity,
                        "seed": trial["seed"],
                        "channel": trial["channel"],
                        "source": trial["source"],
                        "drive": trial["drive"],
                        "label": meta["label"],
                        "scope": meta["scope"],
                        "hops": int(meta["hops"]),
                        "euclidean_distance": distance,
                        "source_transfer": transfer,
                        **result,
                    }
                )
                times = [index * dt for index in range(1, len(values))]
                signal = [value[0] for value in values[1:]]
                for bin_id in range(17):
                    omega = math.tau * bin_id / 16
                    value = component(signal, times, omega)
                    spectra.append(
                        {
                            "id": identity,
                            "probe": probe,
                            "bin": bin_id,
                            "omega": omega,
                            "amplitude": 2 * abs(value) / len(signal),
                            "phase": cmath.phase(value),
                        }
                    )
            else:
                responses.append({**trial, **result})
        if trial["drive"] in ("multisine", "random_phase"):
            times = [index * dt for index in range(1, len(inputs))]
            phases[trial["drive"]] = [
                abs(component(inputs[1:], times, math.tau * bin_id / 64)) for bin_id in (8, 16, 24)
            ]
    for a, b in zip(phases["multisine"], phases["random_phase"], strict=True):
        close(a, b)
    return decisions(
        folder, ledger, observations, spectra, responses, energies, numerical_failures, zero_ok
    )


def decisions(
    folder, ledger, observations, spectra, responses, energies, numerical_failures, zero_ok
):
    """Apply all fixed scientific comparisons without discarding negative conditions."""
    curves = defaultdict(list)
    for response in responses:
        if response["drive"] == "tone":
            identity = (
                response["seed"],
                response["configuration"],
                response["mode"],
                response["channel"],
                float(response["dt"]),
                float(response["gamma_factor"]),
            )
            curves[identity].append(response)
    for curve in curves.values():
        curve.sort(key=lambda row: int(row["bin"]))

    def get(seed, config, mode, channel, dt=0.1, gamma=1.0):
        curve = curves[str(seed), config, mode, str(channel), dt, gamma]
        if not curve:
            raise ValueError("missing registered frequency-response curve")
        return curve

    selectivity, sensitivity, convergence = [], [], []
    for seed, config, channel in itertools.product(SEEDS, CLASSES, range(1, 5)):
        coarse = get(seed, config, "isolated", channel)
        fine = get(seed, config, "isolated", channel, dt=0.05)
        peak = peak_test([row["gain"] for row in coarse])
        fine_peak = peak_test([row["gain"] for row in fine])
        selectivity.append({"seed": seed, "configuration": config, "channel": channel, **peak})
        error = relative_l2(
            [complex(r["transfer_re"], r["transfer_im"]) for r in coarse],
            [complex(r["transfer_re"], r["transfer_im"]) for r in fine],
        )
        convergence.append(
            {
                "seed": seed,
                "configuration": config,
                "channel": channel,
                "relative_complex_error": error,
                "peak_agrees": peak["peak_bin"] == fine_peak["peak_bin"],
                "pass": error <= 0.10 and peak["peak_bin"] == fine_peak["peak_bin"],
            }
        )
    for seed, channel in itertools.product(SEEDS, range(1, 5)):
        normal = [r["gain"] for r in get(seed, "heterogeneous", "isolated", channel)]
        rearranged = [r["gain"] for r in get(seed, "rearranged", "isolated", channel)]
        difference = relative_l2(rearranged, normal)
        sensitivity.append(
            {"seed": seed, "channel": channel, "relative_l2": difference, "pass": difference > 0.05}
        )
    full_coarse = {int(r["bin"]): r for r in get(7, "heterogeneous", "full", 1)}
    full_fine = get(7, "heterogeneous", "full", 1, dt=0.05)
    full_convergence = []
    for fine in full_fine:
        coarse = full_coarse[int(fine["bin"])]
        ref = complex(fine["transfer_re"], fine["transfer_im"])
        error = abs(complex(coarse["transfer_re"], coarse["transfer_im"]) - ref) / abs(ref)
        full_convergence.append({"bin": fine["bin"], "error": error, "pass": error <= 0.15})
    required = [
        row
        for row in observations
        if row["source"] == "center"
        and row["hops"] == 4
        and row["scope"] == "external"
        and len(row["label"]) == 2
        and row["drive"] != "zero"
    ]
    if len(required) != 96:
        raise ValueError("Stage T criterion coverage mismatch")
    transport_passes = sum(
        row["arrival"] is not None and row["source_transfer"] >= 1e-3 for row in required
    )
    common = zero_ok and not numerical_failures and max(energies) <= 1e-10
    numerical_r = all(row["pass"] for row in convergence + full_convergence)
    stage_t = (
        "indeterminate" if not common else "positive" if transport_passes == 96 else "negative"
    )
    stage_r = (
        "indeterminate"
        if not common or not numerical_r or any(not row["interior"] for row in selectivity)
        else "positive"
        if all(row["resolved"] for row in selectivity) and all(row["pass"] for row in sensitivity)
        else "negative"
    )
    damping_normal = max(row["gain"] for row in get(7, "heterogeneous", "isolated", 1))
    damping_double = max(row["gain"] for row in get(7, "heterogeneous", "isolated", 1, gamma=2.0))
    write_table(folder / "transport-verified.csv", observations)
    write_table(folder / "spectra-verified.csv", spectra)
    write_table(folder / "responses-verified.csv", responses)
    detail = {
        "selectivity": selectivity,
        "shape_sensitivity": sensitivity,
        "convergence": convergence,
        "full_convergence": full_convergence,
    }
    (folder / "spectral-decisions.json").write_text(
        json.dumps(detail, indent=2, sort_keys=True) + "\n"
    )
    result = {
        "trials": len(ledger),
        "T_trials": 73,
        "R_trials": 1028,
        "stage_T": stage_t,
        "stage_R": stage_r,
        "stage_P": "not_run",
        "transport_probes_passing": transport_passes,
        "transport_probes_required": 96,
        "resolved_sweeps": sum(row["resolved"] for row in selectivity),
        "isolated_sweeps": 36,
        "shape_sensitive_pairs": sum(row["pass"] for row in sensitivity),
        "shape_pairs": 12,
        "converged_sweeps": sum(row["pass"] for row in convergence),
        "full_convergence_passes": sum(row["pass"] for row in full_convergence),
        "max_input_energy_error": max(energies),
        "zero_drive_exact": zero_ok,
        "numerical_failures": numerical_failures,
        "all_structures_preserved": True,
        "damping_normal_peak_gain": damping_normal,
        "damping_double_peak_gain": damping_double,
        "raw_summary_sha256": hashlib.sha256((folder / "summary.csv").read_bytes()).hexdigest(),
    }
    (folder / "analysis-summary.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n"
    )
    return result


def main() -> None:
    """Verify all raw evidence and write analysis tables and decisions."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.folder), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
