"""Independent QR5 decoding, energy reconstruction and preregistered decisions.

No integration is performed here. Full-field checks are at stored checkpoints;
every-tick metrics and selected probes are checked separately.
"""

import argparse
import csv
import hashlib
import json
import math
import struct
from pathlib import Path

import numpy as np

FIELDS = 4096 * 64
CHECKPOINTS = [0, 0.8, 3.2, 16, 32, 102.4, 307.2]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fnv(data):
    value = 0xCBF29CE484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def decode(path):
    data = path.read_bytes()
    require(len(data) >= 56 and data[:8] == b"QR5FLD01", "magic/length")
    schema, tick, dt_bits, seed, config, length = struct.unpack_from("<6Q", data, 8)
    require(schema == 1 and 0 < length <= 200, "schema/label length")
    require(len(data) == 56 + length + 17 * FIELDS + 8, "exact frame size")
    require(fnv(data[:-8]) == struct.unpack_from("<Q", data, len(data) - 8)[0], "checksum")
    dt = struct.unpack("<d", struct.pack("<Q", dt_bits))[0]
    require(dt in (0.1, 0.05, 0.025) and tick <= 12288 and config <= 2, "metadata")
    start = 56 + length
    palette = np.frombuffer(data, dtype="u1", count=FIELDS, offset=start).reshape(4096, 8, 8)
    u = np.frombuffer(data, dtype="<f8", count=FIELDS, offset=start + FIELDS).reshape(4096, 8, 8)
    v = np.frombuffer(data, dtype="<f8", count=FIELDS, offset=start + 9 * FIELDS).reshape(
        4096, 8, 8
    )
    require(np.all(palette <= 4), "palette")
    require(np.isfinite(u).all() and np.isfinite(v).all(), "nonfinite")
    require(max(np.max(np.abs(u)), np.max(np.abs(v))) <= 100, "boundedness")
    return dict(
        tick=tick,
        dt=dt,
        seed=seed,
        config=config,
        label=data[56:start].decode(),
        palette=palette,
        u=u,
        v=v,
    )


def coefficients(palette):
    occupied = palette != 0
    neighbors = np.zeros(palette.shape, dtype=np.int8)
    degree = np.zeros(palette.shape, dtype=np.int8)
    for axis in (1, 2):
        left = [slice(None)] * 3
        right = [slice(None)] * 3
        left[axis], right[axis] = slice(None, -1), slice(1, None)
        left, right = tuple(left), tuple(right)
        neighbors[left] += occupied[right]
        neighbors[right] += occupied[left]
        degree[left] += 1
        degree[right] += 1
    omega = 0.8 + 0.08 * palette + 0.04 * neighbors + 0.12 * (palette == 1)
    gamma = 0.08 + 0.01 * (degree - neighbors) + 0.02 * (palette != 1)
    conductance = np.where(palette == 0, 0.5, np.where(palette == 1, 1.0, 0.75))
    return omega**2, gamma, conductance


def energy(palette, u, v):
    omega2, _, conductance = coefficients(palette)
    kinetic = 0.5 * float(np.sum(v * v))
    onsite = 0.5 * float(np.sum(omega2 * u * u))
    local = 0.5 * v * v + 0.5 * omega2 * u * u
    coupling = 0.0
    # Independent tensor slicing: no engine edge list is used.
    for field, g, partition, axes, coefficient in (
        (u, conductance, local, (1, 2), 0.15),
        (
            u.reshape(16, 16, 16, 8, 8),
            conductance.reshape(16, 16, 16, 8, 8),
            local.reshape(16, 16, 16, 8, 8),
            (0, 1, 2),
            0.4 / 6,
        ),
    ):
        for axis in axes:
            left, right = [slice(None)] * field.ndim, [slice(None)] * field.ndim
            left[axis], right[axis] = slice(None, -1), slice(1, None)
            left, right = tuple(left), tuple(right)
            potential = 0.5 * coefficient * g[left] * g[right] * (field[left] - field[right]) ** 2
            coupling += float(np.sum(potential))
            partition[left] += 0.5 * potential
            partition[right] += 0.5 * potential
    return kinetic, onsite, coupling, np.sum(local, axis=(1, 2))


