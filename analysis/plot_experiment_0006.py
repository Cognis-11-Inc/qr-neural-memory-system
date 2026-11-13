"""Measured QR6 numerical, morphology and causal figures, no synthetic fields."""

import argparse
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.colors import ListedColormap

from analysis.experiment_0006 import resolve, rows


def plot(root, destination):
    destination.mkdir(parents=True, exist_ok=True)
    data = json.loads((root / "verification.json").read_text())
    figure, axes = plt.subplots(1, 2, figsize=(12, 4.5), constrained_layout=True)
    for config, seed, color in (
        ("homogeneous", 7, "tab:blue"),
        ("heterogeneous", 7, "tab:orange"),
        ("heterogeneous", 19, "tab:green"),
    ):
        selected = [
            r
            for r in data["numerical"]["comparisons"]
            if r["config"] == config and r["seed"] == seed
        ]
        for key, style, label in (("coarse", "-", ".05/.025"), ("fine", "--", ".025/.0125")):
            axes[0].plot(
                [r["time"] for r in selected],
                [100 * r[key]["relative_l2"] for r in selected],
                style,
                color=color,
                marker="o",
                markersize=3,
                label=f"{config} {seed} {label}",
            )
        metrics = rows(root / f"n-{config}-s{seed}-d0.025" / "metrics.csv")
        initial = sum(float(metrics[0][key]) for key in ("kinetic", "onsite", "coupling"))
        axes[1].plot(
            [float(r["time"]) for r in metrics],
            [100 * float(r["drift"]) / initial for r in metrics],
            color=color,
            label=f"{config} {seed}",
        )
    axes[0].set(
        xscale="log",
        xlabel="Physical time",
        ylabel="Unaligned signed-field relative L2 (%)",
        title="New registered refined-grid sensitivity passes",
    )
    axes[0].legend(fontsize=7)
    axes[1].set(
        xlabel="Physical time",
        ylabel="Cumulative drift / initial energy (%)",
        title="Energy audit is distinct from trajectory error",
    )
    axes[1].legend(fontsize=8)
    figure.savefig(destination / "experiment_0006_numerics.png", dpi=160)
    plt.close(figure)
    figure, axes = plt.subplots(1, 3, figsize=(12, 4), constrained_layout=True)
    folder = root / "m-s7-m27-E1-R3-d0.025"
    before = np.frombuffer((folder / "before.palette").read_bytes(), dtype="u1").reshape(
        4096, 8, 8
    )[2182]
    after = np.frombuffer((folder / "after.palette").read_bytes(), dtype="u1").reshape(4096, 8, 8)[
        2182
    ]
    color = ListedColormap(["#090d14", "#e83332", "#22c966", "#336df5", "#f4cd30"])
    for axis, tile, title in zip(
        axes,
        (before, after, before != after),
        ("Recorded P before", "Recorded P after (R3)", "Eight changed positions"),
        strict=True,
    ):
        axis.imshow(
            tile,
            cmap=color if title != "Eight changed positions" else "binary",
            vmin=0,
            vmax=4 if title != "Eight changed positions" else 1,
            interpolation="nearest",
        )
        axis.set(title=title, xlabel="Internal column", ylabel="Internal row")
        axis.set_xticks(range(8))
        axis.set_yticks(range(8))
    figure.suptitle("Source neuron 2182, seed7, module27 input; palette histogram preserved")
    figure.savefig(destination / "experiment_0006_palette.png", dpi=160)
    plt.close(figure)
    if data.get("causal"):
        figure, axes = plt.subplots(1, 2, figsize=(12, 4.3), constrained_layout=True)
        for axis, budget in zip(axes, ("UNIT", "MATCH"), strict=True):
            for kind, label in (
                ("VIRGIN", "Virgin"),
                ("SOURCE_ONLY", "Source palette only"),
                ("OFF_SOURCE_ONLY", "Off-source palettes only"),
                ("ALL", "All retained palettes"),
            ):
                ident = f"c-s7-m27-R3-d0.025-{kind}-{budget}"
                trace = [
                    r for r in rows(resolve(root, ident) / "probes.csv") if int(r["id"]) == 2182
                ]
                axis.plot(
                    [float(r["time"]) for r in trace],
                    [float(r["u_source"]) for r in trace],
                    label=label,
                    linewidth=1,
                )
            axis.set(
                xlabel="Re-exposure time",
                ylabel="Signed source-module displacement",
                title="Same unit displacement"
                if budget == "UNIT"
                else "Equal initial mechanical energy",
            )
            axis.legend(fontsize=7)
        figure.suptitle("Seed7/module27/R3 after complete transient reset; frozen re-exposure")
        figure.savefig(destination / "experiment_0006_reexposure.png", dpi=160)
        plt.close(figure)
        figure, axis = plt.subplots(figsize=(12, 4.4), constrained_layout=True)
        for offset, budget, color in ((-0.18, "UNIT", "tab:blue"), (0.18, "MATCH", "tab:orange")):
            selected = [
                r
                for r in data["causal"]
                if r["dt"] == 0.025 and r["kind"] == "OFF_SOURCE_ONLY" and r["budget"] == budget
            ]
            selected.sort(key=lambda r: (r["seed"], r["module"], r["rule"]))
            labels = [f"s{r['seed']} m{r['module']} R{r['rule']}" for r in selected]
            values = [100 * r["effects"][-1]["offsource_effect"]["relative_l2"] for r in selected]
            axis.bar(
                np.arange(len(selected)) + offset, values, width=0.34, color=color, label=budget
            )
        axis.axhline(1, color="black", linestyle=":", label="Registered effect threshold 1%")
        axis.set(
            xticks=np.arange(len(labels)),
            xticklabels=labels,
            ylabel="Off-source field difference / virgin norm (%)",
            title="OFF_SOURCE_ONLY vs VIRGIN at32; differences are not percent contributions",
        )
        axis.legend(fontsize=8)
        figure.savefig(destination / "experiment_0006_offsource.png", dpi=160)
        plt.close(figure)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    plot(args.root, args.destination)
