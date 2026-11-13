"""Create an explicitly partial, smaller QR6 review bundle alongside full evidence."""

import argparse
import csv
import hashlib
import io
import json
import zipfile
from pathlib import Path


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def package(root):
    entries = {}
    for row in csv.DictReader((root / "execution-ledger.csv").open()):
        folder = root / row["id"]
        names = (
            ("REUSED",)
            if row["status"] == "REUSED"
            else ("parameters.csv", "metrics.csv", "before.palette", "after.palette")
        )
        for name in names:
            path = folder / name
            entries[path.relative_to(root).as_posix()] = path
    for path in (root / "agent-report").iterdir():
        if path.is_file():
            entries[path.relative_to(root).as_posix()] = path
    for name in (
        "REPORT.md",
        "RULES.md",
        "verification.json",
        "execution-ledger.csv",
        "GATES.csv",
        "repeat-verification.json",
    ):
        entries[name] = root / name
    for path in Path("docs/figures").glob("experiment_0006*.png"):
        entries["figures/" + path.name] = path
    entries["INVENTORY.md"] = Path("docs/EXPERIMENT_0006_INVENTORY.md")
    entries["SCHEMA.md"] = Path("docs/EXPERIMENT_0006_SCHEMA.md")
    entries["analysis/quick_package_0006.py"] = Path(__file__)
    description = (
        "# Experiment 0006 quick review subset\n\n"
        "Start with REPORT.md and agent-report/*.csv. This subset contains all "
        "primary palette files, global metric traces, input parameters, complete "
        "logical ledger, derived diagnosis/morphology/causal tables and original "
        "verification receipts. REUSED entries name their original case.\n\n"
        "It omits full per-module eligibility/history, complete signed probe "
        "streams and full signed checkpoints. Full independent field/rule "
        "re-verification requires AGENT_REPORT.zip plus FULL_FIELDS.zip. "
        "These omissions do not imply unexecuted experimental conditions.\n"
    )
    manifest = io.StringIO()
    writer = csv.writer(manifest)
    writer.writerow(("path", "sha256"))
    for name, path in sorted(entries.items()):
        writer.writerow((name, sha(path)))
    destination = root / "QUICK_REPORT.zip"
    with zipfile.ZipFile(
        destination, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6
    ) as archive:
        for name, path in sorted(entries.items()):
            archive.write(path, name)
        archive.writestr("README.md", description)
        archive.writestr("FILE_SHA256.csv", manifest.getvalue())
    with zipfile.ZipFile(destination) as archive:
        assert archive.testzip() is None
        for row in csv.DictReader(io.StringIO(manifest.getvalue())):
            assert hashlib.sha256(archive.read(row["path"])).hexdigest() == row["sha256"]
    result = dict(
        file=destination.name,
        bytes=destination.stat().st_size,
        sha256=sha(destination),
        complete_dataset=False,
    )
    (root / "quick-package-verification.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    package(parser.parse_args().root)
