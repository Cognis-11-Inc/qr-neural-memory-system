# QR scientific observatory — operator guide

The Bevy viewer observes the CPU engine or saved data. It is not a second physics
implementation. Default graphics preference is **low power**, verified on this
workstation's AMD Raphael iGPU; the 5060 Ti can remain available for models.

Run all commands from the project root. First generate the bounded recording:

```sh
cargo run --locked --release --bin experiment_0003 -- results/experiment_0003-a
```

The runner refuses an existing folder. Then launch a **paused replay**:

```sh
CARGO_BUILD_JOBS=1 cargo run --locked --manifest-path viewer/Cargo.toml -- --recording results/experiment_0003-a/case-0000
```

For live CPU stepping of the same fixture, add `--live`. Cubic26's corresponding
fixture is `case-0055`; the Grid2D8 regression recording is `case-0110` (zoom out
for its larger planar extent). All contain 4,096 stable neuron IDs.

## Controls

| Control | Behavior |
|---|---|
| Right mouse drag | Orbit |
| Middle mouse drag | Pan |
| Wheel | Zoom |
| R | Reset camera |
| Click glyph | Select nearest visible glyph along cursor ray |
| Type ID, then Enter | Inspect any ID 0–4095, including hidden nodes |
| Up / Down | Next / previous selected ID |
| Space | Play / pause |
| Period | Single engine tick in LIVE; one recorded frame in REPLAY |
| Left / Right | Seek recorded frames; switches to REPLAY |
| Home / End | First / last recorded frame; switches to REPLAY |
| L | Switch REPLAY to a fresh LIVE fixture at tick 0, or LIVE to REPLAY |
| X / Y / Z | Toggle an integer-coordinate slice axis |
| [ / ] | Change slice plane |
| F | All / shell / interior display filter |
| N | Node visibility |
| E | Selected-node physical-neighbor edges |
| A | Activity false-color overlay |
| P | Persistent-palette summary glyph view |
| G | Fixed linear / logarithmic activity mapping |
| Escape | Exit |

**Display filters do not disable neurons, change coupling, or create lesions.**
The inspector remains usable when its selected node is hidden. Edge overlays
show actual neighbors, including those hidden by display filters. Selection
highlight and visual spacing are presentation, not measured deformation.

## Read the instrument correctly

- Header distinguishes `LIVE`, `REPLAY`, and `SYNTHETIC TEST FIXTURE`.
- Tick is authoritative; time is tick × 0.1 in nondimensional units. Normal play
  dispatches at 20 Hz; this is not a physical real-time fluid simulation. An Experiment 0003
  recorded frame advances eight ticks. Replay only looks up observations.
- LIVE advances the same headless Rust engine. Camera, render delta, frame caps,
  selection and colors never enter its update. At tick 160 the fixture stops.
- Seeking drops the live engine. L explicitly restarts from the fixture's seed;
  it does not pretend aggregate observations are full restart checkpoints.
- Activity is max absolute displacement across modules/channels. Linear scale
  is fixed 0–1. Log scale is fixed log10(1e-8)–log10(1), clamped at endpoints.
  There is no hidden auto-gain. Exact numbers remain in the inspector/recording.
- Persistent glyph color is the most frequent module palette value. The 8 × 8
  panel shows **all actual module values**, separately from activity false color.
- Peak fields aggregate 64 modules; `u27`/`v27` are signed values at internal
  row 3, column 3. The bounded history shows measured max amplitude by tick.
- Nonfinite input is flagged magenta and counted. Color, persistence of a
  visual pattern, or a large response is not a resonance/learning measurement.

## Graphics selection and bounded witness mode

The default low-power preference is portable. Explicit `--adapter SUBSTRING`,
`WGPU_ADAPTER_NAME`, or `WGPU_POWER_PREF` can override adapter choice. The
startup receipt prints the actual adapter/backend. Do not assume which GPU won.

