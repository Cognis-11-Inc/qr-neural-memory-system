"""Independent full-signed-state verification of frozen-medium Experiment 0004."""

import argparse
import array
import csv
import hashlib
import json
import math
import struct
import sys
from pathlib import Path

from analysis.experiment_0003 import fnv

FIELDS = 4096 * 64
MODEL = b"qr4-fixed-0002-v1;registration=4a2283a"
TIMES = (0, 8, 16, 32, 64, 128, 160, 256, 384, 512, 768, 1024)
ARMS = ("A", "B", "AB+", "AB-")


def rows(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def close(a, b, tolerance=1e-10):
    if not math.isfinite(a) or not math.isfinite(b) or abs(a - b) > tolerance * max(1, abs(b)):
        raise ValueError(f"measurement mismatch: {a} != {b}")


def read_frame(path, checksum=True):
    """Decode exact bounded full fields; no simulation code is imported."""
    data = path.read_bytes()
    start = 52 + len(MODEL)
    if len(data) != start + 17 * FIELDS + 8 or data[:8] != b"QR4FLD01":
        raise ValueError("invalid full-field length/magic")
    version = struct.unpack_from("<I", data, 8)[0]
    topology, config, arm, reserved = struct.unpack_from("<4B", data, 12)
    seed, step, dt, source_a, source_b, length = struct.unpack_from("<QQdIII", data, 16)
    expected_sources = (2184, 2184) if arm == 5 else (2182, 2186)
    if (
        version != 1
        or topology != 0
        or config not in (0, 1)
        or arm not in range(6)
        or reserved
        or dt not in (0.1, 0.05)
        or step > (1024 if dt == 0.1 else 2048)
        or length != len(MODEL)
        or data[52:start] != MODEL
        or (source_a, source_b) != expected_sources
    ):
        raise ValueError("invalid full-field provenance")
    expected = struct.unpack_from("<Q", data, len(data) - 8)[0]
    if checksum and fnv(data[:-8]) != expected:
        raise ValueError("full-field checksum mismatch")
    palette = data[start : start + FIELDS]
    if max(palette) > 4:
        raise ValueError("invalid palette")
    u, v = array.array("d"), array.array("d")
    u.frombytes(data[start + FIELDS : start + 9 * FIELDS])
    v.frombytes(data[start + 9 * FIELDS : -8])
    if sys.byteorder != "little":
        u.byteswap()
        v.byteswap()
    if any(not math.isfinite(x) or abs(x) > 100 for a in (u, v) for x in a):
        raise ValueError("nonfinite/unbounded recorded field")
    return {
        "seed": seed,
        "step": step,
        "dt": dt,
        "config": config,
        "arm": arm,
        "palette": palette,
        "u": u,
        "v": v,
        "fnv": f"{expected:016x}",
        "sha256": hashlib.sha256(data).hexdigest(),
    }


def norms(a, b, plus, minus):
    """Independent signed-field residual/cross-term measurements across all modules."""
    maximum = [0.0, 0.0]
    residual_squared = [0.0, 0.0]
    expected_squared = [0.0, 0.0]
    for name in ("u", "v"):
        fields = [f[name] for f in (a, b, plus, minus)]
        dot = sum(x * y for x, y in zip(fields[0], fields[1], strict=True))
        squares = [sum(x * x for x in values) for values in fields]
        scale = max(1, max(abs(x) + abs(y) for x, y in zip(*fields[:2], strict=True)))
        for j, sign in enumerate((1, -1)):
            maximum[j] = 0.0
            residual = expected_norm = 0.0
            for x, y, actual in zip(fields[0], fields[1], fields[j + 2], strict=True):
                expected = x + sign * y
                difference = actual - expected
                maximum[j] = max(maximum[j], abs(difference))
                residual += difference * difference
                expected_norm += expected * expected
            if maximum[j] > 1e-10 * scale:
                raise ValueError("signed superposition residual exceeds registration")
            close(squares[j + 2] - squares[0] - squares[1], sign * 2 * dot)
            residual_squared[j] += residual
            expected_squared[j] += expected_norm
        yield name, maximum.copy(), squares, dot
    yield (
        "relative",
        [
            math.sqrt(r / e) if e else math.sqrt(r)
            for r, e in zip(residual_squared, expected_squared, strict=True)
        ],
        [],
        0,
    )


def expected_cases():
    cases = []
    groups = [(seed, config, 0.1) for seed in (7, 19, 41) for config in (0, 1)]
    groups += [(7, config, 0.05) for config in (0, 1)]
    for g, (seed, config, dt) in enumerate(groups):
        for j, arm in enumerate(ARMS):
            cases.append((str(g), arm, seed, config, dt, f"group-{g:02}/{j}"))
    cases += [(f"zero-{j}", "zero", 7, 0, dt, f"zero-{j}") for j, dt in enumerate((0.1, 0.05))]
    cases.append(("regression", "regression", 7, 0, 0.1, "regression.csv"))
    return cases


def verify_metrics(path, dt):
    data = rows(path / "metrics.csv")
    steps = 1024 if dt == 0.1 else 2048
    if [(int(r["step"]), r["arm"]) for r in data] != [
        (step, arm) for step in range(steps + 1) for arm in ARMS
    ]:
        raise ValueError("all-tick condition/order coverage failed")
    for step in range(steps + 1):
        chunk = data[4 * step : 4 * step + 4]
        expected_overlap = all(float(r["mid_peak"]) >= 1e-6 for r in chunk[:2])
        if any((r["overlap"] == "true") != expected_overlap for r in chunk):
            raise ValueError("midpoint overlap flag mismatch")
        for r in chunk:
            if any(
                not math.isfinite(float(v)) for k, v in r.items() if k not in ("arm", "overlap")
            ):
                raise ValueError("nonfinite metric")
    maximum = 0.0
    overlap = None
    for r in data:
        step = int(r["step"])
        close(float(r["time"]), step * dt)
        if int(r["nonfinite"]) or float(r["peak"]) > 100 or float(r["velocity_peak"]) > 100:
            raise ValueError("invalid all-tick numerical screen")
        maximum = max(maximum, abs(float(r["residual_u"])), abs(float(r["residual_v"])))
        if maximum > 1e-10:
            raise ValueError("all-tick residual beyond tolerance")
        if r["arm"] in ARMS[2:]:
            sign = 1 if r["arm"] == "AB+" else -1
            for suffix in ("u", "v"):
                close(float(r[f"cross_{suffix}"]), sign * 2 * float(r[f"dot_{suffix}"]))
        if r["overlap"] == "true" and overlap is None:
            overlap = step
    exported = rows(path / "overlap.csv")[0]
    if exported["step"] != (str(overlap) if overlap is not None else ""):
        raise ValueError("overlap export mismatch")
    return {(int(r["step"]), r["arm"]): r for r in data}, maximum, overlap


def verify_group(folder, g, identities, hashes, sha_rows):
    dt = identities[0][4]
    factor = 1 if dt == 0.1 else 2
    path = folder / f"group-{g:02}"
    if not (path / "COMPLETE").exists() or (path / "SUPERPOSITION_FAILURE").exists():
        raise ValueError("incomplete/failed group")
    metrics, maximum, overlap = verify_metrics(path, dt)
    palette = None
    sampled_peak = [[0.0] * 4096 for _ in ARMS]
    sampled_peak16 = [[0.0] * 4096 for _ in ARMS]
    sampled_arrival = [[None] * 4096 for _ in ARMS]
    for tick in TIMES:
        frames = []
        for j, identity in enumerate(identities):
            relative = f"group-{g:02}/{j}/frame-{tick * factor:06}.q4f"
            f = read_frame(folder / relative)
            if (f["seed"], f["config"], f["dt"], f["arm"], f["step"]) != (
                identity[2],
                identity[3],
                dt,
                j,
                tick * factor,
            ) or f["fnv"] != hashes[relative]:
                raise ValueError("frame/ledger identity mismatch")
            if palette is None:
                palette = f["palette"]
            if f["palette"] != palette:
                raise ValueError("persistent state changed across arms/time")
            if f["config"] == 1 and set(f["palette"]) != {1}:
                raise ValueError("homogeneous configuration mismatch")
            sha_rows.append({"file": relative, "sha256": f["sha256"]})
            row = metrics[tick * factor, ARMS[j]]
            for field, name in (("u", "sum_u2"), ("v", "sum_v2")):
                close(sum(x * x for x in f[field]), float(row[name]))
            close(max(map(abs, f["u"])), float(row["peak"]))
            close(max(map(abs, f["v"])), float(row["velocity_peak"]))
            close(f["u"][2184 * 64 + 27], float(row["mid_u"]))
            close(f["v"][2184 * 64 + 27], float(row["mid_v"]))
            for n in range(4096):
                p = max(map(abs, f["u"][n * 64 : (n + 1) * 64]))
                sampled_peak[j][n] = max(sampled_peak[j][n], p)
                if tick <= 160:
                    sampled_peak16[j][n] = max(sampled_peak16[j][n], p)
                if p >= 1e-6 and sampled_arrival[j][n] is None:
                    sampled_arrival[j][n] = tick * factor
            if tick == 0:
                close(sum(x * x for x in f["u"]), 1 if j < 2 else 2)
                if any(f["v"]):
                    raise ValueError("nonzero initial velocity")
            frames.append(f)
        for field, residuals, _, _ in norms(*frames):
            for j in range(2):
                close(
                    residuals[j],
                    float(
                        metrics[tick * factor, ARMS[j + 2]][
                            "relative_l2" if field == "relative" else f"residual_{field}"
                        ]
                    ),
                )
        print(f"verified group {g} t={tick * 0.1:g}", flush=True)
    reach = []
    for j in range(4):
        spatial = rows(path / str(j) / "spatial.csv")
        if len(spatial) != 4096:
            raise ValueError("spatial coverage incomplete")
        for n, r in enumerate(spatial):
            if int(r["id"]) != n or float(r["peak"]) + 1e-30 < sampled_peak[j][n]:
                raise ValueError("spatial peak/ID mismatch")
            if not sampled_peak16[j][n] - 1e-30 <= float(r["peak_at_time16"]) <= float(r["peak"]):
                raise ValueError("time16 cumulative peak inconsistent with observed fields")
            arrival = int(r["arrival_step"]) if r["arrival_step"] else None
            if arrival is not None:
                close(float(r["arrival_time"]), arrival * dt)
                x, y, z = n % 16, n // 16 % 16, n // 256
                sources = (6,) if j == 0 else (10,) if j == 1 else (6, 10)
                hops = min(abs(x - s) + abs(y - 8) + abs(z - 8) for s in sources)
                if not hops <= arrival <= 1024 * factor:
                    raise ValueError("noncausal arrival")
            sampled = sampled_arrival[j][n]
            if sampled is not None and (arrival is None or arrival > sampled):
                raise ValueError("arrival inconsistent with observed state")
        counts = (
            sum(
                bool(r["arrival_step"]) and int(r["arrival_step"]) <= 160 * factor for r in spatial
            ),
            sum(bool(r["arrival_step"]) for r in spatial),
        )
        exported = rows(path / str(j) / "reach.csv")[0]
        if counts != (int(exported["at_time16"]), int(exported["at_time102_4"])):
            raise ValueError("reach counts mismatch")
        reach.append(counts)
    return {
        "group": g,
        "seed": identities[0][2],
        "config": identities[0][3],
        "dt": dt,
        "max_residual": maximum,
        "first_overlap_step": overlap,
        "reach": reach,
    }


def convergence(folder):
    results = []
    for config in (0, 1):
        for j, arm in enumerate(ARMS):
            maximum, worst = 0.0, 0
            for tick in TIMES[1:]:
                a = read_frame(folder / f"group-{config:02}/{j}/frame-{tick:06}.q4f", False)
                b = read_frame(folder / f"group-{6 + config:02}/{j}/frame-{tick * 2:06}.q4f", False)
                delta = reference = 0.0
                for name in ("u", "v"):
                    delta += sum((x - y) ** 2 for x, y in zip(a[name], b[name], strict=True))
                    reference += sum(y * y for y in b[name])
                error = math.sqrt(delta / reference) if reference else math.sqrt(delta)
                if error > maximum:
                    maximum, worst = error, tick
            results.append(
                {
                    "config": config,
                    "arm": arm,
                    "max_relative_l2": maximum,
                    "worst_base_tick": worst,
                    "passes_10_percent": maximum <= 0.1,
                }
            )
    return results


def verify(folder):
    if not (folder / "COMPLETE").exists():
        raise ValueError("suite incomplete")
    ledger = rows(folder / "cases.csv")
    identities = expected_cases()
    actual = [
        (
            r["group"],
            r["arm"],
            int(r["seed"]),
            {"heterogeneous": 0, "homogeneous": 1}[r["configuration"]],
            float(r["dt"]),
            r["folder"],
        )
        for r in ledger
    ]
    if any(
        int(r["steps"])
        != (160 if r["arm"] == "regression" else 1024 if float(r["dt"]) == 0.1 else 2048)
        for r in ledger
    ):
        raise ValueError("registered duration mismatch")
    if actual != identities:
        raise ValueError("registered 35-case coverage mismatch")
    hashes = {r["file"]: r["fnv1a"] for r in rows(folder / "hashes.csv")}
    if len(hashes) != 408 or set(hashes) != {
        str(p.relative_to(folder)) for p in folder.glob("**/*.q4f")
    }:
        raise ValueError("frame ledger coverage mismatch")
    sha_rows = []
    groups = [
        verify_group(folder, g, identities[g * 4 : g * 4 + 4], hashes, sha_rows) for g in range(8)
    ]
    for j in range(2):
        frozen_palette = None
        for tick in TIMES:
            step = tick * (j + 1)
            relative = f"zero-{j}/frame-{step:06}.q4f"
            f = read_frame(folder / relative)
            if frozen_palette is None:
                frozen_palette = f["palette"]
            if (
                f["palette"] != frozen_palette
                or f["seed"] != 7
                or f["config"] != 0
                or f["dt"] != (0.1 if j == 0 else 0.05)
                or f["step"] != step
            ):
                raise ValueError("zero provenance/palette mismatch")
            if f["arm"] != 4 or any(f["u"]) or any(f["v"]) or f["fnv"] != hashes[relative]:
                raise ValueError("zero control failed")
            sha_rows.append({"file": relative, "sha256": f["sha256"]})
    regression = rows(folder / "regression.csv")
    if len(regression) != 161:
        raise ValueError("legacy regression incomplete")
    for step, r in enumerate(regression):
        if int(r["step"]) != step or r["whole_observation_equal"] != "true":
            raise ValueError("legacy whole-observation regression failed")
        for name in ("u", "v"):
            if float(r[f"new_source_{name}"]) != float(r[f"old_source_{name}"]):
                raise ValueError("legacy signed probe mismatch")
    with (folder / "sha256.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=("file", "sha256"))
        writer.writeheader()
        writer.writerows(sha_rows)
    result = {
        "executions": 35,
        "full_frames_verified": len(sha_rows),
        "superposition_within_tolerance": True,
        "persistent_changes": 0,
        "groups": groups,
        "convergence": convergence(folder),
        "sha256_ledger": hashlib.sha256((folder / "sha256.csv").read_bytes()).hexdigest(),
    }
    (folder / "analysis-summary.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n"
    )
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.folder), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
