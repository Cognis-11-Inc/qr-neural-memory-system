# QR Neural Memory System — handoff

Updated: 2026-11-12 (America/Toronto).

## Rehydrate

Read `AGENTS.md`, `README.md`, `PRINCIPLES.md`, `HYPOTHESES.md`,
`EXPERIMENT_0001.md`, and `docs/EXPERIMENT_0001_RESULTS.md`, then inspect the
current worktree and recent commits. The report distinguishes software checks,
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

## Latest task and design decisions

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

## Implementation

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

## Actual results

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

## Verification and delivery

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

## Next bounded step

Do not tune Experiment 0001 retroactively. Register a separate transport
experiment with plasticity disabled: amplitude/channel fidelity against distance,
time, damping and coupling, with boundary/stability checks. Then investigate
shape-sensitive cue binding and persistent plasticity if usable signals exist.
The present color-offset rule cannot route arbitrary source shapes into target
shapes. Capacity, resilient recall, live visualization, and a general-purpose
memory engine remain future work.
