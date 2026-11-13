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

## All-tick 3,072-tick replay

```sh
CARGO_BUILD_JOBS=1 cargo run --locked --manifest-path viewer/Cargo.toml -- --decay-0004 --recording results/experiment_0004-decay-a --select 2184
```

This separate extension has all ticks0..3072. Full cube, paused, is the default.
Space plays at the default20ticks/s; Period/Right advances one tick. Home/End
seek to0/3072; `--frame N` chooses an exact initial tick. Optional `--slice-z 8`
shows the central plane. `--replay-hz` changes wall-clock cadence, not dt.

Frames are indexed on disk, with bounded six-frame history. Playback advances
at most once per rendered application update, waiting for scene refresh; if
rendering is slow, playback slows without catching up by dropping ticks.
The header shows global max|u|/max|v| and instantaneous below1e-6 status. This
is distinct from the inspector's selected-neuron values and from the display
color floor1e-8. Repeated quiet-tail status is computed offline, not inferred
from a dark image. The original timestep-convergence failure remains relevant.

For a fresh automated native audit folder (create its parent first):

```sh
viewer/target/debug/qr-scientific-observatory --decay-0004 --recording results/experiment_0004-decay-a --select 2184 --play --replay-hz 60 --decay-audit results/my-decay-audit/audit.csv
.venv/bin/python -m analysis.decay_0004 results/experiment_0004-decay-a --audit-only results/my-decay-audit/audit.csv
```

The audit records each scene-update tick/hash and global/selected fields, captures
an endpoint PNG and exits after the final frame. Use it without manual seeking;
independent verification rejects gaps, duplicates and wrong selections/hashes.
Native audit is separate from monitor scanout or FPS measurement. LIVE, synthetic
and old witness modes are rejected. Corrupt frame reads stop with a visible error.

The analyzer exports a viewer-free `agent-report/REPORT.md` and `AGENT_REPORT.zip`
with metadata, rules, every-tick CSVs and all-neuron summaries. See the
[extension results](../docs/EXPERIMENT_0004_EXTENSION_RESULTS.md).

## Display spacing and Experiment 0005

Default node-center spacing is now **1.8**, with unchanged glyph size. Set
`--node-spacing 1.0` for the old spacing, or any finite value 1.0–3.0. During use,
`-` / `=` change spacing by 0.1; Backslash restores 1.8; R fits/resets the camera
at the current spacing. Zoom and pan remain available. These controls scale
node centers, picking, selected edges and axes together. Slices still use integer
coordinates. Graph hops, couplings, timestep, source distance and exported states
are unchanged; a wider screen lattice is not a physically more distant graph.
Camera fitting also accommodates Grid2D8 and Cubic26.

```sh
CARGO_BUILD_JOBS=1 cargo run --locked --manifest-path viewer/Cargo.toml -- --experiment-0005 --recording results/experiment_0005/a-heterogeneous-7-AB+ --node-spacing 1.8 --select 2184
```

0005 is **checkpoint REPLAY ONLY**, with its own validated QR5 files and identity;
LIVE, synthetic and old 0003-witness substitution are rejected. Space plays saved
checkpoints; Right/Left seek; the displayed time uses actual dt/tick and checkpoint
intervals are unequal. It is not an every-tick animation or a physics restart.
The same spacing controls work in 0003, 0004, and every-tick decay modes.

Optional `--capture results/capture.png` saves an actual native screenshot after
30 application updates, a sibling text receipt (tick, full observation checksum,
spacing, selected world position, camera radius, engine count), then exits at 60.
The receipt uses `QR-DISPLAY-RECEIPT-v1` FNV over scientific observation fields;
it excludes display spacing/camera. It is not a sustained-FPS measurement.

## Experiment 0006 morphology replay

`--experiment-0006 --recording CASE` reads QR6 checkpoints only. No live engine,
synthetic substitute or old witness mode is allowed. Source module9/27 and the
registered case ID are labeled; tick/time use actual recorded dt. Checkpoints
have unequal intervals. Spacing remains1.8 with the same -/=,Backslash,R controls.

`--region all|shell|interior` scripts spatial filtering without changing the graph.
The default overlay is **recorded final Pafter-Pbefore**, not wave amplitude:
gray0 changed modules,blue1,yellow2–3,red4+. M toggles this overlay to the usual
activity/palette view. Selection shows exact recorded BEFORE and FINAL AFTER
8×8 panels side by side, with change count,global mechanical energy and switch
work. Both panels use actual palette bytes, including when viewing earlier ticks;
the fixed final-difference overlay is not a claim that edits already occurred
at that earlier time. Numeric u27/v27 remain module27 fields even for module9
injection. Full signed source-module traces are in the scientific CSV.

```sh
CARGO_BUILD_JOBS=1 cargo run --locked --manifest-path viewer/Cargo.toml -- --experiment-0006 --recording results/experiment_0006/m-s7-m27-E1-R2-d0.025 --frame 5 --select 2182 --slice-z 8
```

`--capture` also records the selected before/after palette integrity checksums,
exact changed count,input module and overlay state. These are native instrument
receipts,not a substitute for independent scientific verification. A REUSED
ledger entry has no duplicate frames: open its explicitly named source trajectory.
