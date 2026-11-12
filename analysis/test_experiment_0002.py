"""Check spectral decision boundaries and independent numerical-verification helpers."""

import math
import tempfile
import unittest
from pathlib import Path

from analysis.experiment_0002 import (
    close,
    component,
    expected_keys,
    peak_test,
    relative_l2,
    rms,
    verify,
)


class DynamicsAnalysisTests(unittest.TestCase):
    def test_registered_schedule_and_incomplete_evidence(self):
        self.assertEqual(len(expected_keys()), 1101)
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(ValueError):
            verify(Path(directory))

    def test_spectral_peak_requires_controls_and_interior_resolution(self):
        self.assertTrue(peak_test([1, 2, 5, 2, 1, 0.5, 0.25])["resolved"])
        self.assertFalse(peak_test([1, 1, 1, 1, 1, 1, 1])["resolved"])
        boundary = peak_test([7, 6, 5, 4, 3, 2, 1])
        self.assertFalse(boundary["resolved"])
        self.assertFalse(boundary["interior"])

    def test_sine_phase_gain_and_relative_errors(self):
        times = [index * 0.1 for index in range(1, 641)]
        omega = math.tau * 8 / 64
        drive = [math.sin(omega * time) for time in times]
        response = [2 * math.sin(omega * time + math.pi / 3) for time in times]
        transfer = component(response, times, omega) / component(drive, times, omega)
        self.assertAlmostEqual(abs(transfer), 2.0)
        self.assertAlmostEqual(math.atan2(transfer.imag, transfer.real), math.pi / 3)
        self.assertAlmostEqual(rms(response) / rms(drive), 2.0)
        self.assertAlmostEqual(relative_l2([2 + 2j], [1 + 1j]), 1.0)
        self.assertEqual(relative_l2([0], [0]), 0.0)

    def test_nonfinite_or_mismatched_results_are_rejected(self):
        for bad in (math.nan, math.inf, 1.01):
            with self.subTest(value=bad), self.assertRaises(ValueError):
                close(bad, 1.0)
