"""Verify every decay tick and export a self-contained, viewer-free agent report."""

import argparse
import csv
import filecmp
import hashlib
import json
import math
import shutil
import struct
import zipfile
from pathlib import Path

from analysis.experiment_0003 import fnv

MODEL = b"qr4-decay-v1;physics=0002;extension=2c39004"
POPULATION = 4096
LAST = 3072
DT = 0.1
THRESHOLD = 1e-6
HEADER_SIZE = 52 + len(MODEL) + POPULATION * 64 + 8
NEURON = struct.Struct("<6dI")
FRAME_SIZE = 16 + POPULATION * NEURON.size


def rows(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def close(a, b):
    if (
        not math.isfinite(a)
        or not math.isfinite(b)
        or not math.isclose(a, b, rel_tol=1e-11, abs_tol=1e-30)
    ):
        raise ValueError(f"measurement mismatch {a} != {b}")


def header(data, size):
    """Validate fixed provenance, explicit bound and the actual persistent palette."""
    if len(data) != HEADER_SIZE or data[:8] != b"QR4DEC01":
        raise ValueError("invalid decay header")
    if (
        struct.unpack_from("<I", data, 8)[0] != 1
        or struct.unpack_from("<QdQII4BI", data, 12)
        != (7, DT, LAST, 2182, 2186, 1, 2, 0, 0, len(MODEL))
        or data[52 : 52 + len(MODEL)] != MODEL
        or size != HEADER_SIZE + (LAST + 1) * FRAME_SIZE
    ):
        raise ValueError("decay provenance/size mismatch")
    if fnv(data[:-8]) != struct.unpack_from("<Q", data, HEADER_SIZE - 8)[0]:
        raise ValueError("header checksum mismatch")
    palette = data[52 + len(MODEL) : -8]
    if set(palette) != {1}:
        raise ValueError("persistent palette is not homogeneous all-red")
    return hashlib.sha256(palette).hexdigest()


def decode_frame(data, tick):
    """Decode exactly one indexed frame; explicitly reject corruption and wrong ticks."""
    if len(data) != FRAME_SIZE or struct.unpack_from("<Q", data)[0] != tick:
        raise ValueError("frame length/tick mismatch")
    checksum = struct.unpack_from("<Q", data, FRAME_SIZE - 8)[0]
    if fnv(data[:-8]) != checksum:
        raise ValueError("frame checksum mismatch")
    return NEURON.iter_unpack(data[8:-8]), f"{checksum:016x}"


def probe_ids():
    selected = {x + 16 * (8 + 16 * 8) for x in range(16)}
    selected.update(8 + 16 * (y + 16 * 8) for y in (0, 4, 8, 12, 15))
    selected.update(8 + 16 * (8 + 16 * z) for z in (0, 4, 8, 12, 15))
    selected.update(x + 16 * (y + 16 * z) for x in (0, 15) for y in (0, 15) for z in (0, 15))
    return sorted(selected)


def scan(folder, report, metrics):
    probes = set(probe_ids())
    peak, velocity_peak = [0.0] * POPULATION, [0.0] * POPULATION
    peak_tick, velocity_tick = [0] * POPULATION, [0] * POPULATION
    arrival = [None] * POPULATION
    last_above = 0
    with (
        (folder / "replay.q4d").open("rb") as stream,
        (report / "probes.csv").open("w", newline="") as probe_file,
        (report / "frame_hashes.csv").open("w", newline="") as hash_file,
    ):
        palette_sha = header(stream.read(HEADER_SIZE), (folder / "replay.q4d").stat().st_size)
        probe_writer = csv.writer(probe_file)
        probe_writer.writerow(
            (
                "tick",
                "time",
                "id",
                "x",
                "y",
                "z",
                "peak",
                "velocity_peak",
                "u27",
                "v27",
                "sum_u2",
                "sum_v2",
            )
        )
        hash_writer = csv.writer(hash_file)
        hash_writer.writerow(("tick", "fnv1a"))
        for tick in range(LAST + 1):
            values, checksum = decode_frame(stream.read(FRAME_SIZE), tick)
            hash_writer.writerow((tick, checksum))
            observed = []
            max_u = max_v = sum_u2 = sum_v2 = 0.0
            active = 0
            for n, value in enumerate(values):
                u, v, u27, v27, squared, vsquared, nonfinite = value
                if nonfinite or any(not math.isfinite(x) for x in value):
                    raise ValueError("nonfinite decay observation")
                if not 0 <= u <= 100 or not 0 <= v <= 100:
                    raise ValueError("decay amplitude/velocity screen failed")
                for p, signed, q in ((u, u27, squared), (v, v27, vsquared)):
                    if abs(signed) > p * (1 + 1e-12) + 1e-300 or not p * p * (
                        1 - 1e-12
                    ) <= q <= 64 * p * p * (1 + 1e-12):
                        raise ValueError("invalid per-neuron aggregate bounds")
                if tick == 0:
                    expected = 1.0 if n in (2182, 2186) else 0.0
                    if (u, v, u27, v27, squared, vsquared) != (
                        expected,
                        0.0,
                        expected,
                        0.0,
                        expected,
                        0.0,
                    ):
                        raise ValueError("initial dual impulse mismatch")
                max_u, max_v = max(max_u, u), max(max_v, v)
                sum_u2 += squared
                sum_v2 += vsquared
                active += int(u >= THRESHOLD)
                if u > peak[n]:
                    peak[n], peak_tick[n] = u, tick
                if v > velocity_peak[n]:
                    velocity_peak[n], velocity_tick[n] = v, tick
                if u >= THRESHOLD and arrival[n] is None:
                    arrival[n] = tick
                if n in probes:
                    probe_writer.writerow(
                        (
                            tick,
                            f"{tick * DT:.17e}",
                            n,
                            n % 16,
                            n // 16 % 16,
                            n // 256,
                            *[f"{x:.17e}" for x in value[:6]],
                        )
                    )
                observed.append(value)
            row = metrics[tick]
            for actual, name in (
                (max_u, "peak"),
                (max_v, "velocity_peak"),
                (sum_u2, "sum_u2"),
                (sum_v2, "sum_v2"),
            ):
                close(actual, float(row[name]))
            close(observed[2184][2], float(row["mid_u27"]))
            close(observed[2184][3], float(row["mid_v27"]))
            close(tick * DT, float(row["time"]))
            if int(row["nonfinite"]) or active != int(row["active_neurons"]):
                raise ValueError("nonfinite/active-neuron metric mismatch")
            quiet = max_u < THRESHOLD and max_v < THRESHOLD
            if (row["below_threshold"] == "true") != quiet:
                raise ValueError("quiet threshold flag mismatch")
            if not quiet:
                last_above = tick
            if tick % 512 == 0:
                print(f"verified every-tick stream through {tick}/{LAST}", flush=True)
    with (report / "spatial_summary.csv").open("w", newline="") as output:
        writer = csv.writer(output)
        writer.writerow(
            (
                "id",
                "x",
                "y",
                "z",
                "first_arrival_tick",
                "cumulative_peak",
                "peak_tick",
                "cumulative_velocity_peak",
                "velocity_peak_tick",
                "final_peak",
                "final_velocity_peak",
                "final_u27",
                "final_v27",
                "final_sum_u2",
                "final_sum_v2",
            )
        )
        for n, final in enumerate(observed):
            writer.writerow(
                (
                    n,
                    n % 16,
                    n // 16 % 16,
                    n // 256,
                    "" if arrival[n] is None else arrival[n],
                    f"{peak[n]:.17e}",
                    peak_tick[n],
                    f"{velocity_peak[n]:.17e}",
                    velocity_tick[n],
                    *[f"{x:.17e}" for x in final[:6]],
                )
            )
    return {
        "frames": LAST + 1,
        "neurons_per_frame": POPULATION,
        "probe_count": len(probes),
        "last_at_or_above": last_above,
        "quiet_tail": LAST - last_above,
        "sustained_quiet": LAST - last_above >= 128,
        "exact_zero": max_u == 0 and max_v == 0,
        "final_peak": max_u,
        "final_velocity_peak": max_v,
        "palette_sha256": palette_sha,
        "first_arrival_count": sum(x is not None for x in arrival),
    }


def package(report):
    """Write portable file hashes and a deterministic ZIP without the large stream."""
    files = sorted(p for p in report.iterdir() if p.is_file() and p.name != "FILE_SHA256.csv")
    with (report / "FILE_SHA256.csv").open("w", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(("file", "sha256"))
        for p in files:
            writer.writerow((p.name, hashlib.sha256(p.read_bytes()).hexdigest()))
    archive = report.parent / "AGENT_REPORT.zip"
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        for p in sorted(report.iterdir()):
            if p.is_file():
                info = zipfile.ZipInfo(p.name, date_time=(2026, 11, 13, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                zipped.writestr(info, p.read_bytes())
    return archive


def write_report(folder, report, result, metrics):
    checkpoints = (0, 512, 1024, 1536, 2048, 2560, 2578, 2579, 2816, 3072)
    table = "\n".join(
        f"| {t} | {t * DT:.1f} | {float(metrics[t]['peak']):.8e} | "
        f"{float(metrics[t]['velocity_peak']):.8e} | {metrics[t]['active_neurons']} |"
        for t in checkpoints
    )
    criterion = "met" if result["sustained_quiet"] else "not met"
    if result["quiet_tail"]:
        observation = (
            f"Both global maxima remain below 1e-6 from tick "
            f"**{result['last_at_or_above'] + 1}** through 3072: "
            f"**{result['quiet_tail']} consecutive ticks**."
        )
    else:
        observation = "There is no final below-threshold tail at the observation horizon."
    prefix_note = (
        (
            f"**{result['prefix_matches']}** full-state checkpoints through 1024, "
            "all matching (Rust full u/v/palette comparison)."
        )
        if result["prefix_matches"]
        else "Not supplied; no prefix equivalence is asserted."
    )
    text = f"""# Cube experiment report: 3,072-tick frozen-medium decay

## Read this first

{observation}
Final max|u|={result["final_peak"]:.12e}; final max|v|={result["final_velocity_peak"]:.12e}.
Exact-zero status: **{result["exact_zero"]}**. The 128-tick sustained-quiet criterion
is **{criterion}**. This describes the defined discrete simulation, not a proof
of information erasure, physical accuracy, general stability, or learned memory.

The original dt 0.1/dt 0.05 study failed all 8 long-window accuracy comparisons;
the homogeneous AB+ comparison reached 29.085% relative L2 disagreement at
physical time 102.4. **No new convergence test was run at time 307.2.** Longer
integration does not fix that earlier failure. Please retain this limitation
when interpreting decay times and late spatial patterns.

## Exact experiment identity

- Extension registration: `2c39004`, `EXPERIMENT_0004_EXTENSION.md` copied as `RULES.md`.
- Frozen parent: Experiment 0004; unchanged Rust `propagation_0004::Medium`.
- Population 4096, open Cubic6 16x16x16, ID=x+16(y+16z), no wrapping.
- Each neuron owns its own 8x8 persistent spatial palette. In **this fixture all
  64 modules of every neuron are red/value1**. Palettes stayed fixed; differing
  activity colors never mean that the palette learned or changed.
- Seed 7, initial A 2182=(6,8,8) and B 2186=(10,8,8), internal module 27=(3,3),
  channel 1, both initial displacements+1; all other displacements/velocities 0.
- No subsequent forcing, plasticity, adaptation, lesion, or topology change.
- dt 0.1, 3072 updates, time 307.2 in **nondimensional simulation units**.
- All integer ticks 0..3072 recorded: 3073 frames, no interpolation/subsampling.
- Midpoint 2184=(8,8,8). Only the independent active channel is stored;
  three inactive channels remain zero by the exact uncoupled equations.
- Dual input has sum(u²) = 2 initially; this is not physical energy.
- CPU Rust simulation, serial synchronous old-state updates. GPU only renders.

## Rules sufficient to interpret or reimplement the model

For internal module i, palette P_i, active color channel k=1:

    omega_i = .8 + .08*P_i + .04*occupied_internal_neighbors + .12*[P_i=1]
    gamma_i = .08 + .01*empty_internal_neighbors + .02*[P_i!=1]
    g_i = .5 if P_i=0, 1 if P_i=1, .75 otherwise
    a_i = -omega_i²*u_i - gamma_i*v_i
          + (.6/4) sum_internal g_i*g_j*(u_j-u_i)
          + (.4/6) sum_external g_i*g_j*(u_j-u_i)
    v_next = v + .1*a
    u_next = u + .1*v_next

Internal neighbors: four cardinal 8x8 module neighbors, open edges. External
neighbors: six face-sharing neurons, same internal module index; fixed divisor 6,
including at boundaries (no renormalization by actual degree). Neighbors ascend
by stable ID. Compute every module from the old state, then swap buffers.
The frozen medium is linear. Interference is not an experience-dependent change.
`PHYSICS.rs` and the recorded `RECORDER.rs` are included for exact inspection.

## What the numbers mean

u is signed displacement; v signed velocity-like state. `peak` is max absolute
u over all active modules, not a selected module or averaged signal. `velocity_peak`
is the analogous max|v|. Per-neuron peaks aggregate its 64 modules. `u27`/`v27`
are signed values at internal module 27. `sum_u2`/`sum_v2` are squared-field
sums, not conserved physical energy. `active_neurons` counts instantaneous
neurons with max-module|u|>=1e-6, not cumulative reach or memory capacity.

Quiet requires **both** global maxima<1e-6. A sustained quiet tail requires at
least 128 final consecutive quiet ticks. First crossing alone is not sufficient,
as oscillations can cross a threshold repeatedly. Exact-zero status is false.
The viewer's fixed log-color floor 1e-8 is a separate display convention.

| Tick | Simulation time | Global peak u | Global peak v | Active neurons |
|---|---|---|---|---|
{table}

## Evidence and verification boundaries

Independent Python decoded/checksummed every recorded frame, verified contiguous
indices, actual fixed palette, per-neuron aggregate bounds, global metrics,
midpoint values, instantaneous active counts, and the quiet-tail decision.
Stream SHA-256: `{result["stream_sha256"]}`.
Two raw runs compare byte-for-byte identical: **{result["repeated_raw_identical"]}**.
Original-prefix comparisons supplied: {prefix_note}

The stream stores per-neuron measured aggregates and signed module 27, with the
persistent palette once in a checked header. It does **not** store all 64 module
u/v values at every tick and is not a restart checkpoint. Independent checks
validate exported observations/consistency, not a second integration of physics.
The CSV data below suffice for numerical/temporal/spatial analysis of those
observations without the viewer, video, or the 650 MB stream. No learning, resilience,
physical-water, emergent-computation, or converged-long-time claim is supported.
A native replay audit, if bundled as `native_audit.json`, is separate UI evidence.

## Files for another agent

- `metrics.csv`: every tick's global values, active counts, midpoint signed u/v.
- `probes.csv`: every tick for {result["probe_count"]} explicitly identified neurons;
  all 16 sites on the X source line, selected Y/Z-axis points, and 8 corners.
  Coordinates, signed u27/v27, max-module magnitudes and squared sums included.
- `spatial_summary.csv`: all 4096 neurons, first 1e-6 arrival, cumulative peaks and
  their ticks, final fields. Blank arrival means no threshold crossing.
- `frame_hashes.csv`: all 3073 indexed frame checksums; FNV detects corruption,
  not cryptographic authenticity. Complete-stream SHA-256 is recorded above.
- `metadata.json`, `verification.json`, `prefix.csv`, `RULES.md`, `PHYSICS.rs`,
  `RECORDER.rs`, `FILE_SHA256.csv`: identity, actual checks, rules and source.
- The ZIP excludes the large binary recording. Check file hashes before analysis.

Useful follow-up analyses include displacement-versus-velocity decay envelopes,
late threshold recrossings, source/midpoint/boundary differences, and comparing
homogeneous response to a separately registered heterogeneous or refined-dt run.
A persistent trace experiment would require an explicitly new plasticity rule;
this frozen-medium run cannot establish one.
"""
    (report / "REPORT.md").write_text(text)
    metadata = {
        "experiment": "0004-3072-tick-extension",
        "registration": "2c39004",
        "repository": "https://github.com/Cognis-11-Inc/qr-neural-memory-system",
        "dt": DT,
        "last_tick": LAST,
        "frame_stride": 1,
        "configuration": "homogeneous_all_red",
        "seed": 7,
        "arm": "AB+",
        "sources": [2182, 2186],
        "module": 27,
        "channel": 1,
        "probe_ids": probe_ids(),
        "quiet_threshold": THRESHOLD,
        "quiet_tail_minimum": 128,
        "full_module_state_recorded": False,
        "prior_long_window_convergence": "failed; not retested at extension horizon",
        **result,
    }
    (report / "metadata.json").write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n")
    (report / "verification.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    for source, name in (
        (folder / "metrics.csv", "metrics.csv"),
        (folder / "prefix.csv", "prefix.csv"),
        (folder / "registration.md", "RULES.md"),
        (folder / "recorder.rs", "RECORDER.rs"),
        (Path(__file__).resolve().parents[1] / "src/propagation_0004.rs", "PHYSICS.rs"),
    ):
        shutil.copyfile(source, report / name)
    package(report)


def quiet_from_metrics(metrics):
    """Use the final quiet tail, including velocity and any threshold recrossings."""
    if not metrics:
        raise ValueError("empty decay trace")
    last = int(metrics[-1]["tick"])
    above = [
        int(r["tick"])
        for r in metrics
        if float(r["peak"]) >= THRESHOLD or float(r["velocity_peak"]) >= THRESHOLD
    ]
    tail = last - max(above) if above else len(metrics)
    return {
        "quiet_tail": tail,
        "sustained_quiet": tail >= 128,
        "exact_zero": float(metrics[-1]["peak"]) == 0 and float(metrics[-1]["velocity_peak"]) == 0,
    }


def verify(folder, compare=None):
    if not (folder / "COMPLETE").exists():
        raise ValueError("incomplete decay run")
    metrics = rows(folder / "metrics.csv")
    if [int(r["tick"]) for r in metrics] != list(range(LAST + 1)):
        raise ValueError("every-tick metrics coverage mismatch")
    report = folder / "agent-report"
    report.mkdir(exist_ok=True)
    result = scan(folder, report, metrics)
    if any(result[key] != value for key, value in quiet_from_metrics(metrics).items()):
        raise ValueError("quiet-tail independent metric recomputation mismatch")
    summary = rows(folder / "summary.csv")[0]
    for key in ("last_at_or_above", "quiet_tail"):
        if int(summary[key]) != result[key]:
            raise ValueError("quiet-tail decision mismatch")
    for key in ("sustained_quiet", "exact_zero"):
        if (summary[key] == "true") != result[key]:
            raise ValueError("quiet/zero decision mismatch")
    close(float(summary["final_peak"]), result["final_peak"])
    close(float(summary["final_velocity_peak"]), result["final_velocity_peak"])
    prefix = rows(folder / "prefix.csv")
    if prefix and (
        [int(r["tick"]) for r in prefix] != [0, 8, 16, 32, 64, 128, 160, 256, 384, 512, 768, 1024]
        or any(r["full_state_equal"] != "true" for r in prefix)
    ):
        raise ValueError("original-prefix regression failed")
    result["prefix_matches"] = len(prefix)
    with (folder / "replay.q4d").open("rb") as stream:
        result["stream_sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
    result["repeated_raw_identical"] = None
    if compare:
        for name in (
            "replay.q4d",
            "metrics.csv",
            "summary.csv",
            "prefix.csv",
            "recorder.rs",
            "registration.md",
            "COMPLETE",
        ):
            if not filecmp.cmp(folder / name, compare / name, shallow=False):
                raise ValueError(f"repeated raw run differs: {name}")
        result["repeated_raw_identical"] = True
    write_report(folder, report, result, metrics)
    (folder / "analysis-summary.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n"
    )
    return result


def verify_audit(folder, audit):
    report = folder / "agent-report"
    hashes = {int(r["tick"]): r["fnv1a"] for r in rows(report / "frame_hashes.csv")}
    metrics = rows(folder / "metrics.csv")
    data = rows(audit)
    if [int(r["tick"]) for r in data] != list(range(LAST + 1)) or not audit.with_suffix(
        ".complete"
    ).exists():
        raise ValueError("native audit omitted/duplicated ticks or did not complete")
    for tick, r in enumerate(data):
        if r["fnv1a"] != hashes[tick] or r["contiguous"] != "true" or int(r["id"]) != 2184:
            raise ValueError("native displayed tick/hash/selection mismatch")
        for key, expected in (
            ("peak", "peak"),
            ("velocity_peak", "velocity_peak"),
            ("u27", "mid_u27"),
            ("v27", "mid_v27"),
        ):
            close(float(r[key]), float(metrics[tick][expected]))
    result = {
        "native_ticks_verified": len(data),
        "every_tick_contiguous": True,
        "displayed_observation_hashes_equal": True,
        "selected_neuron": 2184,
        "numeric_fields_equal": True,
    }
    (report / "native_audit.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    package(report)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("--compare", type=Path)
    parser.add_argument("--audit-only", type=Path)
    args = parser.parse_args()
    result = (
        verify_audit(args.folder, args.audit_only)
        if args.audit_only
        else verify(args.folder, args.compare)
    )
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
