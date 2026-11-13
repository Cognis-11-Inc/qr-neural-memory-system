"""Stream portable QR6 observations and optional full signed fields into checked ZIPs."""

import argparse
import csv
import hashlib
import io
import json
import shutil
import zipfile
from pathlib import Path

from analysis.experiment_0006 import planned, require


def sha_file(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def package(root, native, repeat):
    expected = {ident for ident, _ in planned()}
    ledger = list(csv.DictReader((root / "execution-ledger.csv").open()))
    require({r["id"] for r in ledger} == expected and len(ledger) == 190, "full registered ledger")
    require(
        sum(r["status"] == "COMPLETE" for r in ledger) == 168
        and sum(r["status"] == "REUSED" for r in ledger) == 22,
        "planned actual/reused counts",
    )
    ident = "m-s19-m9-E1-R3-d0.025"
    primary, repeated = root / ident, repeat / ident
    original_files = {
        p.relative_to(primary).as_posix(): p for p in primary.rglob("*") if p.is_file()
    }
    repeated_files = {
        p.relative_to(repeated).as_posix(): p for p in repeated.rglob("*") if p.is_file()
    }
    require(original_files.keys() == repeated_files.keys(), "repeat file coverage")
    require(
        all(sha_file(p) == sha_file(repeated_files[name]) for name, p in original_files.items()),
        "preselected repeat bytes",
    )
    (root / "repeat-verification.json").write_text(
        json.dumps(
            dict(case=ident, all_case_files_byte_identical=True, cross_platform_guarantee=False),
            indent=2,
        )
        + "\n"
    )
    old_manifest = None
    previous = []
    if (root / "package-verification.json").exists() and (root / "AGENT_REPORT.zip").exists():
        previous = json.loads((root / "package-verification.json").read_text())
        with zipfile.ZipFile(root / "AGENT_REPORT.zip") as old:
            old_manifest = old.read("SIGNED_FIELDS_SHA256.csv").decode()
    portable, full = {}, {}
    for ident in sorted(expected):
        folder = root / ident
        for file in folder.rglob("*"):
            if file.is_file():
                target = full if file.suffix == ".q6f" else portable
                target[file.relative_to(root).as_posix()] = file
    for sub in ("sources", "derived", "agent-report"):
        for file in (root / sub).rglob("*"):
            if file.is_file():
                portable[file.relative_to(root).as_posix()] = file
    for name in (
        "RULES.md",
        "verification.json",
        "execution-ledger.csv",
        "GATES.csv",
        "repeat-verification.json",
        "REPORT.md",
    ):
        portable[name] = root / name
    for file in native.rglob("*"):
        if file.is_file():
            portable["native/" + file.relative_to(native).as_posix()] = file
    # Root CPU sources, exact old include files, independent verifier and schemas.
    for pattern in (
        "src/**/*.rs",
        "analysis/*.py",
        "tests/fixtures/*",
        "EXPERIMENT_*.md",
        "docs/EXPERIMENT_*.md",
        "requirements*.txt",
    ):
        for file in Path(".").glob(pattern):
            if file.is_file():
                portable[file.as_posix()] = file
    for name in (
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "README.md",
        "PRINCIPLES.md",
        "HYPOTHESES.md",
        "docs/EXPERIMENT_0006_PROTECTED_SHA256.txt",
    ):
        portable[name] = Path(name)
    for file in Path("docs/figures").glob("experiment_0006*.png"):
        portable[file.as_posix()] = file
        portable["figures/" + file.name] = file
    portable["INVENTORY.md"] = Path("docs/EXPERIMENT_0006_INVENTORY.md")
    portable["SCHEMA.md"] = Path("docs/EXPERIMENT_0006_SCHEMA.md")
    signed_manifest = io.StringIO()
    writer = csv.writer(signed_manifest)
    writer.writerow(("path", "sha256"))
    for name, file in sorted(full.items()):
        writer.writerow((name, sha_file(file)))
    (root / "SIGNED_FIELDS_SHA256.csv").write_text(signed_manifest.getvalue())
    portable["SIGNED_FIELDS_SHA256.csv"] = root / "SIGNED_FIELDS_SHA256.csv"
    member_manifest = io.StringIO()
    writer = csv.writer(member_manifest)
    writer.writerow(("path", "sha256"))
    for name, file in sorted(portable.items()):
        writer.writerow((name, sha_file(file)))
    (root / "FILE_SHA256.csv").write_text(member_manifest.getvalue())
    portable["FILE_SHA256.csv"] = root / "FILE_SHA256.csv"
    results = []
    for name, files, manifest in (
        ("AGENT_REPORT.zip", portable, member_manifest),
        ("FULL_FIELDS.zip", full, signed_manifest),
    ):
        destination = root / name
        prior = next((r for r in previous if r["file"] == name), None)
        reusable = (
            name == "FULL_FIELDS.zip"
            and destination.exists()
            and prior is not None
            and old_manifest == signed_manifest.getvalue()
            and sha_file(destination) == prior["sha256"]
        )
        if not reusable:
            with zipfile.ZipFile(destination, "w", allowZip64=True) as archive:
                for member, file in sorted(files.items()):
                    info = zipfile.ZipInfo(member, date_time=(2026, 11, 13, 0, 0, 0))
                    info.compress_type = (
                        zipfile.ZIP_STORED if file.suffix == ".gz" else zipfile.ZIP_DEFLATED
                    )
                    with (
                        file.open("rb") as source,
                        archive.open(info, "w", force_zip64=True) as target,
                    ):
                        shutil.copyfileobj(source, target, 1024 * 1024)
        with zipfile.ZipFile(destination) as archive:
            require(set(archive.namelist()) == set(files), "ZIP member coverage")
            require(archive.testzip() is None, "ZIP CRC")
            for row in csv.DictReader(io.StringIO(manifest.getvalue())):
                with archive.open(row["path"]) as stream:
                    require(
                        hashlib.file_digest(stream, "sha256").hexdigest() == row["sha256"],
                        "ZIP member SHA",
                    )
        result = dict(
            file=name,
            bytes=destination.stat().st_size,
            sha256=sha_file(destination),
            members=len(files),
        )
        results.append(result)
        print(json.dumps(result), flush=True)
    (root / "package-verification.json").write_text(json.dumps(results, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--repeat", type=Path, required=True)
    args = parser.parse_args()
    package(args.root, args.native, args.repeat)
