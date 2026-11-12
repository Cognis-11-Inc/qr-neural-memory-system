"""Independent colored-output scoring and incomplete-evidence rejection."""

import tempfile
import unittest
from pathlib import Path

from analysis.experiment_0001 import colored_errors, summarize


class ExperimentEvidenceTests(unittest.TestCase):
    def test_missing_outputs_and_empty_targets(self):
        blank = bytes(1024)
        target = bytes([2]) * 1024
        self.assertEqual(colored_errors(blank, target), (1.0, 1.0, 1.0))
        self.assertEqual(colored_errors(target, blank), (1.0, 1.0, 0.0))
        self.assertEqual(colored_errors(bytes([3]) * 1024, target), (1.0, 0.0, 1.0))
        self.assertEqual(colored_errors(target, target), (0.0, 0.0, 0.0))

    def test_invalid_evidence_is_not_a_success(self):
        with self.assertRaises(ValueError):
            colored_errors(bytes(1023), bytes(1024))
        with self.assertRaises(ValueError):
            colored_errors(bytes([5]) * 1024, bytes(1024))
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(ValueError):
            summarize(Path(directory))
