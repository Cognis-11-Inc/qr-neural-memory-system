"""Independent QR6 field, local-rule, energy and causal-contrast verification.

Histories are re-integrated from recorded early probes for six neurons, not the
whole engine. All module edit decisions are rederived from exported local fields.
"""

import argparse
import csv
import gzip
import hashlib
import json
import math
import struct
import time
from pathlib import Path

import numpy as np

from analysis.experiment_0005 import coefficients, energy, fnv, require, rows

FIELDS = 4096 * 64
TIMES = (0, 0.8, 3.2, 8, 16, 32, 51.2, 102.4, 153.6, 204.8, 256, 307.2)
EARLY = [0, 2182, 2183, 2184, 2186, 2191]
VALIDATED = set()
SOURCE_BYTES = Path(__file__).read_bytes() + Path("analysis/experiment_0005.py").read_bytes()


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def decode(path):
    data = path.read_bytes()
    require(len(data) >= 64 and data[:8] == b"QR6FLD01", "QR6 magic/length")
    version, tick, dt, seed, config, length, module = struct.unpack_from("<QQdQQQQ", data, 8)
    require(version == 1 and 0 < length <= 200 and module in (9, 27), "QR6 metadata")
    require(dt in (0.05, 0.025, 0.0125) and tick <= 24576 and config <= 2, "QR6 grid")
    require(len(data) == 64 + length + 17 * FIELDS + 8, "QR6 exact length")
    digest = hashlib.sha256(data).hexdigest()
    if digest not in VALIDATED:
        require(fnv(data[:-8]) == struct.unpack_from("<Q", data, len(data) - 8)[0], "QR6 checksum")
        VALIDATED.add(digest)
    offset = 64 + length
    palette = np.frombuffer(data, dtype="u1", count=FIELDS, offset=offset).reshape(4096, 8, 8)
    u = np.frombuffer(data, dtype="<f8", count=FIELDS, offset=offset + FIELDS).reshape(4096, 8, 8)
    v = np.frombuffer(data, dtype="<f8", count=FIELDS, offset=offset + 9 * FIELDS).reshape(
        4096, 8, 8
    )
    require(
        np.max(palette) <= 4 and np.isfinite(u).all() and np.isfinite(v).all(), "QR6 field screen"
    )
    require(max(float(np.max(np.abs(u))), float(np.max(np.abs(v)))) <= 100, "QR6 boundedness")
    return dict(
        tick=tick,
        dt=dt,
        seed=seed,
        config=config,
        module=module,
        label=data[64:offset].decode(),
        palette=palette,
        u=u,
        v=v,
        sha256=digest,
    )


def resolve(root, ident):
    folder = root / ident
    if (folder / "REUSED").exists():
        reference = (folder / "REUSED").read_text()
        require("/" not in reference and ".." not in reference, "unsafe reuse reference")
        folder = root / reference
    require((folder / "COMPLETE").exists(), "missing complete trajectory " + ident)
    return folder


def checkpoint(root, ident, physical_time):
    folder = resolve(root, ident)
    dt = float(rows(folder / "parameters.csv")[0]["dt"])
    return decode(folder / f"{round(physical_time / dt):06d}.q6f")


