"""Observation checksum/layout boundaries and measurement comparison checks."""

import tempfile
import unittest
from pathlib import Path

from analysis.experiment_0003 import close, coordinates, fnv, read_snapshot, verify


class ObservationTests(unittest.TestCase):
    def test_known_checksum_and_coordinate_conventions(self):
        self.assertEqual(fnv(b"hello"), 0xA430D84680AABD0B)
        self.assertEqual(coordinates(2184, "Cubic6"), (8, 8, 8))
        self.assertEqual(coordinates(2080, "Grid2D8"), (32, 32, 0))
        self.assertEqual(coordinates(4095, "Cubic26"), (15, 15, 15))

    def test_corrupt_and_incomplete_evidence_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "bad.qrs"
            path.write_bytes(b"QR3OBS01" + bytes(90))
            with self.assertRaises(ValueError):
                read_snapshot(path)
            with self.assertRaises(ValueError):
                verify(Path(folder))
        with self.assertRaises(ValueError):
            close(float("nan"), 0.0)
