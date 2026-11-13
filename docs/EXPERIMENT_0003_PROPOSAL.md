<!-- Proposal archive; trailing whitespace normalized. Authorized implementation scope is registered in EXPERIMENT_0003.md. -->

# Experiment 0003 — Volumetric Lattice and Bevy Scientific Observatory

**Status:** PROPOSED SPECIFICATION — not implemented, measured, or committed
**Prepared:** 2026-11-12 (America/Toronto)
**Project:** QR Neural Memory System — Cognis 11 Inc.
**Intended location in project:** `EXPERIMENT_0003.md` at repository root

## 0. Objective and non-goals

Replace the *experimental network layout* (not the 4,096-neuron population or each neuron's internal structure) with a genuinely three-dimensional lattice, and build a working **Bevy-based scientific viewer** to observe real simulation state.

**Primary question:** How do topology, spatial boundaries, and coupling affect propagation, damping, resonance (if available from Experiment 0002), and collective vibrational behavior in a volumetric network of reconfigurable QR neurons?

**Experiment 0003 is primarily about topology, observation, and reproducible dynamics.** No associative-memory, intelligence, emergence, or resilience success is required or presumed. A visually compelling pattern is not evidence of a new computation. Changes to dynamics must be separately identified and measured.

Do not rewrite or retroactively tune Experiment 0001; keep its rules, results, tests, CLI entry point, and recorded evidence intact. Reuse the physics of Experiment 0002 where implemented and verified. If Experiment 0002 is still local, incomplete, or absent, build the lattice/viewer using real, instrumented Experiment 0001 dynamics or a clearly labeled, deterministic diagnostic fixture. Do not claim to visualize resonance unless the engine exposes a defined, measured resonance quantity.

## 1. Verified starting point (repository inspection on 2026-11-12)

- `EXPERIMENT_0001.md` and `src/experiment.rs` implement the 64 × 64 **two-dimensional** network and two-scale damped-wave candidate; 4,096 neurons each have a separate 8 × 8 module palette configuration, with transient channel amplitudes and previous amplitudes.
- Experiments use four channel values (red, green, blue, yellow) plus zero/empty persistent modules. The module arrangement is the neuron's shape.
- Inter-neuron coupling uses eight neighbors in the 2D grid; intra-neuron coupling uses four orthogonal module neighbors. Synchronous snapshot updates are implemented.
- The reported Experiment 0001 ripple run produced negligible usable output and no persistent configuration changes. Do **not** present the old failure as solved merely by rearranging nodes.
- `rust-toolchain.toml` pins Rust 1.99.1 at time of inspection. Root `Cargo.toml` has no Bevy dependency.
- Neither `EXPERIMENT_0002.md` nor `docs/EXPERIMENT_0002.md` was found on the repository's default branch at inspection. Local work may be ahead of GitHub: inspect `git status`, local files, and active Codex changes before editing.

Any newer local changes supersede this snapshot only after direct inspection, not assumption.

## 2. System architecture: three separate responsibilities

### A. Scientific engine (Rust, headless, authoritative)

- Holds neuron internal configurations, transient amplitudes, previous amplitudes/momentum, and any experimentally defined resonance states.
- Performs fixed-step, synchronous state evolution independently of wall-clock time, graphics frame rate, camera, UI, or display adapter.
- Accepts an explicit topology instance and parameters; computes neighbors through that topology rather than hard-coded `side` and 2D row-column arithmetic.
- Supports headless execution, recording, deterministic experiment fixtures, and unit tests without loading graphics libraries.

### B. Observation / recording boundary (read-only)

- Produces **versioned**, tick-numbered snapshots for the viewer and offline analysis.
- Includes simulation provenance: topology kind, dimensions, neighbor policy, boundary policy, physics parameters, seed, engine version/commit if available, and whether state is measured or synthetic.
- For each neuron, includes stable neuron ID, integer `(x,y,z)` position, persistent 8 × 8 palette modules, and available measured transient fields (e.g., four-channel amplitude and previous amplitude or a clearly defined derived magnitude).
- For any resonance metrics, include the metric definition and provenance. An attractive hue or oscillating animation is not a resonance measurement.
- Record exact step order and simulation tick. Playback reads snapshots; it must not rerun a different physics implementation behind the user's back.

### C. Bevy viewer (separate package/binary)

- Displays **recorded or live engine snapshots**, never serving as the source of physical state truth.
- UI commands such as pause, single-step, seek/replay, rotate, slice, inspect, and change color mapping operate through explicit commands or snapshot selection.
- No simulation state change should depend on rendering delta time, frame rate, lighting, interpolation, selection, or color scaling.
- Viewer-specific camera coordinates are not lattice topology. Visual spacing may change without modifying the graph.

**Packaging recommendation:** Keep Bevy in `viewer/` with its own manifest/lockfile or another isolated build boundary, rather than adding Bevy to the headless engine's core dependencies. Document the exact structure chosen; preserve the existing root severe CI. A separate viewer compilation job can be added when practical, but should not silently force GPU/window creation on headless CI.

## 3. Lattice geometry (registered starting conditions)

**Population:** 4,096 distinct neuron IDs
**Default 3D extent:** `16 × 16 × 16 = 4096`
**Cell geometry:** Each neuron retains an independent 8 × 8 internal colored-module configuration (64 modules per neuron). The number of internal modules does **not** change with network dimensionality.
**Index convention:** `id = x + 16 * (y + 16 * z)` for `0 <= x,y,z < 16`. Define inverse mapping and enforce round-trip tests.
**Boundary:** Open/nonperiodic; absent neighbors contribute no external link, not an implicit wraparound.

Implement at least these topology policies as **separate experimental conditions**:

1. **Cubic6 (primary baseline):** Six face-sharing neighbors, offset by one along exactly one Cartesian axis.
2. **Cubic26 (secondary comparison):** Every other node of the surrounding 3 × 3 × 3 cube, including face/edge/corner diagonals.
3. **Grid2D8 (preserved legacy condition):** Existing 64 × 64 eight-neighbor layout for direct regression testing and original Experiment 0001 replay. Do not automatically compare experimental outcomes as though geometry and spatial source–target distance were matched.

Expected corner degree in a nonwrapping 16³ lattice: **3** for Cubic6 and **7** for Cubic26. Interior node degree: **6** or **26** respectively. Verify degree distribution and reciprocal edges throughout the graph.

**Important control:** More neighbors changes total coupling unless normalized. Specify whether coupling is normalized by fixed maximum degree, actual degree, link length, or something else. For the primary diagnostic, preserve a documented global normalization policy and report its effect. Do not use a single unexplained scaling factor that makes one geometry appear superior.

For Cubic26, orthogonal, face-diagonal, and body-diagonal connections have different Euclidean lengths. Log their counts and test directional bias; a 26-neighbor cubic network is **not inherently isotropic**.

## 4. Internal vibrational quality and resonance (interface requirements)

Each QR neuron may have its own **configuration-dependent vibrational response**. Keep the following distinct:

- Persistent configuration: module palette and spatial shape, and any additional persistent parameters only if independently specified.
- Transient dynamics: amplitude, previous amplitude/velocity-like difference, and any phase or oscillator state introduced by an explicit update rule.
- Derived measurements: spectrum, effective frequency response, local coherence, coupling response, and resonance eligibility. These must be computed and labeled, not asserted from appearance.
- Plasticity: a separate experimental rule. Do not sneak a learning update into a transport-only condition.

If Experiment 0002 already specifies intrinsic frequencies, phase response, or plasticity, reuse that exact definition with provenance. Otherwise **do not invent frequency-based learning in Experiment 0003**. It is acceptable to expose existing four-channel wave dynamics, a controlled sinusoidal input fixture, and measured frequency-response plots as a first step.

### Required distinction

A coherent or persistent-looking visual oscillation does not establish resonance, attractor behavior, memory, or intelligence. A resonance claim needs a defined diagnostic, such as frequency-dependent response under amplitude/energy-matched forcing compared against off-resonance controls, with documented windowing, damping, and boundary handling.

## 5. Viewer: first working scientific instrument

### Initial executable vertical slice (must work before extra polish)

- Launch a Bevy window containing all **4,096 selectable lattice nodes** in three dimensions.
- Orbit, pan, zoom, and reset camera. Expose lattice X/Y/Z orientation axes.
- Show simulation tick, simulation state source (`LIVE`, `REPLAY`, or `SYNTHETIC TEST FIXTURE`), topology policy, and paused/running state.
- Select a neuron (ray picking or a coordinate/ID input). Show exact `(x,y,z)`, neuron ID, number of neighbors, and measured numeric fields.
- Show that neuron's **actual 8 × 8 internal color-and-shape configuration** in a secondary inspection panel.
- Show a time-varying color overlay based on a **named and numerically defined** field, such as maximum absolute four-channel amplitude or per-channel selected amplitude, with an explicit legend and scale.
- Pause, single-step, and replay a bounded timeline of recorded snapshots; seeking to the same tick must reproduce the same displayed values.
- Filter by X/Y/Z slices or shell/interior. Enable node, edge, and activity overlays independently. Reduce edge rendering at large counts (selected-node connections or a bounded neighborhood first).

### Visualization safeguards

- A node's **persistent palette color** and the **false-color activity overlay** are two different data views. Provide a clear switch or side-by-side legend; never silently substitute one for the other.
- Use fixed or documented color normalization for comparative playback. If auto-scaling is offered, show it prominently because it can make tiny noise look like strong activity.
- Nonfinite values must be visibly flagged and recorded, not rendered as arbitrary colors.
- Do not animate neuron geometry, sizes, or brightness in a way that implies measured vibration unless the effect is explicitly identified as display-only exaggeration.
- For efficiency, render one compact glyph per neuron initially. **Do not instantiate all 262,144 internal module cubes** in the first view; inspect the selected neuron's modules on demand or use aggregated textures/instancing.
- Make the instrument practical on the user's Linux/X11 workstation and document tested graphics backend and GPU. Do not assume the compute GPU is the display adapter.

## 6. Controlled dynamics and falsification-oriented probes

With plasticity **disabled**, run registered deterministic probes under Cubic6 and Cubic26:

**T1 — Single impulse:** Inject a known single-neuron / single-module impulse at the volumetric center. Record first-arrival tick, peak amplitude, and post-stimulus decay as a function of topological hop distance and Euclidean distance.

**T2 — Boundary and direction:** Repeat the identical impulse at a corner and at selected face/edge/interior sites. Compare ±X, ±Y, ±Z and diagonal directions where present. Mark unreachable sites and signal extinction explicitly.

**T3 — Forced vibration:** Apply a defined sinusoidal or periodic input only if the engine actually supports it; preregister frequencies, amplitudes, sampling window, and which four-channel inputs are excited. Report peak and frequency-selective response, phase/coherence if measured, and equal-energy/off-frequency controls.

**T4 — Configuration perturbation:** Run the same impulse with unchanged vs a known, explicitly seeded pattern of 8 × 8 persistent palette configurations. Record whether transport or response changes. This tests configuration dependence, **not learning**.

**T5 — View independence:** Repeat the same headless run with viewer disabled, viewer live, different camera positions, paused/single-stepped UI playback, and different graphics frame-rate caps. The engine's tick-indexed scientific snapshots/metrics must agree for equivalent commanded steps. Wall-clock performance may vary.

**T6 — Replay correctness:** Record a short run; load it without the engine executing; compare displayed per-neuron fields and timeline order to original snapshots. Include a deliberately corrupted file to confirm graceful error handling.

For topological comparisons, keep the impulse definition, input energy, source position type, and physical parameters explicit. Do not claim a fair 2D-vs-3D performance superiority unless physical distance, graph distance, source distribution, boundary population, and coupling budgets are accounted for.

### Measurements to export

At minimum: run parameters; lattice topology and degree distribution; waveform configuration; per-tick global and spatially localized amplitude peaks; sum of squared amplitudes (diagnostic proxy, **not physical energy**); nonfinite counts; first-arrival/threshold crossing by target shell; directional differences; persistent configuration checksums; snapshot schema version; selected-neuron time series. Report raw values and plots separately.

No learned-association or resilience claim is made or tested here.

## 7. Software tests and scientific acceptance gates

### Gate A — Geometry correctness

- Exactly 4,096 unique coordinates/IDs; reversible coordinate mapping; independent neuron state ownership.
- Cubic6 and Cubic26 produce reciprocal, unique, in-bounds neighbor sets; no self-links, duplicates, or wraparound.
- Exact expected corner/interior degrees; edge and face degrees checked; controlled neighbor iteration order.
- Every transport edge respects explicit topology. The legacy Grid2D8 path remains unchanged.

### Gate B — Numerical and causal correctness

- Per-step updates use old snapshots, not traversal order; reverse traversal gives matching scientific output under fixed conditions.
- No signal reaches a node farther than the available number of graph edges in fewer than that many updates, assuming no nonlocal injection.
- Transient reset clears all transient fields while retaining persistent configuration.
- A fixed finite-run stability diagnostic records peaks and nonfinite values for all conditions. A pass is **not** a stability proof at every timescale.
- Explicit invariants compare viewer-on vs headless engine results; legacy 0001 regressions remain runnable and unchanged.

### Gate C — Viewer validity

- Viewer launches on the target workstation and shows 4,096 distinctly addressable nodes in a rotatable 3D volume.
- Neuron selection is accurate; the selected 8 × 8 module panel agrees byte-for-byte with recorded state for sample IDs and ticks.
- Slice filters, pause/single-step, numeric legend, tick metadata, and replay operate correctly.
- A saved playback reproduces exact values even if visual interpolation is enabled.
- An explicitly synthetic diagnostic looks different in labeling from measured engine output. No decorative demo may be presented as a discovery.

### Gate D — Reproducibility / delivery

- Two identical headless runs with the same toolchain and registered parameters produce matching raw evidence, or any observed divergence is diagnosed and disclosed.
- The existing root formatting, Clippy, tests, doc tests, release build, Rustdoc, and Python analyzer checks pass; viewer-specific build/tests documented separately.
- Store large traces, screenshots, movie recordings, and caches outside Git under ignored directories unless small examples are deliberately curated and licensed for the repository.
- `docs/EXPERIMENT_0003_RESULTS.md` reports **what actually ran** with dates, hardware/backend where relevant, observed behavior, negative results, and known limitations.
- Update `HANDOFF.md` and `README.md`; inspect staged diff and commit status before delivery. Do not claim hosted CI or remote push without checking.

## 8. Implementation order and stop conditions

1. Read the actual local repo, inspect worktree and ongoing Experiment 0002 changes. Do not overwrite Codex's work in progress.
2. Write and review the final fixed Experiment 0003 condition matrix/parameters **before** running measurements. Keep this proposal separate from the final preregistration.
3. Extract a topology interface; add and test `Grid2D8`, `Cubic6`, and `Cubic26`. Preserve the legacy experiment path.
4. Build a headless 16³ test runner and a small stable, versioned observation format with one deterministic fixture. Confirm finite-speed propagation, boundary behavior and replay hash consistency.
5. Create the isolated Bevy viewer; show 4,096 nodes, camera, selection, 8 × 8 internal panel, tick and legend; load the fixture.
6. Add live snapshot adapter only after saved replay functions. Add slicing, selected-neuron time series, and bounded activity visualization.
7. Run T1–T6 on the registered conditions. Archive both successes and failures; keep any change to physics in a newly labeled experimental arm.
8. Run software checks, update results and handoff, produce a usable operator command and short controls guide.

**Stop or narrow scope if:** the 3D topology changes legacy 0001 results; viewer affects the scientific state; graphics/toolchain compatibility blocks independent engine tests; snapshot format cannot be round-tripped; or numeric instability occurs. Report the blocker and preserve verified working code rather than faking an apparently completed visualization.

## 9. Explicitly deferred

- Claims of associative memory, intelligence, emergence, or fault tolerance.
- GPU simulation or parallel scientific updates.
- Physically accurate water/fluid mechanics.
- Automatic parameter optimization to make patterns look interesting.
- Visualizing every internal module in the entire lattice simultaneously.
- Full 3D plasticity, topology evolution, or dynamic edge rewiring unless explicitly implemented in a separate registered arm.
- Unbounded recording or production-grade UI polish.

## 10. Suggested integration points (targets, not existing verified symbols)

Proposed modules, to be confirmed against the working tree: `src/topology.rs`, a `src/lattice.rs` adapter or clean extension of the existing engine, `src/observation.rs`, `src/bin/experiment_0003.rs`, and an isolated `viewer/` package. These names are **design suggestions**, not claims that those files or public APIs exist.

**Dependency guidance as of 2026-11-12:** Bevy 0.20 was officially released on 2026-10-08, and 0.20.1 is a published patch. Start from the official Bevy examples/docs for the specific version used; confirm the version builds with this repository's pinned Rust toolchain and the user's Linux graphics configuration before locking it in. Do not rely on old Bevy APIs from game projects without inspecting migration requirements.

- Official Bevy release: https://bevy.org/news/bevy-0-20/
- Official 3D gizmos example: https://bevy.org/examples/gizmos/3d-gizmos/
- Official Bevy example index: https://bevy.org/examples/
- Current project baseline: `EXPERIMENT_0001.md`, `docs/EXPERIMENT_0001_RESULTS.md`, `AGENTS.md`, `HANDOFF.md`

## 11. One-sentence Codex mandate

> Implement Experiment 0003 as a **headless deterministic 16 × 16 × 16 QR-neuron lattice with documented Cubic6/Cubic26 topology comparisons, plus a separate Bevy scientific viewer driven entirely by observed, replayable states**. Preserve Experiments 0001/0002 and do not conflate graphics, resonance hypotheses, or successful software checks with discovered neural behavior.