def rows(path):
    with path.open() as stream:
        return list(csv.DictReader(stream))


def field_error(a, b):
    difference = sum(float(np.sum((a[k] - b[k]) ** 2)) for k in ("u", "v"))
    denominator = sum(float(np.sum(b[k] ** 2)) for k in ("u", "v"))
    return math.sqrt(difference / max(denominator, 1e-30))


def checkpoint(folder, time):
    sample = next(folder.glob("*.q5f"))
    # dt read from fixed header, then full decoding of requested checkpoint.
    with sample.open("rb") as stream:
        dt = struct.unpack_from("<d", stream.read(32), 24)[0]
    return decode(folder / f"{round(time / dt):06d}.q5f")


def convergence(root, prefix):
    result = []
    for time in CHECKPOINTS[1:]:
        if not (root / f"{prefix}-0.025" / f"{round(time / 0.025):06d}.q5f").exists():
            continue
        frames = [checkpoint(root / f"{prefix}-{dt}", time) for dt in (0.1, 0.05, 0.025)]
        coarse, fine = field_error(frames[0], frames[1]), field_error(frames[1], frames[2])
        ratio = fine / coarse if coarse > 1e-10 else 0
        tolerance = 0.1 if time > 102.4 else 0.05
        passed = coarse <= tolerance and fine <= tolerance
        if time in (32, 102.4, 307.2):
            passed = passed and ratio <= 0.35
        result.append(
            dict(fixture=prefix, time=time, coarse=coarse, fine=fine, ratio=ratio, passed=passed)
        )
    return result