```sh
CARGO_BUILD_JOBS=1 cargo run --locked --manifest-path viewer/Cargo.toml -- --adapter "AMD Ryzen" --recording results/experiment_0003-a/case-0000
```

`--fps 30` or `--fps 60` sets a rendering cap, not a guaranteed measured rate.
`--camera-b` supplies an alternate camera pose. Neither changes scientific state.
For automated **real-window** validation, use a fresh output folder:

```sh
viewer/target/debug/qr-scientific-observatory --recording results/experiment_0003-a/case-0000 --live --witness results/my-0003-witness --fps 60
.venv/bin/python -m analysis.experiment_0003 results/my-0003-witness --recording results/experiment_0003-a/case-0000
```

Witness mode audits three selected IDs over 21 ticks, samples pause/step and
normal play, changes camera and filters, repeats seeks, captures PNGs, records
the actual renderer, then exits. This is automated validation, not human testing.
`--synthetic-label-test` explicitly copies the display fixture, marks synthetic
provenance and injects a nonfinite **flag** for node 0. It is an instrument-label
test, not observed engine instability; it cannot enter LIVE mode. The analyzer
accepts the same flag to independently verify its expected copied observation.

Corrupt/schema-incompatible inputs are rejected before engine/window startup.
Recordings are limited to 256 frames and 2 MB per file; registered runs use 21.
Raw screenshots, witness logs, and recordings stay ignored under `results/`.

## Isolated quality checks

Bevy has its own manifest/lockfile and target directory. Root scientific checks
remain graphics-free. Compilation is single-job to bound memory use:

```sh
cargo fmt --manifest-path viewer/Cargo.toml -- --check
CARGO_BUILD_JOBS=1 cargo clippy --locked --manifest-path viewer/Cargo.toml --all-targets -- -D warnings
CARGO_BUILD_JOBS=1 cargo test --locked --manifest-path viewer/Cargo.toml
```

The CI viewer job compiles/tests without opening a window. Native rendering,
adapter selection and UI witness results are separately documented in the
[Experiment 0003 report](../docs/EXPERIMENT_0003_RESULTS.md).

## Experiment 0004: frozen-medium dual-source replay

Generate the separate suite using the root `experiment_0004` runner. Then:

```sh
CARGO_BUILD_JOBS=1 cargo run --locked --manifest-path viewer/Cargo.toml -- --experiment-0004 --recording results/experiment_0004-a/group-01/2 --select 2184 --slice-z 8 --play --replay-hz 0.5
```

This shows homogeneous seed7, same-polarity dual sources. Arm folders `0`, `1`,
`2`, `3` mean A, B, AB+, AB−. `group-00` is heterogeneous seed7;
`group-06/07` are the half-step seed7 configurations. Header lists both sources,
arm, timestep and frozen medium. The actual palette stays separately visible.
**0004 is replay-only**: `--live`, synthetic fixtures and the 0003 witness are
rejected; L leaves replay unchanged. No single-source substitute engine starts.

0004's twelve checkpoints have unequal simulation-time intervals. Right/Period
moves to the next recorded checkpoint; `--replay-hz` sets presentation cadence,
not simulation dt. Time remains exact tick × recorded dt. `--frame N` selects
an initial recording index (0–11 here); `--select ID` and `--slice-z Z` initialize
inspection without desktop keyboard focus. `--play` starts playback; default is
paused. Alternate cadence is restricted to replay. Ordinary 0003 defaults remain.

Activity overlay is still max-module magnitude; inspect signed module27 values
or the offline signed-plane plots to assess reinforcement/cancellation. A native
magnitude image cannot itself establish superposition. See the
[0004 measured report](../docs/EXPERIMENT_0004_RESULTS.md): superposition passed,
but **all eight long-window timestep screens failed**. No persistent memory
trace is claimed. `QR4FLD01` records full active-channel u/v with explicit 0004
provenance, rather than encoding dual-source data as 0003 snapshots.
