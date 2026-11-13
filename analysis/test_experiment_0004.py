"""Independent format, coverage and signed-superposition failure tests."""

import struct
import tempfile
import unittest
from pathlib import Path

from analysis.experiment_0003 import fnv
from analysis.experiment_0004 import FIELDS, MODEL, expected_cases, norms, read_frame


class FrozenMediumTests(unittest.TestCase):
    def test_registered_coverage_and_signed_cancellation(self):
        self.assertEqual(len(expected_cases()), 35)

        def frame(u, v):
            return {"u": u, "v": v}

        a = frame([1.0, -2.0], [3.0, 4.0])
        b = frame([-1.0, 2.0], [1.0, -4.0])
        plus = frame([0.0, 0.0], [4.0, 0.0])
        minus = frame([2.0, -4.0], [2.0, 8.0])
        self.assertEqual(list(norms(a, b, plus, minus))[-1][1], [0.0, 0.0])
        plus["u"][0] = 0.001
        with self.assertRaises(ValueError):
            list(norms(a, b, plus, minus))

    def test_full_field_provenance_and_corruption(self):
        data = bytearray(
            b"QR4FLD01"
            + struct.pack("<I4BQQdIII", 1, 0, 0, 4, 0, 7, 0, 0.1, 2182, 2186, len(MODEL))
            + MODEL
        )
        data.extend(bytes(17 * FIELDS))
        data.extend(struct.pack("<Q", fnv(data)))
        with tempfile.TemporaryDirectory() as folder:
            p = Path(folder) / "frame.q4f"
            p.write_bytes(data)
            decoded = read_frame(p)
            self.assertFalse(any(decoded["u"]))
            self.assertFalse(any(decoded["v"]))
            data[100] ^= 1
            p.write_bytes(data)
            with self.assertRaises(ValueError):
                read_frame(p)
            p.write_bytes(data[:100])
            with self.assertRaises(ValueError):
                read_frame(p)