def audit_run(folder):
    metrics = rows(folder / "metrics.csv")
    before, after = (
        (folder / "before.palette").read_bytes(),
        (folder / "after.palette").read_bytes(),
    )
    require(len(before) == FIELDS and len(after) == FIELDS, "palette dimensions")
    frames = sorted(folder.glob("*.q5f"))
    initial = decode(frames[0])
    dt = initial["dt"]
    e0 = sum(energy(initial["palette"], initial["u"], initial["v"])[:3])
    max_energy = 0.0
    max_drift = 0.0
    for tick, row in enumerate(metrics):
        require(int(row["tick"]) == tick and abs(float(row["time"]) - tick * dt) < 1e-9, "timeline")
        values = {k: float(v) for k, v in row.items()}
        require(all(math.isfinite(x) for x in values.values()), "nonfinite metrics")
        total = sum(values[k] for k in ("kinetic", "onsite", "coupling"))
        drift = (
            total
            - e0
            + values["loss"]
            + values["sponge_loss"]
            - values["work"]
            - values["switch_work"]
        )
        require(abs(drift - values["drift"]) < 1e-11, "energy balance arithmetic")
        require(values["loss"] >= 0 and values["sponge_loss"] >= 0, "negative damping")
        require(0 <= values["peak"] <= 100 and 0 <= values["velocity_peak"] <= 100, "metric bound")
        max_drift = max(max_drift, abs(drift))
        max_energy = max(max_energy, total)
    for file in frames:
        frame = decode(file)
        require(frame["label"] == folder.name and frame["dt"] == dt, "provenance")
        require(
            frame["seed"] == initial["seed"] and frame["config"] == initial["config"],
            "mixed seed/config",
        )
        row = metrics[frame["tick"]]
        kinetic, onsite, coupling, local = energy(frame["palette"], frame["u"], frame["v"])
        for key, value in zip(
            ("kinetic", "onsite", "coupling"), (kinetic, onsite, coupling), strict=True
        ):
            require(abs(float(row[key]) - value) <= 1e-10 * max(1, value), f"independent {key}")
        require(abs(np.sum(local) - kinetic - onsite - coupling) < 1e-10, "local partition")
        require(abs(float(row["peak"]) - np.max(np.abs(frame["u"]))) < 1e-14, "global displacement")
        require(
            abs(float(row["velocity_peak"]) - np.max(np.abs(frame["v"]))) < 1e-14, "global velocity"
        )
    probe_rows = rows(folder / "probes.csv")
    require(len(probe_rows) == 11 * len(metrics), "probe coverage")
    spatial = rows(folder / "spatial.csv")
    require(len(spatial) == 4096, "spatial coverage")
    spectra, events = [], []
    for ident in (0, 2180, 2182, 2183, 2184, 2185, 2186, 2188, 2191, 2440, 4095):
        trace = [r for r in probe_rows if int(r["id"]) == ident]
        require([int(r["tick"]) for r in trace] == list(range(len(metrics))), "probe timeline")
        values = [float(r["peak"]) for r in trace]
        crossings = sum(
            x >= 1e-6 and (i == 0 or values[i - 1] < 1e-6) for i, x in enumerate(values)
        )
        last, peaks = -1e6, 0
        for i in range(1, len(values) - 1):
            if (
                values[i] >= 1e-6
                and values[i] > values[i - 1]
                and values[i] > values[i + 1]
                and i * dt - last >= 1 - 1e-12
            ):
                peaks += 1
                last = i * dt
        require(
            crossings == int(spatial[ident]["crossings"])
            and peaks == int(spatial[ident]["separated_peaks"]),
            "event decisions",
        )
        first = next((i * dt for i, x in enumerate(values) if x >= 1e-6), None)
        recorded = spatial[ident]["first_crossing"]
        require(
            (first is None and recorded == "")
            or (first is not None and abs(first - float(recorded)) < 1e-9),
            "first arrival",
        )
        events.append(
            dict(
                id=ident,
                crossings=crossings,
                subsequent=max(0, crossings - 1),
                separated_peaks=peaks,
            )
        )
        for omega in (0.8, 1.2, 1.6):
            coefficient = sum(
                float(r["u27"])
                * complex(math.cos(omega * float(r["time"])), -math.sin(omega * float(r["time"])))
                * dt
                for r in trace
                if float(r["time"]) <= 32 + 1e-9
            )
            spectra.append(
                dict(
                    id=ident,
                    omega=omega,
                    amplitude=abs(coefficient),
                    phase=math.atan2(coefficient.imag, coefficient.real),
                )
            )
    if (folder / "history.bin").exists():
        history = np.frombuffer((folder / "history.bin").read_bytes(), dtype="<f8").reshape(
            4096, 64
        )
        p = np.frombuffer(before, dtype="u1").reshape(4096, 64)
        expected = p.copy()
        if folder.name.startswith("c-time-"):
            expected[:, 0] = (expected[:, 0] + 1) % 5
        elif folder.name.startswith(("c-local-", "c-zero-")):
            module = np.argmax(history, axis=1)
            selected = np.flatnonzero(history[np.arange(4096), module] >= 1e-8)
            expected[selected, module[selected]] = (expected[selected, module[selected]] + 1) % 5
        require(expected.tobytes() == after, "independent local-history rule")
        transition = rows(folder / "switch.csv")[0]
        require(
            abs(
                float(transition["post_energy"])
                - float(transition["pre_energy"])
                - float(transition["work"])
            )
            < 1e-11,
            "switch work",
        )
    changed = sum(a != b for a, b in zip(before, after, strict=True))
    if not folder.name.startswith(("c-local-", "c-time-")):
        require(changed == 0, "unexpected persistent edit")
    return dict(
        id=folder.name,
        ticks=len(metrics),
        frames=len(frames),
        initial_energy=e0,
        max_drift_fraction=max_drift / max(e0, max_energy if "force" in folder.name else 0, 1e-12),
        changed_modules=changed,
        final_energy=sum(float(metrics[-1][k]) for k in ("kinetic", "onsite", "coupling")),
        switch_work=float(metrics[-1]["switch_work"]),
        events=events,
        spectra=spectra,
    )


