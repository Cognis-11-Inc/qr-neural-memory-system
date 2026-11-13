# QR Neural Memory System — handoff

Updated: 2026-11-13 (America/Toronto).

## Rehydrate

Read `AGENTS.md`, `README.md`, `PRINCIPLES.md`, `HYPOTHESES.md`,
`EXPERIMENT_0001.md`, and `docs/EXPERIMENT_0001_RESULTS.md`, then inspect the
current worktree and recent commits. For the latest scope also read
`EXPERIMENT_0002.md`, `docs/EXPERIMENT_0002_RESULTS.md`, and
`docs/EXPERIMENTS.md`. The latest delivery additionally requires
`EXPERIMENT_0003.md`, `docs/EXPERIMENT_0003_RESULTS.md`, and `viewer/README.md`.
For the current delivery read `EXPERIMENT_0004.md` and
`docs/EXPERIMENT_0004_RESULTS.md`. The reports distinguish software checks,
measured experiment failures, replay, and hosted delivery evidence.

## Project and user requirements

- Local directory: `/home/cognis/Cognis_Lab/QR neural memory system`.
- Public repository: https://github.com/Cognis-11-Inc/qr-neural-memory-system
- Owner: Cognis 11 Inc. (`Cognis-11-Inc`); branch `main`; HTTPS `origin`.
- Target approximately 80% Rust / 20% Python, flexible rather than a line quota.
- 4,096 QR neurons, **each with an independently mutable internal spatial
  color-and-shape configuration**. Rust owns the deterministic engine.
- Principle 001: associations must be investigated as persistent,
  experience-dependent changes distributed across neurons. No neuron owns a
  complete association. Retrieval must emerge from explicit local interactions.
- R1 distributed resilience remains untested. Graceful degradation is a target;
  no lesion experiment follows until intact retrieval works. The proposed R1
  protocol is `EXPERIMENT_R1.md`.
- Research reading list: `docs/FOUNDATIONAL_PAPERS.md`, imported unchanged from
  Downloads. Bibliographic claims were not verified during import.

## Latest delivery: Experiment 0004

User requested extended propagation, dual waves and superposition through a
frozen lattice; persistent changes are explicitly deferred. Registration frozen
in `4a2283a` before implementation/measurements, no post-result tuning.

- New separate `src/propagation_0004.rs`, `src/experiment_0004.rs`, CLI
  `experiment_0004`; old scientific sources unchanged. Only `src/lib.rs` exports
  new modules. Scalar active-channel state uses the exact 0002/0003 coefficients;
  all 4,096 independent palettes remain frozen. Cubic6 only, dt0.1,1024steps,
  time102.4; half-step controls dt0.05,2048steps.
- A2182=(6,8,8), B2186=(10,8,8), module27/channel1. Midpoint2184.
  Independently integrated A/B/AB+/AB−; 6 primary groups, 2 half-step groups,
  2 zero controls and old-engine regression =35 executions/suite.
- `results/experiment_0004-final-a` and `-final-b` compare byte-for-byte identical, including
  independent analysis. Each has 408 full signed frames. Every frame was
  checksum-checked independently and a SHA-256 ledger exported:
  `f0b05245646526228f6a1ad9f8972d883a9e260e20b7cbe565efdf03cc5ac172`.
  Every-step full-field checks are Rust measurements; independent full-state
  checks are at recorded checkpoints. Earlier `experiment_0004-a`/`-b` are
  retained: they lacked the registered time16 per-neuron cumulative peak export.
  Final suites add `peak_at_time16`, with unchanged equations/conditions/criteria;
  full signed frame ledgers match the earlier runs. Implementation `f4e97f5`.
- **Superposition passes** registered tolerance: max u/v residual
  1.4432899320127035e-15. Midpoint overlap appears at times0.3–0.5 in primaries.
  Both reinforcement and cancellation depend on local signed phase. Same-sign
  initial sources can cancel after heterogeneous propagation. No persistent change.
- **Long-window timestep screen FAILS 0/8:** worst relative combined-field L2
  errors27.76–30.42%, all worst at time102.4. Boundedness, pretty patterns and
  roundoff-level linearity do not establish accurate long-time trajectories.
  All numerical boundedness screens pass; no nonfinite fields; zero controls
  exactly zero. Whole-observation old 0003 regression matches at all161steps.
