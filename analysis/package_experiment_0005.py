"""Package viewer-free traces and separately retained full signed checkpoints."""

import argparse
import csv
import hashlib
import io
import json
import zipfile
from pathlib import Path

from analysis.experiment_0005 import require


def package(root, native, repeat):
    verification = json.loads((root / "verification.json").read_text())
    require(len(verification["runs"]) == 51, "expected 51 completed primary runs")
    reference = root / "a-heterogeneous-7-AB+"
    repeated = repeat / "a-heterogeneous-7-AB+"
    require(
        {p.name for p in reference.iterdir()} == {p.name for p in repeated.iterdir()},
        "repeat coverage",
    )
    require(
        all(p.read_bytes() == (repeated / p.name).read_bytes() for p in reference.iterdir()),
        "repeat bytes",
    )
    receipt = {
        "case": reference.name,
        "all_case_files_byte_identical": True,
        "total_primary_executions_including_repeat": 52,
        "supplemental_transition_replays": 4,
    }
    (root / "repeat-verification.json").write_text(json.dumps(receipt, indent=2) + "\n")
    entries = {}
    signed = []
    for folder in sorted(root.iterdir()):
        if folder.is_dir() and (folder / "COMPLETE").exists():
            for file in sorted(folder.iterdir()):
                if file.suffix == ".q5f":
                    signed.append(file)
                else:
                    entries[file.relative_to(root).as_posix()] = file
    for file in (root / "transition-audit").rglob("*"):
        if file.suffix == ".q5f":
            signed.append(file)
        elif file.is_file():
            entries[file.relative_to(root).as_posix()] = file
    for name in (
        "RULES.md",
        "verification.json",
        "execution-ledger.csv",
        "NUMERICAL_GATES.csv",
        "repeat-verification.json",
        "REPORT.md",
    ):
        entries[name] = root / name
    for file in root.glob("ledger-*.csv"):
        entries[file.name] = file
    for subfolder in ("sources", "derived"):
        for file in (root / subfolder).rglob("*"):
            if file.is_file():
                entries[file.relative_to(root).as_posix()] = file
    for name in (
        "EXPERIMENT_0005.md",
        "requirements-analysis.txt",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
    ):
        entries[name] = Path(name)
    for name in (
        "analysis/__init__.py",
        "analysis/experiment_0005.py",
        "analysis/package_experiment_0005.py",
        "analysis/plot_experiment_0005.py",
        "docs/EXPERIMENT_0005_SCHEMA.md",
        "docs/EXPERIMENT_0005_PROTECTED_SHA256.txt",
    ):
        entries[name] = Path(name)
    for file in Path("src").rglob("*.rs"):
        entries[file.as_posix()] = file
    for file in native.rglob("*"):
        if file.is_file():
            entries["native/" + file.relative_to(native).as_posix()] = file
    for pattern in ("EXPERIMENT_*.md", "docs/EXPERIMENT_*.md", "tests/fixtures/*"):
        for file in Path(".").glob(pattern):
            if file.is_file():
                entries[file.as_posix()] = file
    entries["requirements-plot.txt"] = Path("requirements-plot.txt")
    manifest = io.StringIO()
    writer = csv.writer(manifest)
    writer.writerow(("path", "sha256"))
    for file in signed:
        writer.writerow(
            (file.relative_to(root).as_posix(), hashlib.sha256(file.read_bytes()).hexdigest())
        )
    (root / "SIGNED_FIELDS_SHA256.csv").write_text(manifest.getvalue())
    entries["SIGNED_FIELDS_SHA256.csv"] = root / "SIGNED_FIELDS_SHA256.csv"
    checksums = io.StringIO()
    writer = csv.writer(checksums)
    writer.writerow(("path", "sha256"))
    for name, file in sorted(entries.items()):
        writer.writerow((name, hashlib.sha256(file.read_bytes()).hexdigest()))
    (root / "FILE_SHA256.csv").write_text(checksums.getvalue())
    entries["FILE_SHA256.csv"] = root / "FILE_SHA256.csv"
    for name, files in (
        ("AGENT_REPORT.zip", entries),
        ("FULL_FIELDS.zip", {p.relative_to(root).as_posix(): p for p in signed}),
    ):
        destination = root / name
        with zipfile.ZipFile(
            destination, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6
        ) as archive:
            for member, file in sorted(files.items()):
                # Fixed member metadata makes repeat packaging deterministic.
                info = zipfile.ZipInfo(member, date_time=(2026, 11, 13, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(info, file.read_bytes(), compresslevel=6)
        with zipfile.ZipFile(destination) as archive:
            require(archive.testzip() is None, "ZIP CRC")
        print(
            destination,
            destination.stat().st_size,
            hashlib.sha256(destination.read_bytes()).hexdigest(),
        )
    with zipfile.ZipFile(root / "AGENT_REPORT.zip") as archive:
        for row in csv.DictReader(io.StringIO(archive.read("FILE_SHA256.csv").decode())):
            require(
                hashlib.sha256(archive.read(row["path"])).hexdigest() == row["sha256"],
                "package member integrity",
            )
    with zipfile.ZipFile(root / "FULL_FIELDS.zip") as archive:
        for row in csv.DictReader(io.StringIO(manifest.getvalue())):
            require(
                hashlib.sha256(archive.read(row["path"])).hexdigest() == row["sha256"],
                "full signed package member integrity",
            )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--repeat", type=Path, required=True)
    args = parser.parse_args()
    package(args.root, args.native, args.repeat)


if __name__ == "__main__":
    main()