def numerical_decision(root, audited):
    comparisons = []
    for name in ("homogeneous", "heterogeneous", "undamped", "force"):
        prefix = f"n-{name}"
        comparisons += convergence(root, prefix)
    primary = [r for r in audited if r["id"].startswith("n-") and r["id"].endswith("-0.05")]
    energy_pass = len(primary) == 4 and all(r["max_drift_fraction"] <= 0.01 for r in primary)
    undamped = [
        next(r for r in audited if r["id"] == f"n-undamped-{dt}")["max_drift_fraction"]
        for dt in (0.1, 0.05, 0.025)
    ]
    energy_pass = energy_pass and undamped[2] < undamped[1] < undamped[0]
    return dict(
        short_window_pass=energy_pass and all(r["passed"] for r in comparisons if r["time"] <= 32),
        comparisons=comparisons,
        energy_pass=energy_pass,
        passed=energy_pass and all(r["passed"] for r in comparisons),
    )


def planned_cases():
    """Exact frozen matrix, including conditions gated before execution."""
    planned = []
    for name in ("homogeneous", "heterogeneous", "undamped", "force"):
        for dt in (0.1, 0.05, 0.025):
            end = 16 if name == "force" else 32 if name == "undamped" else 307.2
            planned.append((f"n-{name}-{dt}", "M5-0", dt, end))
    for seed, config in (
        (7, "homogeneous"),
        (7, "heterogeneous"),
        (19, "heterogeneous"),
        (7, "rearranged"),
        (19, "rearranged"),
    ):
        for arm in ("A", "B", "AB+", "AB-", "zero"):
            planned.append((f"a-{config}-{seed}-{arm}", "M5-A", 0.05, 32))
    for config, arm in (("homogeneous", "AB+"), ("heterogeneous", "AB+"), ("heterogeneous", "AB-")):
        for sponge in ("false", "true"):
            planned.append((f"b-{config}-{arm}-{sponge}", "M5-B", 0.05, 307.2))
    for dt in (0.05, 0.025):
        for mode in (
            "frozen",
            "disabled",
            "local",
            "time",
            "zero",
            "reexpose-local",
            "reexpose-time",
        ):
            end = 16 if mode in ("local", "time", "zero") else 32
            planned.append((f"c-{mode}-{dt}", "M5-C", dt, end))
    return planned


def coverage(root, stage, output):
    present = {r["id"]: r for r in output["runs"]}
    with (root / "execution-ledger.csv").open("w") as stream:
        writer = csv.writer(stream)
        writer.writerow(("id", "stage", "dt", "end", "status", "reason"))
        for ident, group, dt, end in planned_cases():
            result = present.get(ident)
            if result:
                require(result["ticks"] == round(end / dt) + 1, "registered horizon coverage")
                expected = sum(t <= end for t in CHECKPOINTS)
                require(result["frames"] == expected, "checkpoint coverage")
                status, reason = "COMPLETE", "execution is not scientific success"
            else:
                gate = output.get("numerical", {})
                gated = (group == "M5-B" and not gate.get("passed", False)) or (
                    group == "M5-C" and not gate.get("short_window_pass", False)
                )
                if stage == "all":
                    require(gated, "missing required condition " + ident)
                elif group == "M5-0":
                    require(False, "missing numerical condition " + ident)
                status, reason = (
                    ("GATED", "registered numerical accuracy failure")
                    if gated
                    else ("NOT_RUN", "outside selected verification stage")
                )
            writer.writerow((ident, group, dt, end, status, reason))


def audit_transitions(root):
    result = []
    for folder in sorted((root / "transition-audit").glob("c-*")):
        require((folder / "COMPLETE").exists(), "incomplete switch audit")
        pre, post = decode(folder / "pre.q5f"), decode(folder / "post.q5f")
        require(
            np.array_equal(pre["u"], post["u"]) and np.array_equal(pre["v"], post["v"]),
            "switch altered transients",
        )
        require(
            pre["tick"] == post["tick"] and abs(pre["tick"] * pre["dt"] - 8) < 1e-9, "switch time"
        )
        before = sum(energy(pre["palette"], pre["u"], pre["v"])[:3])
        after = sum(energy(post["palette"], post["u"], post["v"])[:3])
        transition = rows(root / folder.name / "switch.csv")[0]
        require(
            abs(before - float(transition["pre_energy"])) < 1e-10, "independent switch pre-energy"
        )
        require(
            abs(after - float(transition["post_energy"])) < 1e-10, "independent switch post-energy"
        )
        require(abs(after - before - float(transition["work"])) < 1e-10, "independent switch work")
        result.append(
            dict(id=folder.name, pre_energy=before, post_energy=after, switch_work=after - before)
        )
    return result