def internal(module):
    return [q for q in range(64) if abs(module // 8 - q // 8) + abs(module % 8 - q % 8) == 1]


def scores_from_history(history):
    result = history.copy()
    for module in range(64):
        result[:, module] += 0.5 * np.mean(history[:, internal(module)], axis=1)
    return result


def morph(before, velocity, scores, rule):
    after = before.copy()
    for ident in range(4096):
        local = scores[ident]
        maximum = float(np.max(local))
        if rule == 1:
            module = int(np.argmax(velocity[ident]))
            if velocity[ident, module] >= 1e-8:
                after[ident, module] = (int(before[ident, module]) + 1) % 5
        elif rule == 2:
            candidates = [m for m in range(64) if local[m] >= 1e-8 and local[m] >= 0.05 * maximum]
            for module in sorted(candidates, key=lambda m: (-local[m], m))[:8]:
                after[ident, module] = (int(before[ident, module]) + 1) % 5
        elif rule == 3:
            edges = [
                (m, q)
                for m in range(64)
                for q in internal(m)
                if m < q
                and before[ident, m] != before[ident, q]
                and local[m] >= 1e-8
                and local[q] >= 1e-8
                and max(local[m], local[q]) >= 0.05 * maximum
            ]
            used = set()
            for m, q in sorted(edges, key=lambda pair: (-sum(local[k] for k in pair), pair)):
                if len(used) < 8 and m not in used and q not in used:
                    after[ident, m], after[ident, q] = before[ident, q], before[ident, m]
                    used.update((m, q))
    return after


def splitmix(value):
    mask = (1 << 64) - 1
    value = (value + 0x9E3779B97F4A7C15) & mask
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & mask
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & mask
    return value ^ (value >> 31)


def variant(before, after, kind, rule, seed):
    result = before.copy()
    if kind == "ALL":
        return after.copy()
    if kind == "SOURCE_ONLY":
        result[2182] = after[2182]
    elif kind == "OFF_SOURCE_ONLY":
        result = after.copy()
        result[2182] = before[2182]
    elif kind == "SHUFFLE_POSITION":
        for ident in range(4096):
            count = int(np.count_nonzero(before[ident] != after[ident]))
            order = sorted(range(64), key=lambda m: (splitmix(seed ^ (ident * 64 + m) ^ 0x6006), m))
            if rule == 2:
                for module in order[:count]:
                    result[ident, module] = (int(before[ident, module]) + 1) % 5
            else:
                groups = [[m for m in order if before[ident, m] == color] for color in range(5)]
                for _ in range(count // 2):
                    colors = sorted(range(5), key=lambda c: (-len(groups[c]), c))
                    require(bool(groups[colors[1]]), "infeasible shuffle control")
                    a, b = groups[colors[0]].pop(0), groups[colors[1]].pop(0)
                    result[ident, a], result[ident, b] = before[ident, b], before[ident, a]
    return result


def compare(a, b, mask=None):
    numerator = denominator = 0.0
    count = 0
    for key in ("u", "v"):
        x, y = a[key], b[key]
        if mask is not None:
            x, y = x[mask], y[mask]
        numerator += float(np.sum((x - y) ** 2))
        denominator += float(np.sum(y * y))
        count += x.size
    return dict(
        relative_l2=math.sqrt(numerator / max(denominator, 1e-30)),
        absolute_rms=math.sqrt(numerator / count),
    )


def external_flux(palette, u, v):
    conductance = coefficients(palette)[2].reshape(16, 16, 16, 8, 8)
    u, v = u.reshape(16, 16, 16, 8, 8), v.reshape(16, 16, 16, 8, 8)
    flux = np.zeros(u.shape)
    for axis in range(3):
        left, right = [slice(None)] * 5, [slice(None)] * 5
        left[axis], right[axis] = slice(None, -1), slice(1, None)
        left, right = tuple(left), tuple(right)
        power = (
            0.5
            * (0.4 / 6)
            * conductance[left]
            * conductance[right]
            * (u[right] - u[left])
            * (v[left] + v[right])
        )
        flux[left] += power
        flux[right] -= power
    return np.sum(flux, axis=(3, 4)).reshape(4096)


def histories(folder, before):
    velocity, history, scores = [
        np.frombuffer((folder / f"{name}.bin").read_bytes(), dtype="<f8").reshape(4096, 64)
        for name in ("velocity-history", "shape-history", "scores")
    ]
    require(
        np.isfinite(history).all() and np.all(history >= 0) and np.all(velocity >= 0),
        "history screen",
    )
    expected = scores_from_history(history)
    require(np.allclose(expected, scores, rtol=1e-14, atol=1e-300), "within-neuron scores")
    data = (folder / "early.q6p").read_bytes()
    require(data[:8] == b"QR6PRB01", "probe schema")
    dt = float(rows(folder / "parameters.csv")[0]["dt"])
    stride = 8 + 6 * 64 * 2 * 8
    require((len(data) - 8) % stride == 0, "probe stream length")
    dtype = np.dtype([("tick", "<u8"), ("state", "<f8", (6, 64, 2))])
    samples = np.frombuffer(data, dtype=dtype, offset=8)
    require(np.array_equal(samples["tick"], np.arange(len(samples))), "probe exact timeline")
    require(
        len(samples) == round(min(16, float(rows(folder / "parameters.csv")[0]["end"])) / dt) + 1,
        "complete early probe horizon",
    )
    end = round(8 / dt)
    states = samples["state"][: end + 1]
    u, v = states[:, :, :, 0], states[:, :, :, 1]
    strain = np.empty(u.shape)
    for module in range(64):
        strain[:, :, module] = np.mean(
            (u[:, :, module, None] - u[:, :, internal(module)]) ** 2, axis=2
        )
    weights = np.ones(len(states))
    weights[[0, -1]] = 0.5
    computed_v = np.sum(v * v * weights[:, None, None], axis=0) * dt
    computed_h = np.sum((v * v + 0.25 * strain) * weights[:, None, None], axis=0) * dt
    require(
        np.allclose(computed_v, velocity[EARLY], rtol=2e-12, atol=1e-300),
        "probe velocity-history integration",
    )
    require(
        np.allclose(computed_h, history[EARLY], rtol=2e-12, atol=1e-300),
        "probe shape-history integration",
    )
    params = rows(folder / "parameters.csv")[0]
    rule = 0 if params["rule"] == "OFF" else int(params["rule"])
    expected_palette = morph(before, velocity, scores, rule)
    after = np.frombuffer((folder / "after.palette").read_bytes(), dtype="u1").reshape(4096, 64)
    require(np.array_equal(after, expected_palette), "independent registered edit positions")
    if rule == 3:
        require(np.array_equal(np.sort(before, axis=1), np.sort(after, axis=1)), "R3 histogram")
    count = np.count_nonzero(before != after, axis=1)
    require(np.max(count) <= (1 if rule == 1 else 8), "per-neuron edit bound")
    pre, post = decode(folder / "switch/pre.q6f"), decode(folder / "switch/post.q6f")
    require(
        np.array_equal(pre["u"], post["u"]) and np.array_equal(pre["v"], post["v"]),
        "transient switch jump",
    )
    pre_energy = energy(pre["palette"], pre["u"], pre["v"])
    post_energy = energy(post["palette"], post["u"], post["v"])
    audit = rows(folder / "switch/energy.csv")
    for ident, row in enumerate(audit):
        require(
            abs(pre_energy[3][ident] - float(row["pre_energy"])) < 1e-11, "local pre-switch energy"
        )
        require(
            abs(post_energy[3][ident] - float(row["post_energy"])) < 1e-11,
            "local post-switch energy",
        )
    work = sum(post_energy[:3]) - sum(pre_energy[:3])
    require(
        abs(work - sum(float(r["switch_work"]) for r in audit)) < 1e-10,
        "independent total switch work",
    )
    eligible = (scores >= 1e-8) & (scores >= 0.05 * np.max(scores, axis=1)[:, None])
    argmax = np.argmax(velocity, axis=1)
    raw_selected = np.max(velocity, axis=1) >= 1e-8
    derived = folder.parent / "derived" / folder.name
    derived.mkdir(parents=True, exist_ok=True)
    order = np.argsort(-scores, axis=1, kind="stable")
    ranks = np.empty(order.shape, dtype=np.int16)
    np.put_along_axis(ranks, order, np.broadcast_to(np.arange(64), order.shape), axis=1)
    with gzip.open(derived / "eligibility.csv.gz", "wt", compresslevel=5) as stream:
        writer = csv.writer(stream)
        writer.writerow(
            (
                "id",
                "module",
                "velocity_history",
                "shape_history",
                "score",
                "rank",
                "above_absolute",
                "above_relative",
            )
        )
        for ident in range(4096):
            maximum = float(np.max(scores[ident]))
            for module in range(64):
                writer.writerow(
                    (
                        ident,
                        module,
                        format(velocity[ident, module], ".17e"),
                        format(history[ident, module], ".17e"),
                        format(scores[ident, module], ".17e"),
                        int(ranks[ident, module]),
                        int(scores[ident, module] >= 1e-8),
                        int(scores[ident, module] >= 0.05 * maximum),
                    )
                )
    with (derived / "palettes.jsonl").open("w") as stream:
        for ident in range(4096):
            stream.write(
                json.dumps(
                    dict(
                        id=ident,
                        before=before[ident].reshape(8, 8).tolist(),
                        after=after[ident].reshape(8, 8).tolist(),
                        changed_modules=np.flatnonzero(before[ident] != after[ident]).tolist(),
                    )
                )
                + "\n"
            )
    return dict(
        changed_modules=int(np.sum(count)),
        changed_neurons=int(np.count_nonzero(count)),
        count_histogram=np.bincount(count, minlength=9).tolist(),
        switch_work=work,
        raw_argmax_histogram=np.bincount(argmax[raw_selected], minlength=64).tolist(),
        eligible_module_histogram=np.bincount(
            np.count_nonzero(eligible, axis=1), minlength=65
        ).tolist(),
        changed_module_indices=np.nonzero(np.any(before != after, axis=0))[0].tolist(),
    )


def audit_run(root, folder):
    params = rows(folder / "parameters.csv")[0]
    metrics = rows(folder / "metrics.csv")
    dt, end = float(params["dt"]), float(params["end"])
    require(len(metrics) == round(end / dt) + 1, "registered every-tick horizon")
    before = np.frombuffer((folder / "before.palette").read_bytes(), dtype="u1").reshape(4096, 64)
    after = np.frombuffer((folder / "after.palette").read_bytes(), dtype="u1").reshape(4096, 64)
    require(np.max(before) <= 4 and np.max(after) <= 4, "palette values")
    initial = checkpoint(root, folder.name, 0)
    initial_energy = sum(energy(initial["palette"], initial["u"], initial["v"])[:3])
    require(abs(initial_energy - float(params["initial_energy"])) <= 1e-11, "initial energy")
    if params["budget"] == "E1":
        require(abs(initial_energy - 1) <= 1e-12, "E1 match")
    if params["arm"] != "zero":
        expected = np.zeros(initial["u"].shape)
        module = int(params["module"])
        expected[2182, module // 8, module % 8] = float(params["amplitude"])
        if params["arm"] == "AB+":
            expected[2186, module // 8, module % 8] = float(params["amplitude"])
        require(
            np.array_equal(initial["u"], expected) and np.count_nonzero(initial["v"]) == 0,
            "exact reset/impulse",
        )
    worst = 0.0
    for tick, row in enumerate(metrics):
        numbers = {k: float(v) for k, v in row.items()}
        require(
            int(row["tick"]) == tick and abs(numbers["time"] - tick * dt) < 1e-9, "metric timeline"
        )
        require(
            all(math.isfinite(x) for x in numbers.values()) and numbers["nonfinite"] == 0,
            "nonfinite metrics",
        )
        total = sum(numbers[k] for k in ("kinetic", "onsite", "coupling"))
        residual = total - initial_energy + numbers["loss"] - numbers["switch_work"]
        require(abs(residual - numbers["drift"]) < 1e-10, "energy arithmetic")
        require(
            0 <= numbers["peak"] <= 100 and 0 <= numbers["velocity_peak"] <= 100, "amplitude screen"
        )
        if params["arm"] == "zero":
            require(
                total == 0 and numbers["peak"] == 0 and numbers["velocity_peak"] == 0,
                "zero-drive control",
            )
        worst = max(worst, abs(residual) / max(initial_energy, 1e-12))
    frames = sorted(folder.glob("*.q6f"))
    require(len(frames) == sum(t <= end for t in TIMES), "checkpoint coverage")
    frame_hashes = []
    wanted = {round(t / dt) for t in TIMES if t <= end}
    probe_checkpoints = {}
    probe_count = 0
    with (folder / "probes.csv").open() as stream:
        for row in csv.DictReader(stream):
            tick = int(row["tick"])
            ident = int(row["id"])
            require(
                tick == probe_count // 11
                and ident in (0, 2180, 2182, 2183, 2184, 2185, 2186, 2188, 2191, 2440, 4095),
                "probe every-tick coverage",
            )
            require(abs(float(row["time"]) - tick * dt) < 1e-9, "probe time")
            if tick in wanted:
                probe_checkpoints[(tick, ident)] = row
            probe_count += 1
    require(
        probe_count == len(metrics) * 11 and len(probe_checkpoints) == len(wanted) * 11,
        "probe complete indices",
    )
    for file in frames:
        state = decode(file)
        frame_hashes.append(state["sha256"])
        require(
            state["label"] == folder.name and state["module"] == int(params["module"]),
            "condition identity",
        )
        measured = energy(state["palette"], state["u"], state["v"])
        row = metrics[state["tick"]]
        for key, value in zip(("kinetic", "onsite", "coupling"), measured[:3], strict=True):
            require(
                abs(value - float(row[key])) < 1e-10 * max(1, value),
                "independent field energy " + key,
            )
        flux = external_flux(state["palette"], state["u"], state["v"])
        require(abs(float(np.sum(flux))) < 1e-10, "external power cancellation")
        flat_u, flat_v = state["u"].reshape(4096, 64), state["v"].reshape(4096, 64)
        for ident in (0, 2180, 2182, 2183, 2184, 2185, 2186, 2188, 2191, 2440, 4095):
            probe = probe_checkpoints[(state["tick"], ident)]
            for module, label in ((state["module"], "source"), (9, "9"), (27, "27")):
                for field, array in (("u", flat_u), ("v", flat_v)):
                    key = field + ("_source" if label == "source" else label)
                    require(float(probe[key]) == array[ident, module], "signed probe checkpoint")
            require(abs(float(probe["energy"]) - measured[3][ident]) < 1e-10, "local probe energy")
            require(
                abs(float(probe["net_external_power"]) - flux[ident]) < 1e-10,
                "local probe external power",
            )
        require(abs(float(row["peak"]) - np.max(np.abs(state["u"]))) < 1e-14, "full field maximum")
    result = dict(
        id=folder.name,
        dt=dt,
        initial_energy=initial_energy,
        amplitude=float(params["amplitude"]),
        ticks=len(metrics),
        frames=len(frames),
        max_energy_drift=worst,
        changed_modules=int(np.count_nonzero(before != after)),
        frame_sha256=frame_hashes,
    )
    if (folder / "scores.bin").exists():
        result.update(histories(folder, before))
    else:
        require(np.array_equal(before, after), "frozen trajectory changed morphology")
    return result


def audit_available(root):
    cache = root / "audit-cache"
    cache.mkdir(exist_ok=True)
    source = SOURCE_BYTES
    audited = []
    for folder in sorted(root.iterdir()):
        if not folder.is_dir() or not (folder / "COMPLETE").exists():
            continue
        digest = hashlib.sha256(source)
        for file in sorted(folder.rglob("*")):
            if file.is_file():
                digest.update(file.relative_to(folder).as_posix().encode())
                digest.update(bytes.fromhex(sha(file)))
        identity = digest.hexdigest()
        receipt = cache / f"{folder.name}.json"
        if receipt.exists() and json.loads(receipt.read_text()).get("input_sha256") == identity:
            result = json.loads(receipt.read_text())
            VALIDATED.update(result["frame_sha256"])
        else:
            print("auditing", folder.name, flush=True)
            result = audit_run(root, folder)
            result["input_sha256"] = identity
            receipt.write_text(json.dumps(result, indent=2) + "\n")
        audited.append(result)
    return audited


def waveform_diagnostics(root):
    result = []
    for config, seed in (("homogeneous", 7), ("heterogeneous", 7), ("heterogeneous", 19)):
        series = []
        for dt in (0.05, 0.025, 0.0125):
            path = root / f"n-{config}-s{seed}-d{dt}" / "probes.csv"
            selected = {i: [] for i in (2182, 2184, 2191)}
            with path.open() as stream:
                for row in csv.DictReader(stream):
                    ident = int(row["id"])
                    if ident in selected:
                        selected[ident].append(
                            (float(row["time"]), float(row["u_source"]), float(row["v_source"]))
                        )
            series.append({i: np.asarray(value) for i, value in selected.items()})
        for pair in range(2):
            for ident in (2182, 2184, 2191):
                a, b = series[pair][ident], series[pair + 1][ident]
                require(
                    a.shape == b[::2].shape and np.max(np.abs(a[:, 0] - b[::2, 0])) < 1e-9,
                    "matched probe physical times",
                )

                def peaks(trace):
                    envelope = np.abs(trace[:, 1])
                    indices = []
                    last = -1e6
                    for i in range(1, len(trace) - 1):
                        if (
                            envelope[i] >= 1e-6
                            and envelope[i] > envelope[i - 1]
                            and envelope[i] > envelope[i + 1]
                            and trace[i, 0] - last >= 1 - 1e-12
                        ):
                            indices.append(i)
                            last = trace[i, 0]
                    return trace[indices, 0]

                first, second = peaks(a), peaks(b)
                lag = (
                    max((float(np.min(np.abs(second - t))) for t in first), default=0.0)
                    if len(second)
                    else None
                )
                result.append(
                    dict(
                        config=config,
                        seed=seed,
                        pair=pair,
                        id=ident,
                        max_u_disagreement=float(np.max(np.abs(a[:, 1] - b[::2, 1]))),
                        max_v_disagreement=float(np.max(np.abs(a[:, 2] - b[::2, 2]))),
                        coarse_peak_count=len(first),
                        fine_peak_count=len(second),
                        max_nearest_peak_lag=lag,
                    )
                )
    return result


def numerical(root, audited):
    output = []
    complete = all(
        (root / f"n-{config}-s{seed}-d{dt}/COMPLETE").exists()
        for config, seed in (("homogeneous", 7), ("heterogeneous", 7), ("heterogeneous", 19))
        for dt in (0.05, 0.025, 0.0125)
    )
    if not complete:
        return dict(complete=False, short_pass=False, long_pass=False, comparisons=[])
    for config, seed in (("homogeneous", 7), ("heterogeneous", 7), ("heterogeneous", 19)):
        for physical_time in TIMES[1:]:
            frames = [
                checkpoint(root, f"n-{config}-s{seed}-d{dt}", physical_time)
                for dt in (0.05, 0.025, 0.0125)
            ]
            a, b = compare(frames[0], frames[1]), compare(frames[1], frames[2])
            ratio = b["relative_l2"] / a["relative_l2"] if a["relative_l2"] > 1e-10 else 0
            limit_a, limit_b = (0.05, 0.015) if physical_time <= 102.4 else (0.1, 0.03)
            output.append(
                dict(
                    config=config,
                    seed=seed,
                    time=physical_time,
                    coarse=a,
                    fine=b,
                    ratio=ratio,
                    passed=a["relative_l2"] <= limit_a
                    and b["relative_l2"] <= limit_b
                    and ratio <= 0.35,
                )
            )
    cases = [r for r in audited if r["id"].startswith("n-")]
    energy_ok = len(cases) == 12 and all(r["max_energy_drift"] <= 0.01 for r in cases)
    return dict(
        complete=energy_ok,
        energy_pass=energy_ok,
        short_pass=energy_ok and all(r["passed"] for r in output if r["time"] <= 32),
        long_pass=energy_ok and all(r["passed"] for r in output),
        comparisons=output,
    )


def m_id(seed, module, budget, rule, dt):
    return f"m-s{seed}-m{module}-{budget}-R{rule}-d{dt}"


def planned():
    result = []
    for config, seed in (("homogeneous", 7), ("heterogeneous", 7), ("heterogeneous", 19)):
        for dt in (0.05, 0.025, 0.0125):
            result.append((f"n-{config}-s{seed}-d{dt}", "N"))
    result += [(f"n-zero-d{dt}", "N") for dt in (0.05, 0.025, 0.0125)]
    for seed in (7, 19):
        for module in (27, 9):
            for budget in ("UNIT", "E1"):
                result.append((f"d-s{seed}-m{module}-{budget}", "D"))
                result += [(m_id(seed, module, budget, rule, 0.025), "M") for rule in range(4)]
            result.append((m_id(seed, module, "E1", 4, 0.025), "M"))
        result.append((f"d-s{seed}-zero", "D"))
        result += [(f"m-s{seed}-zero-R{rule}", "M") for rule in range(4)]
    result += [
        (m_id(seed, module, "E1", rule, 0.0125), "M")
        for seed, module in ((7, 27), (19, 9))
        for rule in (2, 3)
    ]
    for dt in (0.025, 0.0125):
        for seed in (7, 19):
            for module in (27, 9):
                if dt == 0.0125 and (seed, module) not in ((7, 27), (19, 9)):
                    continue
                for rule in (2, 3):
                    for kind in (
                        "VIRGIN",
                        "SOURCE_ONLY",
                        "OFF_SOURCE_ONLY",
                        "ALL",
                        "SHUFFLE_POSITION",
                    ):
                        for budget in ("UNIT", "MATCH"):
                            result.append(
                                (f"c-s{seed}-m{module}-R{rule}-d{dt}-{kind}-{budget}", "C")
                            )
    require(len(result) == 190 and len(set(x[0] for x in result)) == 190, "registered ledger")
    return result


def causal_and_structural(root, audited):
    structural = []
    causal = []
    source_mask = np.arange(4096) == 2182
    for seed, module in ((7, 27), (19, 9)):
        for rule in (2, 3):
            coarse, fine = [m_id(seed, module, "E1", rule, dt) for dt in (0.025, 0.0125)]
            if not (root / fine / "COMPLETE").exists():
                continue
            before = np.frombuffer((root / coarse / "before.palette").read_bytes(), dtype="u1")
            a = np.frombuffer((root / coarse / "after.palette").read_bytes(), dtype="u1") != before
            b = np.frombuffer((root / fine / "after.palette").read_bytes(), dtype="u1") != before
            union = int(np.count_nonzero(a | b))
            agreement = int(np.count_nonzero(a & b)) / max(1, union)
            errors = [
                compare(checkpoint(root, coarse, t), checkpoint(root, fine, t))["relative_l2"]
                for t in (16, 32)
            ]
            counts = [
                next(r for r in audited if r["id"] == ident)["count_histogram"]
                for ident in (coarse, fine)
            ]
            passes = (
                all(sum(count[2:]) > 0 for count in counts)
                and agreement >= 0.9
                and max(errors) <= 0.015
            )
            structural.append(
                dict(
                    seed=seed,
                    module=module,
                    rule=rule,
                    jaccard=agreement,
                    exposure_errors=errors,
                    passed=passes,
                )
            )
    for dt in (0.025, 0.0125):
        for seed in (7, 19):
            for module in (27, 9):
                if dt == 0.0125 and (seed, module) not in ((7, 27), (19, 9)):
                    continue
                for rule in (2, 3):
                    exposure = root / m_id(seed, module, "E1", rule, dt)
                    before = np.frombuffer(
                        (exposure / "before.palette").read_bytes(), dtype="u1"
                    ).reshape(4096, 64)
                    after = np.frombuffer(
                        (exposure / "after.palette").read_bytes(), dtype="u1"
                    ).reshape(4096, 64)
                    virgin_prefix = f"c-s{seed}-m{module}-R{rule}-d{dt}"
                    for kind in (
                        "VIRGIN",
                        "SOURCE_ONLY",
                        "OFF_SOURCE_ONLY",
                        "ALL",
                        "SHUFFLE_POSITION",
                    ):
                        for budget in ("UNIT", "MATCH"):
                            ident = f"{virgin_prefix}-{kind}-{budget}"
                            if not (root / ident).exists():
                                continue
                            initial = checkpoint(root, ident, 0)
                            expected = variant(before, after, kind, rule, seed)
                            require(
                                np.array_equal(initial["palette"].reshape(4096, 64), expected),
                                "immutable causal palette variant",
                            )
                            virgin = checkpoint(root, f"{virgin_prefix}-VIRGIN-UNIT", 0)
                            e0 = sum(energy(initial["palette"], initial["u"], initial["v"])[:3])
                            v0 = sum(energy(virgin["palette"], virgin["u"], virgin["v"])[:3])
                            if budget == "MATCH":
                                require(
                                    abs(e0 - v0) / v0 <= 1e-12, "causal initial-energy matching"
                                )
                            effects = []
                            for t in (16, 32):
                                state, reference = (
                                    checkpoint(root, ident, t),
                                    checkpoint(root, f"{virgin_prefix}-VIRGIN-UNIT", t),
                                )
                                pair = checkpoint(root, f"{virgin_prefix}-SOURCE_ONLY-{budget}", t)
                                effects.append(
                                    dict(
                                        time=t,
                                        global_effect=compare(state, reference),
                                        source_effect=compare(state, reference, source_mask),
                                        offsource_effect=compare(state, reference, ~source_mask),
                                        versus_source_only=compare(state, pair, ~source_mask),
                                    )
                                )
                            causal.append(
                                dict(
                                    id=ident,
                                    dt=dt,
                                    seed=seed,
                                    module=module,
                                    rule=rule,
                                    kind=kind,
                                    budget=budget,
                                    initial_energy=e0,
                                    changed_modules=int(np.count_nonzero(expected != before)),
                                    effects=effects,
                                )
                            )
    return structural, causal


def causal_gates(root, causal):
    decisions = []
    for seed, module in ((7, 27), (19, 9)):
        for rule in (2, 3):
            for budget in ("UNIT", "MATCH"):
                records = [
                    next(
                        (
                            r
                            for r in causal
                            if r["seed"] == seed
                            and r["module"] == module
                            and r["rule"] == rule
                            and r["budget"] == budget
                            and r["kind"] == "OFF_SOURCE_ONLY"
                            and r["dt"] == dt
                        ),
                        None,
                    )
                    for dt in (0.025, 0.0125)
                ]
                if any(r is None for r in records):
                    continue
                values = [
                    [e["offsource_effect"]["relative_l2"] for e in r["effects"]] for r in records
                ]
                effect_pass = all(v >= 0.01 for series in values for v in series)
                effect_agreement = all(
                    abs(a - b) <= 0.2 * max(b, 1e-30) for a, b in zip(*values, strict=True)
                )
                field_checks = []
                for kind in ("VIRGIN", "SOURCE_ONLY", "OFF_SOURCE_ONLY", "ALL", "SHUFFLE_POSITION"):
                    ids = [
                        f"c-s{seed}-m{module}-R{rule}-d{dt}-{kind}-{budget}"
                        for dt in (0.025, 0.0125)
                    ]
                    errors = [
                        compare(checkpoint(root, ids[0], t), checkpoint(root, ids[1], t))[
                            "relative_l2"
                        ]
                        for t in (16, 32)
                    ]
                    field_checks.append(dict(kind=kind, errors=errors, passed=max(errors) <= 0.015))
                all_record = next(
                    r
                    for r in causal
                    if r["seed"] == seed
                    and r["module"] == module
                    and r["rule"] == rule
                    and r["budget"] == budget
                    and r["kind"] == "ALL"
                    and r["dt"] == 0.025
                )
                near_source = all(
                    e["versus_source_only"]["relative_l2"] <= 0.01 for e in all_record["effects"]
                )
                reliable = effect_agreement and all(r["passed"] for r in field_checks)
                decisions.append(
                    dict(
                        seed=seed,
                        module=module,
                        rule=rule,
                        budget=budget,
                        offsource_effect_pass=effect_pass,
                        effect_grid_agreement=effect_agreement,
                        field_checks=field_checks,
                        source_only_nearly_matches_all=near_source,
                        status=("PASS" if effect_pass else "FAIL") if reliable else "INDETERMINATE",
                    )
                )
    return decisions


def historical_regressions(root):
    from analysis.experiment_0005 import decode as decode_m5

    checks = []
    oldroot = Path("results/experiment_0005")
    if not oldroot.exists():
        return [dict(status="NOT_RUN", reason="historical raw data absent")]
    for config in ("homogeneous", "heterogeneous"):
        for dt in (0.05, 0.025):
            ident = f"n-{config}-s7-d{dt}"
            if not (root / ident / "COMPLETE").exists():
                continue
            old = oldroot / f"n-{config}-{dt}" / f"{round(307.2 / dt):06d}.q5f"
            a, b = checkpoint(root, ident, 307.2), decode_m5(old)
            require(
                all(a[k].tobytes() == b[k].tobytes() for k in ("palette", "u", "v")),
                "M5 full-field physical regression",
            )
            checks.append(dict(id=ident, status="PASS", time=307.2, arrays_byte_identical=True))
    ident = m_id(7, 27, "UNIT", 1, 0.025)
    if (root / ident / "COMPLETE").exists():
        a = checkpoint(root, ident, 16)
        b = decode_m5(oldroot / "c-local-0.025/000640.q5f")
        require(
            all(a[k].tobytes() == b[k].tobytes() for k in ("palette", "u", "v")),
            "M5 R1 exact state regression",
        )
        require(
            (root / ident / "velocity-history.bin").read_bytes()
            == (oldroot / "c-local-0.025/history.bin").read_bytes(),
            "M5 R1 exact local history",
        )
        checks.append(
            dict(id=ident, status="PASS", time=16, arrays_and_history_byte_identical=True)
        )
    return checks


def verify(root, stage):
    require(
        (root / "RULES.md").read_bytes() == Path("EXPERIMENT_0006.md").read_bytes(),
        "frozen registration identity",
    )
    audited = audit_available(root)
    gate = numerical(root, audited)
    output = dict(schema="QR6-independent-v1", runs=audited, numerical=gate)
    if gate["complete"]:
        output["waveform_diagnostics"] = waveform_diagnostics(root)
    (root / "GATES.csv").write_text(
        "short,long\n" + f"{str(gate['short_pass']).lower()},{str(gate['long_pass']).lower()}\n"
    )
    with (root / "execution-ledger.csv").open("w") as stream:
        writer = csv.writer(stream)
        writer.writerow(("id", "stage", "status", "reference_or_reason"))
        for ident, group in planned():
            folder = root / ident
            if (folder / "COMPLETE").exists():
                status, reason = "COMPLETE", "execution does not imply scientific success"
            elif (folder / "REUSED").exists():
                status, reason = "REUSED", (folder / "REUSED").read_text()
                resolve(root, ident)
            else:
                status, reason = (
                    ("GATED", "M6-N short-window failure")
                    if group != "N" and gate["complete"] and not gate["short_pass"]
                    else ("NOT_RUN", "not yet executed")
                )
                if stage == "all":
                    require(status == "GATED", "missing registered condition " + ident)
            writer.writerow((ident, group, status, reason))
    if stage == "all" and gate["short_pass"]:
        output["structural"], output["causal"] = causal_and_structural(root, audited)
    output["historical_regressions"] = historical_regressions(root)
    if "causal" in output:
        output["causal_gates"] = causal_gates(root, output["causal"])
    (root / "verification.json").write_text(json.dumps(output, indent=2) + "\n")
    print(json.dumps({k: v for k, v in gate.items() if k != "comparisons"}), flush=True)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--stage", choices=("n", "all"), default="all")
    parser.add_argument("--watch-audit", action="store_true")
    args = parser.parse_args()
    if args.watch_audit:
        while True:
            audit_available(args.root)
            if (args.root / "FINISHED-c").exists():
                break
            time.sleep(5)
    else:
        verify(args.root, args.stage)


if __name__ == "__main__":
    main()
