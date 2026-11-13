"""Plot signed, measured central-plane fields; fixed scale, no per-frame gain."""

import argparse
from pathlib import Path

from analysis.experiment_0004 import ARMS, TIMES, read_frame, rows


def plane(frame):
    return [[frame["u"][(x + 16 * (y + 16 * 8)) * 64 + 27] for x in range(16)] for y in range(16)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--movie", type=Path)
    args = parser.parse_args()
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from matplotlib.animation import FFMpegWriter
    from matplotlib.colors import SymLogNorm

    args.output.mkdir(parents=True, exist_ok=True)
    fig, axes = plt.subplots(2, 1, figsize=(10, 6), layout="constrained")
    for g, ax, label in zip((0, 1), axes, ("heterogeneous", "homogeneous"), strict=True):
        data = rows(args.folder / f"group-{g:02}/metrics.csv")
        for arm in ARMS:
            r = [x for x in data if x["arm"] == arm]
            ax.plot(
                [float(x["time"]) for x in r],
                [float(x["mid_u"]) for x in r],
                label=arm,
                linewidth=1,
            )
        ax.set(
            title=f"Seed 7 · {label} · midpoint module27 signed u",
            xlabel="Simulation time",
            ylabel="Signed displacement",
            yscale="symlog",
        )
        ax.set_yscale("symlog", linthresh=1e-6)
        ax.grid(alpha=0.2)
        ax.legend(ncol=4)
    fig.suptitle(
        "Frozen-medium interference; independent arm simulations\n"
        "Long-window timestep convergence screen FAILED (see results report)"
    )
    fig.savefig(args.output / "experiment_0004_midpoint.png", dpi=150)
    plt.close(fig)
    frames = {}
    for arm in range(4):
        for tick in TIMES:
            frames[arm, tick] = read_frame(
                args.folder / f"group-01/{arm}/frame-{tick:06}.q4f", False
            )
    norm = SymLogNorm(linthresh=1e-6, vmin=-1, vmax=1, base=10)
    cmap = "RdBu_r"
    chosen = (32, 160, 512, 1024)
    fig, axes = plt.subplots(4, 4, figsize=(10, 10), layout="constrained")
    for arm in range(4):
        for j, tick in enumerate(chosen):
            ax = axes[arm, j]
            image = ax.imshow(
                plane(frames[arm, tick]),
                origin="lower",
                extent=(-0.5, 15.5, -0.5, 15.5),
                cmap=cmap,
                norm=norm,
            )
            ax.scatter([6, 10], [8, 8], s=14, facecolors="none", edgecolors="black")
            if arm == 0:
                ax.set_title(f"t={tick * 0.1:g}")
            if j == 0:
                ax.set_ylabel(f"{ARMS[arm]} / Y")
            if arm == 3:
                ax.set_xlabel("X")
    fig.colorbar(
        image,
        ax=axes,
        shrink=0.65,
        label="Signed module27 u · fixed symmetric log scale ±1, linear core ±1e-6",
    )
    fig.suptitle(
        "Measured Z=8 plane · homogeneous seed7 · dt0.1\n"
        "Frozen palette; rings mark source sites. Long-time convergence screen failed."
    )
    fig.savefig(args.output / "experiment_0004_plane.png", dpi=150)
    plt.close(fig)
    if args.movie:
        args.movie.parent.mkdir(parents=True, exist_ok=True)
        fig, axes = plt.subplots(1, 4, figsize=(12, 4), layout="constrained")
        images = []
        for arm, ax in enumerate(axes):
            images.append(ax.imshow(plane(frames[arm, 0]), origin="lower", cmap=cmap, norm=norm))
            ax.scatter([6, 10], [8, 8], s=25, facecolors="none", edgecolors="black")
            ax.set(title=ARMS[arm], xlabel="X", ylabel="Y")
        fig.colorbar(
            images[0], ax=axes, shrink=0.65, label="Signed u27 · fixed symlog ±1 / core ±1e-6"
        )
        title = fig.suptitle("")
        writer = FFMpegWriter(
            fps=2, metadata={"title": "Experiment 0004 measured frozen-medium signed fields"}
        )
        with writer.saving(fig, str(args.movie), 100):
            for tick in TIMES:
                for arm, image in enumerate(images):
                    image.set_data(plane(frames[arm, tick]))
                title.set_text(
                    f"0004 · homogeneous seed7 · Z=8 · t={tick * 0.1:g} · dt0.1\n"
                    "Recorded checkpoints held 2s; unequal physical intervals. "
                    "Convergence screen FAILED."
                )
                for _ in range(4):
                    writer.grab_frame()
        plt.close(fig)


if __name__ == "__main__":
    main()
