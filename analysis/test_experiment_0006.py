"""Independent threshold/tie, shape invariants and QR6 corruption tests."""

import struct
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np

from analysis.experiment_0006 import (
    FIELDS,
    decode,
    fnv,
    internal,
    morph,
    numerical,
    planned,
    scores_from_history,
    variant,
)


class MorphologyTests(unittest.TestCase):
    def test_local_thresholds_disjoint_cardinal_swaps_and_masks(self):
        before = np.zeros((4096, 64), dtype=np.uint8)
        before[2182] = np.arange(64) % 5
        history = np.zeros((4096, 64))
        history[2182, :10] = 1e-8
        velocity = np.zeros_like(history)
        recolor = morph(before, velocity, history, 2)
        self.assertTrue(np.array_equal(np.flatnonzero(recolor[2182] != before[2182]), np.arange(8)))
        swaps = morph(before, velocity, history, 3)
        self.assertEqual(np.count_nonzero(swaps != before), 8)
        self.assertTrue(np.array_equal(np.sort(swaps, axis=1), np.sort(before, axis=1)))
        self.assertEqual(internal(0), [1, 8])
        self.assertNotIn(8, internal(7))
        self.assertTrue(np.array_equal(morph(before, velocity, np.zeros_like(history), 2), before))
        self.assertTrue(np.array_equal(variant(before, swaps, "OFF_SOURCE_ONLY", 3, 7), before))
        shuffled = variant(before, swaps, "SHUFFLE_POSITION", 3, 7)
        self.assertEqual(np.count_nonzero(shuffled != before), 8)
        self.assertTrue(np.array_equal(np.sort(shuffled, axis=1), np.sort(before, axis=1)))
        self.assertTrue(np.array_equal(before[0], np.zeros(64)))
        smoothed = scores_from_history(history)
        self.assertAlmostEqual(smoothed[2182, 0], 1.5e-8, delta=1e-22)
        self.assertEqual(len(planned()), 190)

    def test_numerical_gates_fail_without_relaxing_short_or_energy_limits(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            expected = [ident for ident, stage in planned() if stage == "N"]
            for ident in expected:
                (root / ident).mkdir()
                (root / ident / "COMPLETE").touch()
            audited = [dict(id=ident, max_energy_drift=0.001) for ident in expected]

            def state(_root, ident, time):
                dt = float(ident.split("-d")[-1])
                value = 1 + 0.1 * dt * dt * time
                return dict(u=np.array([value]), v=np.array([0.0]))

            with patch("analysis.experiment_0006.checkpoint", side_effect=state):
                result = numerical(root, audited)
            self.assertTrue(result["short_pass"] and result["long_pass"])

            def late_failure(_root, ident, time):
                result = state(_root, ident, time)
                if time == 307.2 and ident.endswith("d0.05"):
                    result["u"] = np.array([2.0])
                return result

            with patch("analysis.experiment_0006.checkpoint", side_effect=late_failure):
                result = numerical(root, audited)
            self.assertTrue(result["short_pass"])
            self.assertFalse(result["long_pass"])
            audited[0]["max_energy_drift"] = 0.011
            with patch("analysis.experiment_0006.checkpoint", side_effect=state):
                result = numerical(root, audited)
            self.assertTrue(result["complete"])
            self.assertFalse(result["energy_pass"])
            self.assertFalse(result["short_pass"] or result["long_pass"])
            (root / "n-zero-d0.0125" / "COMPLETE").unlink()
            self.assertFalse(numerical(root, audited)["complete"])

    def test_refined_snapshot_roundtrip_and_corruption(self):
        label = b"test"
        header = b"QR6FLD01" + struct.pack("<QQdQQQQ", 1, 0, 0.0125, 7, 1, len(label), 9)
        payload = header + label + bytes([1]) * FIELDS + bytes(16 * FIELDS)
        encoded = payload + struct.pack("<Q", fnv(payload))
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "snapshot.q6f"
            path.write_bytes(encoded)
            state = decode(path)
            self.assertEqual(state["module"], 9)
            self.assertEqual(state["dt"], 0.0125)
            path.write_bytes(encoded[:100] + bytes([encoded[100] ^ 1]) + encoded[101:])
            with self.assertRaisesRegex(ValueError, "checksum"):
                decode(path)
            path.write_bytes(encoded[:-2])
            with self.assertRaisesRegex(ValueError, "length"):
                decode(path)


if __name__ == "__main__":
    unittest.main()