- Extended cumulative reach grows, e.g. homogeneous AB+755→4011 threshold-crossing
  neurons from time16→102.4. This is cumulative threshold reach, not simultaneous
  activity, capacity, usable recall or evidence of memory/resilience.
- `analysis/experiment_0004.py` verifies full signed fields/provenance/coverage,
  residuals/cross terms, all-tick metrics, spatial arrival consistency and dt
  sensitivity. Optional `analysis/plot_experiment_0004.py` makes actual signed
  figures and movie with fixed symmetric log ±1/core±1e-6; no per-frame gain.
- New `QR4FLD01` format records full scalar u/v plus actual palettes/metadata,
  separate from 0003's aggregate-only schema. Independent exact decode/checksum,
  corruption and mixed-provenance guards. Files ~4.46MB, reader ≤12 frames/arm.
- Shared viewer `--experiment-0004` is **REPLAY ONLY**, labels arm/both sources,
  frozen medium, dt, tick/time. --live/synthetic/0003-witness rejected; L cannot
  start the old engine. New startup --select/--slice-z/--frame/--play/--replay-hz
  make views reproducible without desktop focus; default old controls preserved.
  Unequal 0004 checkpoint intervals must not be represented as constant dt.
- Native Raphael actual replay captured in `results/0004-native/`, including
  26-second `experiment-0004-native-minus.mp4`. The 24-second offline signed-field
  movie is `results/experiment_0004-signed-plane.mp4`. It is a plot animation,
  distinct from native screen capture. Actual adapter: RADV RAPHAEL_MENDOCINO,
  IntegratedGpu/Vulkan/radv Mesa26.2.6-arch1.1, X11 on Wayland/XWayland.
  Desktop key injection was unreliable; startup controls drove verified native
  selection/slice/playback. No claim of exhaustive manual testing/FPS benchmark.
- Local gates:23 Rust tests,12 Python tests,1 viewer test; strict lints, format,
  root release/docs/doctests, Python compilation, native viewer build. All29
  protected old experiment files retain original SHA values; `src/lib.rs` only
  has new exports. Implementation `f4e97f5961b913aa0f7e14aa4e717ff693f4ff02`
  was pushed and [CI43380113097](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/runs/43380113097)
  passed all four jobs. Final result documentation and the replay-only help
  label follow separately; check their commit-specific CI on resumption.

Next bounded research step: **new** registration for finer integration/window
sensitivity. Persistent distributed change from wave interaction is unimplemented
and remains a later experiment. Preserve 0001–0004 rules/results; do not tune
retrospectively. R1 and learned retrieval remain untested/undemonstrated.

## Previous delivery: Experiment 0003

User authorized the full volumetric lattice / Bevy observatory proposal while
preserving both prior experiments. Registration `5a112c14033f9a4a825b290b606d9ed1ca692a2d`
was frozen at 2026-11-13 03:34:26 UTC, before measurements; implementation
checkpoint `e9d9dcaea9ccbb168e3817be3af7706963a1ec73` is pushed.

- New headless modules: `src/topology.rs`, `src/lattice.rs`, `src/observation.rs`,
  `src/experiment_0003.rs`; separate CLI `experiment_0003`.
- 16³ Cubic6 / Cubic26, 64²×1 Grid2D8 adapter; fixed maximum-degree coupling
  divisors 6 / 26 / 8, open boundaries, ascending neighbor order. Exact 0002
  oscillator, dt=0.1, 160 ticks; four stored independent channels, only channel 1
  driven. All 4,096 neurons own independent 64-module palettes; no plasticity.
- Two complete 111-case suites in `results/experiment_0003-a` and `-b` compare
  byte-for-byte identical including analysis. Each has 2,331 frames, zero
  numerical-screen failures, zero nonfinite values, zero persistent changes.
  Raw ledger SHA-256:
  `f6279047dd4015507a7851547bb82471b7a91cfc29c6999c7c530d0b07885ed1`.
  Core checksum readback covers all frames; independent Python checks all fields
  and separately recomputes checksums on 241 frames. See report for scope.
