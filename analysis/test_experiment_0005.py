"""Independent invariants for the new signed-field energy audit."""

import struct
import tempfile
import unittest
from pathlib import Path

import numpy as np

from analysis.experiment_0005 import FIELDS, coefficients, decode, energy, field_error, fnv


class MorphicTests(unittest.TestCase):
    def test_energy_counts_each_edge_once_and_shape_matters(self):
        palette = np.ones((4096, 8, 8), dtype=np.uint8)
        u, v = np.zeros(palette.shape), np.zeros(palette.shape)
        u[2182, 3, 3] = 1
        kinetic, onsite, coupling, local = energy(palette, u, v)
        self.assertEqual(kinetic, 0)
        self.assertAlmostEqual(onsite, 0.5 * 1.16**2)
        self.assertAlmostEqual(coupling, 0.5 * (4 * 0.15 + 6 * 0.4 / 6))
        self.assertAlmostEqual(float(np.sum(local)), kinetic + onsite + coupling)
        palette[2182, 3, 2] = 0
        omega, _, _ = coefficients(palette)
        self.assertAlmostEqual(omega[2182, 3, 3], 1.12**2)
        palette[2182, 3, 2], palette[2182, 0, 0] = 1, 0
        other, _, _ = coefficients(palette)
        self.assertNotEqual(omega[2182, 3, 3], other[2182, 3, 3])

    def test_qr5_exact_decode_and_corruption_rejection(self):
        label = b"test"
        header = b"QR5FLD01" + struct.pack("<QQdQQQ", 1, 0, 0.05, 7, 1, len(label))
        content = header + label + bytes([1]) * FIELDS + bytes(16 * FIELDS)
        data = content + struct.pack("<Q", fnv(content))
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "state.q5f"
            path.write_bytes(data)
            frame = decode(path)
            self.assertEqual(frame["label"], "test")
            self.assertEqual(field_error(frame, frame), 0)
            path.write_bytes(data[:100] + bytes([data[100] ^ 1]) + data[101:])
            with self.assertRaisesRegex(ValueError, "checksum"):
                decode(path)
            path.write_bytes(data[:-1])
            with self.assertRaisesRegex(ValueError, "size"):
                decode(path)


if __name__ == "__main__":
    unittest.main()