def verify(root, stage):
    rules = (root / "RULES.md").read_bytes()
    require(rules == Path("EXPERIMENT_0005.md").read_bytes(), "frozen registration differs")
    if (root / "registration.sha256").exists():
        require(
            hashlib.sha256(rules).hexdigest()
            == (root / "registration.sha256").read_text().split()[0],
            "registration hash",
        )
    folders = sorted(p for p in root.iterdir() if p.is_dir() and (p / "COMPLETE").exists())
    audited = []
    cache = root / "audit-cache"
    cache.mkdir(exist_ok=True)
    for folder in folders:
        saved = cache / f"{folder.name}.json"
        digest = hashlib.sha256(Path(__file__).read_bytes())
        for file in sorted(folder.iterdir()):
            if file.is_file():
                digest.update(file.name.encode())
                digest.update(file.read_bytes())
        identity = digest.hexdigest()
        if saved.exists() and json.loads(saved.read_text()).get("input_sha256") == identity:
            audited.append(json.loads(saved.read_text()))
            continue
        print(f"auditing {folder.name}", flush=True)
        result = audit_run(folder)
        result["input_sha256"] = identity
        saved.write_text(json.dumps(result, indent=2) + "\n")
        audited.append(result)
    output = dict(schema="QR5-independent-verification-v1", runs=audited)
    if len([r for r in audited if r["id"].startswith("n-")]) == 12:
        output["numerical"] = numerical_decision(root, audited)
    if stage != "numerical":
        output.update(compare_stages(root, audited))
    output["transition_energy_audits"] = audit_transitions(root)
    coverage(root, stage, output)
    if "numerical" in output:
        gate = output["numerical"]
        (root / "NUMERICAL_GATES.csv").write_text(
            "short_window,long_window\n"
            + f"{str(gate['short_window_pass']).lower()},{str(gate['passed']).lower()}\n"
        )
    (root / "verification.json").write_text(json.dumps(output, indent=2) + "\n")
    print(json.dumps(output.get("numerical", {}), indent=2), flush=True)
    return output