- Center impulse reach (threshold 1e-6): Cubic6 303–467 neurons / 6–7 hops;
  Cubic26 681–1153 / 5–6 hops. Corners reach fewer sites; graph and Euclidean
  distances and link-length distributions differ. No topology-superiority claim.
  Two forced-frequency probes are not a converged resonance sweep.
- Grid2D8 adapter matched original 0002 u/v bits at 161 source ticks and 861
  external-probe snapshot pairs (`results/0003-legacy-regression.json`). All 12
  protected old source/spec/report/analyzer files passed original SHA receipts.
- `QR3OBS01` schema 1 records derived fields and palettes, not full module u/v
  restart checkpoints. Replay reads frames without a physics engine; seek drops
  live state. L explicitly restarts a fresh seeded live fixture at tick 0.
- Separate `viewer/` Bevy package/lockfile; root remains graphics-free. Build
  single-job (`CARGO_BUILD_JOBS=1`). Default LowPower graphics preference follows
  user's practice: **Raphael iGPU for visuals, RTX 5060 Ti reserved for models**.
  Override explicitly if needed; startup logs actual adapter/backend.
- Native default adapter verified: AMD Ryzen 9 7950X / RADV RAPHAEL_MENDOCINO,
  IntegratedGpu, Vulkan, radv Mesa 26.2.6-arch1.1; X11 window on Wayland/XWayland.
  Earlier NVIDIA witnesses retained. No sustained FPS benchmark; 30/60 are caps.
- Real-window LIVE/REPLAY witnesses at both caps, alternate camera, selection,
  filters, UI bindings and repeated seeks matched headless bytes/fields.
  Final receipts: `results/0003-igpu-live-60-final`, `0003-igpu-replay-30-final`,
  `0003-igpu-c26-live-final`; each has 67 audited rows, 21 ticks, IDs 0/2184/4095.
  Cubic26 replay and synthetic labeling also checked. Corrupted recording exits
  1 before window/engine startup with checksum error. Synthetic fixture injects
  a labeled nonfinite flag, not observed instability. Automated validation is
  distinct from exhaustive human mouse/keyboard testing.
- Actual iGPU captures and measured shell plot are curated under `docs/figures/`;
  raw recordings, witnesses and screenshots stay ignored. Launch/controls:
  `viewer/README.md`. Normal launch starts paused in REPLAY.
- Local gates: 21 root Rust tests, 10 Python tests, one viewer test, strict
  Clippy/Ruff, formatting, root release/rustdoc/doctests, Python compilation,
  native viewer build, independent analysis and old-file preservation.
  New isolated headless viewer CI job supplements original three jobs.
  Implementation [hosted CI 43364632227](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/runs/43364632227)
  passed all four jobs (root Rust, Python 3.12/3.14, isolated Bevy viewer).
  The measured-results commit follows separately; check its own CI on resumption.

No new learning, resonance, memory, resilience or emergence success. 0002 T
remains negative and R indeterminate. Next bounded research step is a separately
registered convergence/transport investigation before Stage P. Do not tune frozen
0001/0002/0003 after seeing results. Viewer slices are not neuron lesions.

## Previous delivery: Experiment 0002 T/R

The user explicitly authorized **execute Stages T and R, preserve Experiment
0001 unchanged**, and requested clean progression. The separate proposal archive
is `docs/EXPERIMENT_0002_PROPOSAL.md`; trailing whitespace was normalized.
Actual fixed design: `EXPERIMENT_0002.md`. Its draft clock is 16:32:35 UTC and
Git freeze checkpoint is `f3167f19a2504c24e53fddc76a35423bf8ecc18d` at
2026-11-12 16:38:24 UTC, before implementation or trial collection.

New engine/runner: `src/experiment_0002.rs`, `src/bin/experiment_0002.rs`.
Independent verifier: `analysis/experiment_0002.py`. Optional figures:
`analysis/plot_experiment_0002.py`, pinned `requirements-plot.txt`.
The engine has no new Rust dependencies; verification is Python standard-library
only. Scalar modules, isolated 8 × 8 neurons, nine-neuron patches, and limited
full-network spectral checks are distinct fixtures. Full T networks contain
4,096 neurons, each with independent internal spatial color-and-shape state.
Intrinsic frequency/damping and coupling derive from configuration. Four
independent channels use synchronous f64 displacement/velocity updates.

