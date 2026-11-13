"""Plot measured graph-shell peak envelopes; graph distances differ by topology."""

import argparse
import csv
import statistics
from collections import defaultdict
from pathlib import Path


def read(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    cases = {r["case"]: r for r in read(args.folder / "case-summary.csv")}
    shells = read(args.folder / "shell-summary.csv")
    fig, axes = plt.subplots(1, 2, figsize=(10, 4.5), layout="constrained")
    for topology, axis in zip(("Cubic6", "Cubic26"), axes, strict=True):
        for config in ("heterogeneous", "homogeneous", "rearranged"):
            data = defaultdict(list)
            for row in shells:
                case = cases[row["case"]]
                if (
                    case["topology"] == topology
                    and case["drive"] == "impulse"
                    and case["source"] == "2184"
                    and case["configuration"] == config
                ):
                    data[int(row["hops"])].append(float(row["peak_max"]))
            x = sorted(data)
            axis.plot(x, [statistics.median(data[h]) for h in x], label=config)
            axis.fill_between(x, [min(data[h]) for h in x], [max(data[h]) for h in x], alpha=0.15)
        axis.axhline(1e-6, color="black", linestyle="--", linewidth=1)
        axis.set(
            title=topology,
            xlabel="External graph hops (policy-specific)",
            ylabel="Largest neuron peak in shell",
            yscale="log",
            ylim=(1e-20, 1.5),
        )
        axis.grid(alpha=0.2)
    axes[0].legend(fontsize=8)
    fig.suptitle("Unit impulse, center source: median and min-max over registered seeds")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(args.output, dpi=150)
    plt.close(fig)


if __name__ == "__main__":
    main()
