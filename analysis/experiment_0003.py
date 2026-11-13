"""Independent QR3OBS01 reader, scientific summaries, and real-viewer receipt checks."""

import argparse
import csv
import hashlib
import itertools
import json
import math
import struct
from pathlib import Path

NEURON = struct.Struct("<4I64s18dI")
MODEL = b"qr3-oscillator-v1;physics=0002;registration=5a112c1"


def fnv(data: bytes) -> int:
    """Independent FNV-1a; corruption checksum, not authentication."""
    value = 0xCBF29CE484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def read_snapshot(path: Path, verify_checksum=True) -> dict:
    """Decode the bounded binary schema without importing/executing the Rust engine."""
    data = path.read_bytes()
    if len(data) < 64 or len(data) > 2_000_000 or data[:8] != b"QR3OBS01":
        raise ValueError("invalid observation length/magic")
    expected = struct.unpack_from("<Q", data, len(data) - 8)[0]
    if verify_checksum and fnv(data[:-8]) != expected:
        raise ValueError("observation checksum mismatch")
    offset = 8

    def take(fmt):
        nonlocal offset
        result = struct.unpack_from(fmt, data, offset)
        offset += struct.calcsize(fmt)
        return result

    (version,) = take("<I")
    topology, config, drive, channel = take("<4B")
    seed, source, tick, dt, synthetic, length = take("<QIQdBI")
    model = data[offset : offset + length]
    offset += length
    dims = take("<3I")
    (count,) = take("<I")
    if (
        version != 1
        or model != MODEL
        or count != 4096
        or channel >= 4
        or source >= 4096
        or tick > 160
    ):
        raise ValueError("invalid schema/provenance/population")
    if (
        dt != 0.1
        or topology not in (0, 1, 2)
        or dims != ((64, 64, 1) if topology == 2 else (16, 16, 16))
    ):
        raise ValueError("invalid topology/dt")
    if len(data) != offset + count * NEURON.size + 8:
        raise ValueError("truncated/trailing neuron observations")
    neurons = list(NEURON.iter_unpack(data[offset:-8]))
    for index, n in enumerate(neurons):
        xyz = index % dims[0], index // dims[0] % dims[1], index // (dims[0] * dims[1])
        if n[0] != index or n[1:4] != xyz or max(n[4]) > 4:
            raise ValueError("invalid neuron ID/coordinates/palette")
    return {
        "topology": topology,
        "config": config,
        "drive": drive,
        "channel": channel,
        "seed": seed,
        "source": source,
        "tick": tick,
        "dt": dt,
        "synthetic": synthetic,
        "dims": dims,
        "neurons": neurons,
        "fnv": f"{expected:016x}",
    }