**Measured outcome:** 73 T + 1,028 R cases per run; both complete runs at
`results/experiment_0002-a` and `results/experiment_0002-b` compare byte-for-byte
identical, including independent analysis. Raw evidence remains ignored by Git.
Every exported probe metric/phase, final-state diagnostic, input energy,
coverage, and before/after persistent snapshot was independently checked.

- **T negative** at the registered four-hop usability threshold: 96/96 required
  probes cross 1e-6 absolute amplitude, but 0/96 reach RMS/source RMS >=1e-3.
  The strongest four-hop ratio is 0.0002332. Nearer two-hop transmission meets
  these thresholds descriptively; this does not replace the registered verdict.
- **R indeterminate**: 28/36 isolated sweeps meet the peak criterion; 11/12
  same-histogram shape pairs differ by >5%. Four isolated complex-transfer
  timestep comparisons fail the 10% limit (errors 10.28%–12.26%). All peak bins
  agree between steps; 3/3 limited full-network comparisons pass the 15% bound.
  Nine shape pairs meet sensitivity and both paired numerical-validity checks.
- All 1,101 cases satisfy every-step amplitude/velocity boundedness screens;
  no nonfinite observations. Both zero-drive cases remain exactly zero. Maximum
  input-energy error is 4.22e-15. All persistent structures remain unchanged.
- Doubling damping lowers the sampled peak gain 4.98265 → 3.11663. Equal-spectrum,
  equal-energy multisine phase controls were verified; they establish no learned
  coherence or spatial synchronization.
- **Stage P not run**: no plasticity, no eligibility traces, no experience-induced
  persistent reconfiguration, no memory/resilience result or novelty claim.

Raw Rust summary SHA-256:
`cd67cccbe6c380dd0384e2b0d5751ef4137f36c62eab6de9f91081acc3c9598d`.
Figures and a complete account of failures/indeterminacy are in
`docs/EXPERIMENT_0002_RESULTS.md`. Raw internal probe `distance` records graph
hops; verified tables derive true internal Euclidean distance explicitly.

Experiment 0001 engine, runner, analyzer, tests, specification, and results
report passed original SHA-256 preservation checks and have an empty Git diff.
Its old raw evidence was read only to copy the historical impulse CSV unchanged;
that reference is not matched forcing and implies no comparative superiority.

Final local gates passed: 16 Rust tests, 8 Python tests, formatting, strict
Clippy/Ruff, release build, doctest execution and rustdoc with warnings denied,
Python compilation, documentation/figure links, independent numerical
verification, complete-run byte comparison and Experiment 0001 preservation.
Both report figures were rendered and inspected. Implementation checkpoint is
`8a0f5b4` following frozen registration `f3167f1`; measured results are recorded
in a separate documentation commit. Delivery `3ca5f25722eb6643e2275178d4cabf925f50d36b`
was pushed and remote `main` matched. [Hosted CI 43264716126](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/runs/43264716126)
passed all three jobs (Rust, Python 3.12, Python 3.14). This verification note is
documentation only; check CI for later commits separately. Software CI does not
change the negative T or indeterminate R scientific outcome.

Next falsification: separately register finer integration/frequency resolution,
longer-window sensitivity and a coupling/damping transport grid. Do not tune
Experiment 0002 retrospectively. Stage P remains deferred and separately
registered/authorized. No change to Experiment 0001 is needed.

## Experiment 0001 task and design decisions

The user requested execution of `/home/cognis/Downloads/EXPERIMENT_0001_CODEX_BRIEF.md`:
formalize rules first, then implement the smallest testable Rust experiment,
run checks, document actual results and failures, update this handoff, and make
no unsupported learning/resilience claim. The brief was read in full.

The user subsequently introduced ripple dynamics: local disturbances within and
between QR neurons; transient amplitude/momentum distinct from persistent
color-and-shape configuration; experience changes the medium. This was added to
the specification **before code or measurements**, retaining a simpler local
baseline. No external literature claims were needed for the candidate equations.

Earlier the user removed an assumed 64 × 64 geometry. The newly supplied brief
explicitly specifies a 64 × 64 network, adopted **for Experiment 0001 only**.
Each neuron's separate internal matrix is 8 × 8, palette 0–4. Eight nonwrapping
external neighbors and four orthogonal internal-module neighbors are explicit
experiment parameters; the neuron count itself implies no geometry.

