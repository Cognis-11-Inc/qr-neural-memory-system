# QR Neural Memory System — handoff

Updated: 2026-11-12 (America/Toronto).

## Rehydrate

Read `AGENTS.md`, `README.md`, `PRINCIPLES.md`, `HYPOTHESES.md`,
`EXPERIMENT_0001.md`, and `docs/EXPERIMENT_0001_RESULTS.md`, then inspect the
current worktree and recent commits. For the latest scope also read
`EXPERIMENT_0002.md`, `docs/EXPERIMENT_0002_RESULTS.md`, and
`docs/EXPERIMENTS.md`. The reports distinguish software checks,
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

## Latest delivery: Experiment 0002 T/R

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
