"""Check actual QR6 native receipts against independently decoded scientific bytes."""

import argparse
import json
import re
import struct
from pathlib import Path

from analysis.experiment_0006 import decode, fnv, require


def observation_hash(frame):
    data = bytearray(b"QR-DISPLAY-RECEIPT-v1")
    data.extend(struct.pack("<8Q", frame["tick"], frame["seed"], 0, frame["config"], 2182, 0, 3, 0))
    u, v, palette = (
        frame["u"].reshape(4096, 64),
        frame["v"].reshape(4096, 64),
        frame["palette"].reshape(4096, 64),
    )
    for ident in range(4096):
        data.extend(struct.pack("<4Q", ident, ident % 16, ident // 16 % 16, ident // 256))
        data.extend(palette[ident].tobytes())
        squared_u = squared_v = 0.0
        for module in range(64):
            x, y = float(u[ident, module]), float(v[ident, module])
            squared_u += x * x
            squared_v += y * y
        values = []
        for value in (
            max(abs(float(x)) for x in u[ident]),
            max(abs(float(x)) for x in v[ident]),
            float(u[ident, 27]),
            float(v[ident, 27]),
        ):
            values.extend((value, 0, 0, 0))
        values.extend((squared_u, squared_v))
        data.extend(struct.pack("<18dI", *values, 0))
    return fnv(data)


def verify(case, receipt):
    text = receipt.read_text()
    tick, selected = [int(re.search(rf"\b{name}=(\d+)", text)[1]) for name in ("tick", "selected")]
    frame = decode(case / f"{tick:06d}.q6f")
    before = (case / "before.palette").read_bytes()[selected * 64 : (selected + 1) * 64]
    after = (case / "after.palette").read_bytes()[selected * 64 : (selected + 1) * 64]
    for name, expected in (
        ("snapshot_hash", observation_hash(frame)),
        ("morphology_before", fnv(before)),
        ("morphology_after", fnv(after)),
    ):
        actual = int(re.search(rf"\b{name}=([0-9a-f]+)", text)[1], 16)
        require(actual == expected, "native scientific/palette checksum " + name)
    count = sum(a != b for a, b in zip(before, after, strict=True))
    require(int(re.search(r"\bchanged_modules=(\d+)", text)[1]) == count, "native edit count")
    require("engine_instances=0" in text and "spacing=1.8" in text, "native replay/spacing")
    require(receipt.with_suffix(".png").exists(), "native image missing")
    result = dict(
        case=case.name,
        tick=tick,
        selected=selected,
        changed_modules=count,
        exact_observation_and_palette_checksums=True,
        engine_instances=0,
        spacing=1.8,
    )
    receipt.with_suffix(".verification.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("case", type=Path)
    parser.add_argument("receipt", type=Path)
    args = parser.parse_args()
    verify(args.case, args.receipt)
