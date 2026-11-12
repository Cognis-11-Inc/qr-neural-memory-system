"""Plot verified Experiment 0002 measurements, retaining all registered seeds."""

import argparse
import csv
import statistics
from collections import defaultdict
from pathlib import Path


def read(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def plot(folder: Path, output: Path) -> None:
    """Create static scientific figures from independently verified tables."""
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    output.mkdir(parents=True, exist_ok=True)
    rows = read(folder / "transport-verified.csv")
    fig, axis = plt.subplots(figsize=(8, 4.5), layout="constrained")
    for drive in ("impulse", "burst"):
        values = defaultdict(list)
        for row in rows:
            if (
                row["source"] == "center"
                and row["drive"] == drive
                and row["scope"] == "external"
                and (row["label"] == "origin" or row["label"][1:].isdigit())
            ):
                values[int(row["hops"])].append(float(row["source_transfer"]))
        x = sorted(values)
        axis.plot(x, [statistics.median(values[h]) for h in x], "o-", label=drive)
        axis.fill_between(x, [min(values[h]) for h in x], [max(values[h]) for h in x], alpha=0.18)
    axis.axhline(1e-3, color="black", linestyle="--", label="registered transfer threshold")
    axis.set(
        xlabel="Cardinal graph hops from source neuron",
        ylabel="RMS displacement / source RMS",
        yscale="log",
        xticks=[0, 1, 2, 4, 8, 16],
        title="Full-network transport: median and observed min-max envelope",
    )
    axis.legend(fontsize=8)
    axis.grid(alpha=0.2)
    fig.savefig(output / "experiment_0002_transport.png", dpi=160)
    plt.close(fig)

    rows = read(folder / "responses-verified.csv")
    fig, axes = plt.subplots(2, 2, figsize=(10, 7), layout="constrained")
    for channel, axis in enumerate(axes.flat, 1):
        for configuration in ("homogeneous", "heterogeneous", "rearranged"):
            values = defaultdict(list)
            for row in rows:
                if (
                    row["configuration"] == configuration
                    and row["channel"] == str(channel)
                    and row["mode"] == "isolated"
                    and row["drive"] == "tone"
                    and float(row["dt"]) == 0.1
                    and float(row["gamma_factor"]) == 1
                ):
                    values[float(row["omega"])].append(float(row["gain"]))
            x = sorted(values)
            axis.plot(x, [statistics.median(values[w]) for w in x], "o-", label=configuration)
            axis.fill_between(
                x, [min(values[w]) for w in x], [max(values[w]) for w in x], alpha=0.15
            )
        axis.set(
            title=f"Channel {channel}", xlabel="Angular drive frequency", ylabel="Window RMS gain"
        )
        axis.grid(alpha=0.2)
    axes[0, 0].legend(fontsize=8)
    fig.suptitle("Isolated QR neurons: all three seeds, dt=0.1; overall Stage R indeterminate")
    fig.savefig(output / "experiment_0002_frequency_response.png", dpi=160)
    plt.close(fig)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    plot(args.folder, args.output)


if __name__ == "__main__":
    main()
