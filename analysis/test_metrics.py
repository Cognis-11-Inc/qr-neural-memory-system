"""Check metric endpoints, partial reconstruction, and invalid measurements."""

import unittest

from analysis.metrics import reconstruction_error


class ReconstructionErrorTests(unittest.TestCase):
    def test_known_reconstructions(self):
        expected = [True, False, True, False]
        self.assertEqual(reconstruction_error(expected, expected), 0.0)
        self.assertEqual(reconstruction_error(expected, [False, True, False, True]), 1.0)
        self.assertEqual(reconstruction_error(expected, [True, False, False, False]), 0.25)

    def test_invalid_measurements(self):
        for expected, actual in [([], []), ([True], []), ([True], [True, False])]:
            with self.subTest(expected=expected, actual=actual), self.assertRaises(ValueError):
                reconstruction_error(expected, actual)
        with self.assertRaises(TypeError):
            reconstruction_error([True], [1])


if __name__ == "__main__":
    unittest.main()
