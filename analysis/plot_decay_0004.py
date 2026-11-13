"""Plot measured every-tick global decay and instantaneous threshold counts."""

import argparse
import csv
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    with (args.folder / "metrics.csv").open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    ticks = [int(r["tick"]) for r in rows]
    fig, axes = plt.subplots(2, 1, figsize=(10, 6), sharex=True, layout="constrained")
    for name, label in (("peak", "Global max |u|"), ("velocity_peak", "Global max |v|")):
        axes[0].semilogy(ticks, [float(r[name]) or float("nan") for r in rows], label=label)
    axes[0].axhline(1e-6, color="black", linestyle="--", label="Quiet threshold (both fields)")
    axes[0].set(ylabel="Measured maximum magnitude")
    axes[0].legend()
    axes[1].plot(ticks, [int(r["active_neurons"]) for r in rows], label="Neuron max |u| >= 1e-6")
    axes[1].set(
        xlabel="Integration tick (dt = 0.1)",
        ylabel="Instantaneously active neurons",
        ylim=(0, 4200),
    )
    axes[1].legend()
    last_above = max(
        int(r["tick"])
        for r in rows
        if float(r["peak"]) >= 1e-6 or float(r["velocity_peak"]) >= 1e-6
    )
    for ax in axes:
        if last_above < ticks[-1]:
            ax.axvline(last_above + 1, color="gray", linestyle=":")
            ax.axvspan(last_above + 1, ticks[-1], color="green", alpha=0.08)
        ax.grid(alpha=0.2)
    fig.suptitle(
        "3,072-tick frozen-medium extension: measured decay, not exact zero\n"
        "Homogeneous seed7 AB+ · prior long-window timestep accuracy screen failed"
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(args.output, dpi=150)
    plt.close(fig)


if __name__ == "__main__":
    main()
