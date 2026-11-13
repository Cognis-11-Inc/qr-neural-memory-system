"""Standalone measured convergence and re-exposure figure; no synthetic fields."""

import argparse
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

from analysis.experiment_0005 import rows


def plot(root, destination):
    data = json.loads((root / "verification.json").read_text())
    figure, axes = plt.subplots(1, 3, figsize=(15, 4.2), constrained_layout=True)
    for fixture, color in (("n-homogeneous", "tab:blue"), ("n-heterogeneous", "tab:orange")):
        selected = [r for r in data["numerical"]["comparisons"] if r["fixture"] == fixture]
        for key, style in (("coarse", "-"), ("fine", "--")):
            axes[0].plot(
                [r["time"] for r in selected],
                [100 * r[key] for r in selected],
                style,
                color=color,
                label=f"{fixture[2:]} {'.1/.05' if key == 'coarse' else '.05/.025'}",
            )
    axes[0].set(
        xscale="log",
        xlabel="Physical time",
        ylabel="Signed field relative L2 error (%)",
        title="Long-window gate fails",
    )
    axes[0].axhline(5, color="gray", alpha=0.5)
    axes[0].axhline(10, color="gray", alpha=0.5, linestyle=":")
    axes[0].legend(fontsize=7)
    for name, label in (
        ("c-frozen-0.05", "virgin P"),
        ("c-reexpose-local-0.05", "retained P after local exposure"),
        ("c-reexpose-time-0.05", "generic-time control P"),
    ):
        probe = [r for r in rows(root / name / "probes.csv") if int(r["id"]) == 2182]
        axes[1].plot(
            [float(r["time"]) for r in probe],
            [float(r["u27"]) for r in probe],
            label=label,
            linewidth=1,
        )
    axes[1].set(
        xlabel="Re-exposure time",
        ylabel="Source A signed module27 u",
        title="Identical S after transient reset",
    )
    axes[1].legend(fontsize=7)
    for dt, marker in ((0.05, "o"), (0.025, "x")):
        result = next(r for r in data["plastic"] if r["dt"] == dt)
        axes[2].plot(
            [16, 32], [100 * x for x in result["effects"]], marker=marker, label=f"local dt{dt}"
        )
        axes[2].plot(
            [16, 32],
            [100 * x for x in result["generic_time_effects"]],
            ":",
            marker=marker,
            label=f"time control dt{dt}",
        )
    axes[2].set(
        xlabel="Re-exposure checkpoint",
        ylabel="Field L2 difference / virgin norm (%)",
        title="Persistent response ≠ associative recall",
    )
    axes[2].legend(fontsize=7)
    destination.parent.mkdir(parents=True, exist_ok=True)
    figure.savefig(destination, dpi=160)
    plt.close(figure)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    plot(args.root, args.destination)