## Experiment 0001 implementation

- `src/experiment.rs`: per-neuron persistent module arrays, transient local
  signals or four-channel current/previous amplitudes, snapshot-based synchronous
  updates, local color-transition plasticity, local baseline and damped-wave
  candidate, sequential training, incomplete-cue recall, controls/ablations,
  exported evidence and fixed scientific decisions. No external dependencies.
- `src/bin/experiment_0001.rs`: runner; refuses an existing output directory.
- `analysis/experiment_0001.py`: independently recomputes colored-output metrics
  and verifies snapshots/change counts, locations, trial coverage and verdicts.
- Original activation-bit helpers remain as legacy scaffolding in `src/lib.rs`;
  the experiment does not use those helpers as its neuron representation.
- Inference receives only the cue and substrate, never a teaching target or
  pair table. Targets are used by the evaluator after inference. Reset clears
  both amplitude and momentum while preserving internal module configurations.
- Snapshots export learned arrays for inspection; importing/restarting an engine
  from snapshots is not implemented or claimed.

## Experiment 0001 actual results

**No successful associative learning, sequential retention, distributed causal
memory, or resilience has been demonstrated.** All 24 registered seed × arm ×
variant trials failed the association/retention criteria; 168 inferences per run.

Local normal A→B errors (seeds 7, 19, 41): 0.831055, 0.783203, 0.789062.
This rule changed 16 taught output neurons, with 741/769/759 modules changed
from initialization. Changes are localized to rows 30–33, columns 60–63; the
other 4,080 sites are relays under this rule. C→D modifies the same 16 sites.
Multiple changed neurons are evidence of mutation, not working memory.

Ripple output was entirely empty; no persistent modules changed in the exported
full-distance trials. Final output-region amplitudes were about 2.4–3.1e-124,
far below the fixed 1e-12 readout threshold. Controls and trained conditions
match. Lower palette error than the local rule reflects empty target modules,
not better retrieval. The 160-cycle unforced impulse screen had peak 5.234436
and zero nonfinite observations; this is not a general stability proof.

Raw local evidence (ignored by Git):

- `results/experiment_0001-run-01`: first complete run, before additional
  output-region/all-cycle-stability instrumentation; scientific outputs unchanged.
- `results/experiment_0001-final-a` and `results/experiment_0001-final-b`:
  completed full runs with final instrumentation, byte-identical directories
  including independently generated analysis summaries. Each includes exact
  sources, specification, parameters, original stimuli, raw outputs, snapshots,
  metrics, change locations, impulse samples, stability screen, verdicts, and
  completion marker. `COMPLETE` is execution evidence, not scientific success.

Output-stream SHA-256 for both final runs:
`c6731221dc80e35f35d58a7d99676917d60a0c3808fda6a68cfb1967b0a4d1d9`.
Python independently verified all 168 scores and persistent changes per final run.
Same-machine/toolchain replay only; no cross-platform bit-identity claim.

## Experiment 0001 verification and delivery

Final local checks passed: Rust formatting, strict Clippy,
9 Rust tests, doctest execution (0 examples), release build and documentation;
Python Ruff formatting/lint, 4 unit tests and compilation. Document links and
diff whitespace checks also passed. Implementation commit
`0771a593df9e124a23e42b99ddb7409c4d87d2c7` was pushed and remote `main` matched;
[hosted CI 43256152047](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/runs/43256152047)
passed all three jobs (Rust, Python 3.12, Python 3.14). This delivery note changes
documentation only; recheck the current commit's CI on resumption. Prior
repository checkpoints include foundation `102ff79`,
Principle 001/geometry correction `130d14d`, R1 protocol `6c13c3e`, and reading-list
import `4735898`. No required-check/branch-protection policy is configured.

## Historical next step after Experiment 0001

Do not tune Experiment 0001 retroactively. Register a separate transport
experiment with plasticity disabled: amplitude/channel fidelity against distance,
time, damping and coupling, with boundary/stability checks. Then investigate
shape-sensitive cue binding and persistent plasticity if usable signals exist.
The present color-offset rule cannot route arbitrary source shapes into target
shapes. Capacity, resilient recall, live visualization, and a general-purpose
memory engine remain future work.
