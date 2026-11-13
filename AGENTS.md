# Working agreement

## Rehydrate first

Read this file, `HANDOFF.md`, `README.md`, `PRINCIPLES.md`, and `HYPOTHESES.md`, then inspect `git status`, the current branch, and recent commits before editing. For Experiment 0001, also read `EXPERIMENT_0001.md` and `docs/EXPERIMENT_0001_RESULTS.md`. Preserve existing user work. Recheck recorded facts against the checkout; a handoff is not live evidence.

For historical Experiment 0002 work, read `EXPERIMENT_0002.md`, `docs/EXPERIMENT_0002_RESULTS.md`, and `docs/EXPERIMENTS.md`. Keep designs, raw evidence, and results separate so progression remains inspectable.

For Experiment 0003, read `EXPERIMENT_0003.md`, `docs/EXPERIMENT_0003_RESULTS.md`, and `viewer/README.md`. Preserve Experiments 0001/0002 unchanged. Root physics and versioned observations are authoritative; Bevy is an isolated consumer. Default visuals to the low-power iGPU; reserve the 5060 Ti for models unless explicitly overridden. Distinguish configured frame caps from measured FPS and synthetic label fixtures from engine observations.

## Research direction

- Aim for roughly **80% Rust / 20% Python**, subject to change. This is an architectural direction, not a line-count quota.
- Rust owns the substrate, state transitions, learning, propagation, persistence, and deterministic experiment execution. Python supports analysis, statistics, plots, and comparisons.
- Begin with **4,096 QR neurons**, CPU first and deterministic first. Each must have an independently mutable internal spatial color-and-shape configuration. Experiment 0001 explicitly adopts a 64 × 64 network, 8 × 8 internal modules, and eight nonwrapping external neighbors from the supplied brief. These are experiment parameters, not consequences of the cell count or universal architecture requirements.
- Follow Principle 001 in `PRINCIPLES.md`: learned associations must reside in persistent, experience-dependent internal changes distributed across many neurons, with retrieval through deterministic local interactions. Keep wave amplitude/momentum separate from persistent configuration. The original activation-bit scaffold is not the neuron model used by the experiment.
- Treat Hypothesis R1 as untested and attack it using `EXPERIMENT_R1.md`: compare random and contiguous damage, report failures, and measure capacity/interference trade-offs. Finalize R1 topology and damage semantics separately; Experiment 0001's eight-neighbor geometry is a candidate, not a resilience result. Damage illustrations are not retrieval evidence.
- Do not introduce conventional QR error correction, PyTorch, a conventional neural-network architecture, GPU execution, or parallel updates without an experimental reason and an explicit discussion of the design change.
- Keep state representation separate from update and learning rules. Document boundary conditions, update order, seeds, and parameters when those mechanisms are introduced.
- Experiment 0002 T/R measures fixed-structure intrinsic dynamics, transport, and spectral selectivity; memory retrieval is not its pass/fail criterion. Stage P plasticity is deferred and requires separate registration/authorization. Preserve Experiment 0001's sources, parameters, specifications, negative results, and existing raw evidence unchanged.
- Do not claim learning, emergence, intelligence, or memory performance from scaffolding, cell counts, or successful compilation. Unexpected behavior requires controls, reproduction, and a causal investigation.

## Implementation and evidence

- Keep dependencies and abstractions proportional to the experiment. Preserve reproducibility over premature optimization.
- Add meaningful tests for state and measurement invariants, deterministic replay, and scientific claims as those features arrive. Never weaken checks to manufacture a green run.
- Run the applicable commands in `README.md`; severe CI must fail on formatting, lint, test, compilation, or documentation errors. Keep the toolchain and dependency pins consistent with CI.
- Distinguish source changes, local checks, observed experiment results, pushed commits, and hosted CI. Report gaps plainly.
- Inspect the actual diff and target contents before delivery. Before commits, run `git diff --cached --check` and inspect the staged summary. After a push, verify the remote branch SHA.
- Update `HANDOFF.md` with current state, evidence, unresolved questions, and the next bounded step. Do not put credentials, private lab material, generated datasets, or checkpoint dumps into Git.
- Repository ownership is **Cognis-11-Inc**, regardless of the local Git author. The user explicitly requested public visibility. A push authorization covers the requested delivery; do not infer authorization for unrelated publishing or messages.
