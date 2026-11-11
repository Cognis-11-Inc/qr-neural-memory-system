"""Metrics for comparing binary memory patterns."""

from collections.abc import Sequence


def reconstruction_error(expected: Sequence[bool], actual: Sequence[bool]) -> float:
    """Return the fraction of mismatched bits for equal, nonempty binary patterns."""
    if not expected or len(expected) != len(actual):
        raise ValueError("patterns must be nonempty and have equal lengths")
    if any(type(bit) is not bool for pattern in (expected, actual) for bit in pattern):
        raise TypeError("patterns must contain bool values")
    return sum(a != b for a, b in zip(expected, actual, strict=True)) / len(expected)