def rows(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def close(actual, expected):
    if not math.isfinite(actual) or not math.isclose(
        actual, expected, rel_tol=2e-12, abs_tol=1e-30
    ):
        raise ValueError(f"numeric mismatch {actual} vs {expected}")


def coordinates(index, topology):
    side = 64 if topology == "Grid2D8" else 16
    return index % side, index // side % side, index // (side * side)


def geometry(folder):
    degrees = {
        (r["topology"], int(r["degree"])): int(r["neurons"]) for r in rows(folder / "degrees.csv")
    }
    expected = {
        ("Cubic6", 3): 8,
        ("Cubic6", 4): 168,
        ("Cubic6", 5): 1176,
        ("Cubic6", 6): 2744,
        ("Cubic26", 7): 8,
        ("Cubic26", 11): 168,
        ("Cubic26", 17): 1176,
        ("Cubic26", 26): 2744,
        ("Grid2D8", 3): 4,
        ("Grid2D8", 5): 248,
        ("Grid2D8", 8): 3844,
    }
    if degrees != expected:
        raise ValueError("degree histogram mismatch")
    lengths = {
        (r["topology"], int(r["squared_length"])): int(r["undirected_links"])
        for r in rows(folder / "link_lengths.csv")
    }
    if lengths != {
        ("Cubic6", 1): 11520,
        ("Cubic6", 2): 0,
        ("Cubic6", 3): 0,
        ("Cubic26", 1): 11520,
        ("Cubic26", 2): 21600,
        ("Cubic26", 3): 13500,
        ("Grid2D8", 1): 8064,
        ("Grid2D8", 2): 7938,
        ("Grid2D8", 3): 0,
    }:
        raise ValueError("link-length count mismatch")


def verify(folder: Path) -> dict:
    """Check all scientific observations; checksum-sample independently of core readback."""
    if not (folder / "COMPLETE").exists():
        raise ValueError("incomplete suite")
    ledger = rows(folder / "cases.csv")
    if len(ledger) != 111 or [int(r["case"]) for r in ledger] != list(range(111)):
        raise ValueError("case ledger incomplete")
    expected = set()
    for topology, seed, config in itertools.product(
        ("Cubic6", "Cubic26"), (7, 19, 41), ("heterogeneous", "homogeneous", "rearranged")
    ):
        for xyz in ((8, 8, 8), (0, 0, 0), (0, 8, 8), (0, 0, 8)):
            expected.add((topology, seed, config, xyz, "impulse"))
        for drive in ("tone_pi_4", "tone_pi_2"):
            expected.add((topology, seed, config, (8, 8, 8), drive))
    for topology in ("Cubic6", "Cubic26"):
        expected.add((topology, 7, "homogeneous", (8, 8, 8), "zero"))
    expected.add(("Grid2D8", 7, "heterogeneous", (32, 32, 0), "impulse"))
    actual = {
        (
            r["topology"],
            int(r["seed"]),
            r["configuration"],
            tuple(int(r[c]) for c in "xyz"),
            r["drive"],
        )
        for r in ledger
    }
    if actual != expected:
        raise ValueError("registered condition coverage mismatch")
    geometry(folder)
    hashes = {(int(r["case"]), int(r["tick"])): r["fnv1a"] for r in rows(folder / "hashes.csv")}
    summary, shells, checksums, frame_count = [], [], 0, 0
    for trial in ledger:
        identity = int(trial["case"])
        path = folder / f"case-{identity:04}"
        metrics = rows(path / "metrics.csv")
        spatial = rows(path / "spatial.csv")
        if len(metrics) != 161 or len(spatial) != 4096:
            raise ValueError("missing tick/spatial records")
        if any(
            int(r["nonfinite"]) or float(r["peak"]) > 100 or float(r["velocity_peak"]) > 100
            for r in metrics
        ):
            raise ValueError("numerical screen failed; retain case as invalid")
        if [int(r["tick"]) for r in metrics] != list(range(161)):
            raise ValueError("tick order mismatch")
        source = int(trial["source"])
        origin = coordinates(source, trial["topology"])
        first_palette = None
        sampled_peaks = [0.0] * 4096
        sampled_first = [None] * 4096
        files = sorted(path.glob("*.qrs"))
        if len(files) != 21:
            raise ValueError("recorded timeline coverage mismatch")
        for frame_index, file in enumerate(files):
            checked = identity == 0 or frame_index in (0, 20)
            snapshot = read_snapshot(file, checked)
            identity_fields = (
                {"Cubic6": 0, "Cubic26": 1, "Grid2D8": 2}[trial["topology"]],
                {"heterogeneous": 0, "homogeneous": 1, "rearranged": 2}[trial["configuration"]],
                {"impulse": 0, "tone_pi_4": 1, "tone_pi_2": 2, "zero": 3}[trial["drive"]],
                int(trial["seed"]),
                source,
                int(trial["channel"]) - 1,
            )
            if (
                tuple(
                    snapshot[k]
                    for k in ("topology", "config", "drive", "seed", "source", "channel")
                )
                != identity_fields
            ):
                raise ValueError("snapshot/ledger physics provenance mismatch")
            checksums += int(checked)
            frame_count += 1
            tick = snapshot["tick"]
            if (
                tick != frame_index * 8
                or snapshot["fnv"] != hashes[identity, tick]
                or snapshot["synthetic"]
            ):
                raise ValueError("snapshot tick/hash/provenance mismatch")
            neurons = snapshot["neurons"]
            palette = b"".join(n[4] for n in neurons)
            if first_palette is None:
                first_palette = palette
            elif palette != first_palette:
                raise ValueError("persistent structure changed")
            if any(not math.isfinite(value) for n in neurons for value in n[5:23]):
                raise ValueError("nonfinite observation field")
            if any(n[23] for n in neurons):
                raise ValueError("nonfinite observation count")
            close(max(max(n[5:9]) for n in neurons), float(metrics[tick]["peak"]))
            close(max(max(n[9:13]) for n in neurons), float(metrics[tick]["velocity_peak"]))
            close(sum(n[21] for n in neurons), float(metrics[tick]["sum_u2"]))
            close(sum(n[22] for n in neurons), float(metrics[tick]["sum_v2"]))
            close(neurons[source][13], float(metrics[tick]["source_u27"]))
            close(neurons[source][17], float(metrics[tick]["source_v27"]))
            for index, n in enumerate(neurons):
                peak = max(n[5:9])
                sampled_peaks[index] = max(sampled_peaks[index], peak)
                if sampled_first[index] is None and peak >= 1e-6:
                    sampled_first[index] = tick
        for index, row in enumerate(spatial):
            xyz = coordinates(index, trial["topology"])
            if int(row["id"]) != index or tuple(int(row[c]) for c in "xyz") != xyz:
                raise ValueError("spatial ID mismatch")
            delta = [abs(a - b) for a, b in zip(xyz, origin, strict=True)]
            hops = sum(delta) if trial["topology"] == "Cubic6" else max(delta)
            if int(row["hops"]) != hops:
                raise ValueError("hop-distance mismatch")
            close(float(row["euclidean"]), math.sqrt(sum(d * d for d in delta)))
            if float(row["peak"]) + 1e-30 < sampled_peaks[index]:
                raise ValueError("spatial peak smaller than a recorded peak")
            arrival = int(row["arrival_tick"]) if row["arrival_tick"] else None
            if arrival is not None and not hops <= arrival <= 160:
                raise ValueError("noncausal/invalid threshold arrival")
            if sampled_first[index] is not None and (
                arrival is None or arrival > sampled_first[index]
            ):
                raise ValueError("arrival inconsistent with observed state")
        energy = sum(float(r["force"]) ** 2 * 0.1 for r in metrics)
        is_tone = trial["drive"].startswith("tone")
        close(energy, 1.0 if is_tone else 0.0)
        if trial["drive"] == "zero" and any(
            float(r["sum_u2"]) != 0 or float(r["sum_v2"]) != 0 for r in metrics
        ):
            raise ValueError("zero-input drift")
        gain, phase = None, None
        if is_tone:
            omega = math.pi / (4 if trial["drive"] == "tone_pi_4" else 2)
            window = metrics[81:]
            response = [float(r["source_u27"]) for r in window]
            force = [float(r["force"]) for r in window]
            gain = math.sqrt(sum(u * u for u in response) / sum(s * s for s in force))
            cosine = sum(
                u * math.cos(omega * int(r["tick"]) * 0.1)
                for u, r in zip(response, window, strict=True)
            )
            sine = sum(
                u * math.sin(omega * int(r["tick"]) * 0.1)
                for u, r in zip(response, window, strict=True)
            )
            phase = math.atan2(cosine, sine)
        activated = [r for r in spatial if r["arrival_tick"]]
        summary.append(
            {
                **trial,
                "valid": True,
                "peak": max(float(r["peak"]) for r in metrics),
                "end_peak": float(metrics[-1]["peak"]),
                "end_squared": float(metrics[-1]["sum_u2"]),
                "activated_neurons": len(activated),
                "furthest_threshold_hops": max((int(r["hops"]) for r in activated), default=0),
                "input_energy": energy,
                "source_gain": gain,
                "source_phase": phase,
            }
        )
        for hops in sorted({int(r["hops"]) for r in spatial}):
            shell = [r for r in spatial if int(r["hops"]) == hops]
            shells.append(
                {
                    "case": identity,
                    "topology": trial["topology"],
                    "hops": hops,
                    "neurons": len(shell),
                    "peak_min": min(float(r["peak"]) for r in shell),
                    "peak_max": max(float(r["peak"]) for r in shell),
                    "threshold_reached": sum(bool(r["arrival_tick"]) for r in shell),
                }
            )
        if identity % 20 == 0:
            print(f"verified case {identity}", flush=True)
    for name, data in (("case-summary.csv", summary), ("shell-summary.csv", shells)):
        with (folder / name).open("w", newline="") as stream:
            writer = csv.DictWriter(stream, fieldnames=list(data[0]))
            writer.writeheader()
            writer.writerows(data)
    result = {
        "cases": 111,
        "frames_verified": frame_count,
        "independent_checksum_frames": checksums,
        "core_readback_checksum_frames": frame_count,
        "numerical_screen_failures": 0,
        "persistent_changes": 0,
        "memory_or_resonance_claim": False,
        "raw_hash_ledger_sha256": hashlib.sha256((folder / "hashes.csv").read_bytes()).hexdigest(),
    }
    (folder / "analysis-summary.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n"
    )
    return result


def verify_witness(folder: Path, recording: Path) -> dict:
    """Compare actual Bevy UI receipts with recorded values, including LIVE hashes."""
    if not (folder / "COMPLETE").exists():
        raise ValueError("viewer witness incomplete")
    frames = {
        frame["tick"]: frame
        for frame in (read_snapshot(p) for p in sorted(recording.glob("*.qrs")))
    }
    audit = rows(folder / "audit.csv")
    if len(audit) != 67:
        raise ValueError("viewer did not cover 21 ticks x 3 selections plus 4 seeks")
    for row in audit:
        frame = frames[int(row["tick"])]
        neuron = frame["neurons"][int(row["id"])]
        if row["fnv1a"] != frame["fnv"]:
            raise ValueError("viewer/headless scientific state mismatch")
        if row["palette"] != "".join(format(value, "x") for value in neuron[4]):
            raise ValueError("displayed palette mismatch")
        close(float(row["u27"]), neuron[13])
        close(float(row["v27"]), neuron[17])
        axis = None if row["axis"] == "None" else int(row["axis"][5:-1])
        visible = 0
        for n in frame["neurons"]:
            if row.get("nodes", "true") == "false":
                continue
            xyz = n[1:4]
            shell = any(
                d > 1 and (p == 0 or p + 1 == d) for p, d in zip(xyz, frame["dims"], strict=True)
            )
            if axis is not None and xyz[axis] != int(row["plane"]):
                continue
            region = int(row["region"])
            if (region == 1 and not shell) or (region == 2 and shell):
                continue
            visible += 1
        if int(row["visible_nodes"]) != visible:
            raise ValueError("viewer filter did not match coordinates")
    result = {
        "rows": len(audit),
        "ticks": len({r["tick"] for r in audit}),
        "modes": sorted({r["mode"] for r in audit}),
        "selected_ids": sorted({int(r["id"]) for r in audit}),
        "headless_hashes_equal": True,
        "palettes_equal": True,
        "numeric_fields_equal": True,
        "filters_verified": True,
    }
    (folder / "verification.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("--recording", type=Path)
    args = parser.parse_args()
    result = verify_witness(args.folder, args.recording) if args.recording else verify(args.folder)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