def compare_stages(root, audited):
    results = {}
    superposition = []
    for seed, config in (
        (7, "homogeneous"),
        (7, "heterogeneous"),
        (19, "heterogeneous"),
        (7, "rearranged"),
        (19, "rearranged"),
    ):
        prefix = f"a-{config}-{seed}"
        if not (root / f"{prefix}-A" / "COMPLETE").exists():
            continue
        for time in (0, 0.8, 3.2, 16, 32):
            a, b, plus, minus = [
                checkpoint(root / f"{prefix}-{arm}", time) for arm in ("A", "B", "AB+", "AB-")
            ]
            for target, sign in ((plus, 1), (minus, -1)):
                residual = max(
                    float(np.max(np.abs(target[k] - a[k] - sign * b[k]))) for k in ("u", "v")
                )
                bound = 1e-10 * max(
                    1, *(float(np.max(np.abs(a[k]) + np.abs(b[k]))) for k in ("u", "v"))
                )
                require(residual <= bound, "superposition failure")
                superposition.append(
                    dict(config=config, seed=seed, time=time, sign=sign, residual=residual)
                )
    results["superposition"] = superposition
    for seed in (7, 19):
        path = root / f"a-heterogeneous-{seed}-A"
        other = root / f"a-rearranged-{seed}-A"
        if path.exists() and other.exists():
            p = np.frombuffer((path / "before.palette").read_bytes(), dtype="u1").reshape(4096, 64)
            q = np.frombuffer((other / "before.palette").read_bytes(), dtype="u1").reshape(4096, 64)
            require(
                np.array_equal(np.sort(p, axis=1), np.sort(q, axis=1))
                and np.array_equal(p[:, 27], q[:, 27]),
                "histogram/source control",
            )
    boundary = []
    for config, arm in (("homogeneous", "AB+"), ("heterogeneous", "AB+"), ("heterogeneous", "AB-")):
        damped = root / f"b-{config}-{arm}-true"
        opened = root / (f"n-{config}-0.05" if arm == "AB+" else f"b-{config}-{arm}-false")
        if not (damped / "COMPLETE").exists():
            continue
        d, o = rows(damped / "metrics.csv"), rows(opened / "metrics.csv")
        initial = sum(float(o[0][k]) for k in ("kinetic", "onsite", "coupling"))
        late_open = max(float(r["peak"]) for r in o if float(r["time"]) >= 32)
        late_damped = max(float(r["peak"]) for r in d if float(r["time"]) >= 32)
        boundary.append(
            dict(
                config=config,
                arm=arm,
                absorption_fraction=float(d[-1]["sponge_loss"]) / initial,
                late_peak_ratio=late_damped / late_open,
            )
        )
    results["boundary"] = boundary
    plastic = []
    for dt in (0.05, 0.025):
        virgin = root / f"c-frozen-{dt}"
        active = root / f"c-reexpose-local-{dt}"
        if not (active / "COMPLETE").exists():
            continue
        require(
            (virgin / "after.palette").read_bytes()
            == (root / f"c-disabled-{dt}" / "after.palette").read_bytes(),
            "disabled palette",
        )
        for time in (0, 0.8, 3.2, 16, 32):
            a = checkpoint(virgin, time)
            b = checkpoint(root / f"c-disabled-{dt}", time)
            require(
                np.array_equal(a["u"], b["u"]) and np.array_equal(a["v"], b["v"]),
                "disabled response",
            )
        initial = checkpoint(active, 0)
        expected = np.zeros(initial["u"].shape)
        expected[2182, 3, 3] = 1
        require(
            np.array_equal(initial["u"], expected) and np.count_nonzero(initial["v"]) == 0,
            "transient reset",
        )
        require(
            initial["palette"].tobytes() == (root / f"c-local-{dt}" / "after.palette").read_bytes(),
            "retained actual morphology",
        )
        effects = [field_error(checkpoint(active, t), checkpoint(virgin, t)) for t in (16, 32)]
        changes = next(r["changed_modules"] for r in audited if r["id"] == f"c-local-{dt}")
        folder = root / f"c-local-{dt}"
        before = np.frombuffer((folder / "before.palette").read_bytes(), dtype="u1").reshape(
            4096, 8, 8
        )
        after = np.frombuffer((folder / "after.palette").read_bytes(), dtype="u1").reshape(
            4096, 8, 8
        )
        derived = root / "derived" / folder.name
        derived.mkdir(parents=True, exist_ok=True)
        with (derived / "palettes.jsonl").open("w") as stream:
            for ident in range(4096):
                changed = np.argwhere(before[ident] != after[ident]).tolist()
                stream.write(
                    json.dumps(
                        dict(
                            id=ident,
                            before=before[ident].tolist(),
                            after=after[ident].tolist(),
                            changed_modules=changed,
                        )
                    )
                    + "\n"
                )
        control = [
            field_error(checkpoint(root / f"c-reexpose-time-{dt}", t), checkpoint(virgin, t))
            for t in (16, 32)
        ]
        plastic.append(
            dict(
                dt=dt,
                changes=changes,
                effects=effects,
                generic_time_effects=control,
                before_histogram=np.bincount(before.ravel(), minlength=5).tolist(),
                after_histogram=np.bincount(after.ravel(), minlength=5).tolist(),
            )
        )
    results["plastic"] = plastic
    if len(plastic) == 2:
        changes_ok = abs(plastic[0]["changes"] - plastic[1]["changes"]) <= 0.05 * max(
            1, plastic[1]["changes"]
        )
        effects_ok = all(
            a >= 0.01 and b >= 0.01 and abs(a - b) <= 0.2 * max(b, 1e-30)
            for a, b in zip(plastic[0]["effects"], plastic[1]["effects"], strict=True)
        )
        results["persistent_response_pass"] = changes_ok and effects_ok
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--stage", default="all", choices=("all", "numerical"))
    args = parser.parse_args()
    verify(args.root, args.stage)


if __name__ == "__main__":
    main()
