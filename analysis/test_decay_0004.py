"""Every-tick format, tail-decision and portable-package correctness tests."""

import csv
import hashlib
import struct
import tempfile
import unittest
import zipfile
from pathlib import Path

from analysis.decay_0004 import (
    FRAME_SIZE,
    HEADER_SIZE,
    LAST,
    MODEL,
    decode_frame,
    header,
    package,
    quiet_from_metrics,
)
from analysis.experiment_0003 import fnv


class EveryTickDecayTests(unittest.TestCase):
    def test_missing_corrupt_and_wrong_tick_frames(self):
        data = bytearray(FRAME_SIZE)
        struct.pack_into("<Q", data, FRAME_SIZE - 8, fnv(data[:-8]))
        decoded, checksum = decode_frame(data, 0)
        self.assertEqual(len(list(decoded)), 4096)
        self.assertEqual(len(checksum), 16)
        with self.assertRaises(ValueError):
            decode_frame(data, 1)
        with self.assertRaises(ValueError):
            decode_frame(data[:-1], 0)
        data[200] ^= 1
        with self.assertRaises(ValueError):
            decode_frame(data, 0)

    def test_provenance_and_final_tail_after_recrossing(self):
        data = bytearray(
            b"QR4DEC01"
            + struct.pack("<IQdQII4BI", 1, 7, 0.1, LAST, 2182, 2186, 1, 2, 0, 0, len(MODEL))
            + MODEL
            + bytes([1]) * 4096 * 64
        )
        data.extend(struct.pack("<Q", fnv(data)))
        self.assertEqual(len(header(data, HEADER_SIZE + (LAST + 1) * FRAME_SIZE)), 64)
        with self.assertRaises(ValueError):
            header(data, HEADER_SIZE + LAST * FRAME_SIZE)
        trace = [{"tick": str(t), "peak": "1e-8", "velocity_peak": "2e-8"} for t in range(260)]
        trace[0]["peak"] = "1"
        trace[131]["velocity_peak"] = "1e-6"
        self.assertEqual(
            quiet_from_metrics(trace),
            {"quiet_tail": 128, "sustained_quiet": True, "exact_zero": False},
        )
        trace[132]["peak"] = "1e-6"
        self.assertFalse(quiet_from_metrics(trace)["sustained_quiet"])

    def test_portable_package_hashes_and_repeatability(self):
        with tempfile.TemporaryDirectory() as temporary:
            report = Path(temporary) / "agent-report"
            report.mkdir()
            (report / "REPORT.md").write_text("Measured data\n")
            archive = package(report)
            before = archive.read_bytes()
            self.assertEqual(package(report).read_bytes(), before)
            with zipfile.ZipFile(archive) as zipped:
                entries = list(csv.DictReader(zipped.read("FILE_SHA256.csv").decode().splitlines()))
                self.assertEqual(
                    entries[0]["sha256"], hashlib.sha256(zipped.read("REPORT.md")).hexdigest()
                )
