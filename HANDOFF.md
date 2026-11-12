# QR Neural Memory System — handoff

Updated: 2026-11-12 (America/Toronto).

## Session starting point

The user requested Git setup, a repository under Cognis 11 Inc., the usual agent/handoff documents, severe CI, and public visibility. Git was already installed. The empty project folder was initialized on `main` and the initial research brief was pushed in commit `7fc8ace`.

- Local directory: `/home/cognis/Cognis_Lab/QR neural memory system`
- Repository: https://github.com/Cognis-11-Inc/qr-neural-memory-system
- Owner: `Cognis-11-Inc` (Cognis 11 Inc.)
- Visibility: public, explicitly requested by the user.
- Default branch: `main`; `origin` uses HTTPS.

## Decisions and boundaries

The current target is approximately **80% Rust and 20% Python**, flexible as the research develops. Rust owns the engine and deterministic experiments; Python handles scientific analysis and visualization. Start with **4,096 computational cells**, CPU first. The user corrected the assumed square grid: cell count stays 4,096; grid dimensions, geometry, and connectivity remain unspecified. The internal representation of a QR neuron also remains to be designed.

The user authored **Principle 001**, recorded verbatim in `PRINCIPLES.md` and `README.md`: associations reside in persistent, experience-dependent changes distributed across internal neuron states, with no single neuron assigned ownership of an entire association. Retrieval must arise from deterministic interactions among these local states.

The first scientific objective is association learning, retrieval from partial cues, and measurement of interference with previously learned associations. Retrieval-induced substrate change and computation beyond retrieval remain research questions. No conventional learning architecture or QR error correction is imposed.

The user authored **Hypothesis R1: distributed resilience**, recorded verbatim in `HYPOTHESES.md` and `README.md`. They explicitly want it attacked experimentally: intact A → B retrieval followed by progressive random loss and contiguous regional damage, with capacity, redundancy, and interference measured together. Graceful degradation is a target, not an assumed capability. Eight-neighbor connectivity is proposed; it does not settle grid dimensions or boundary rules. A 20% loss illustration is not measured performance.

## Current implementation

- `src/lib.rs`: 4,096 activation bits indexed by cell ID, without spatial meaning, and empty-state construction. The former square-grid API was replaced by `ActivationState` and `empty_activation_state`; there are no other in-repository callers. No persistent learned internal state, propagation, learning, retrieval, or persistence yet.
- `analysis/metrics.py`: normalized binary reconstruction error, with invalid-input checks.
- Rust test checks the 4,096-cell initial state and independent mutations. Python tests cover known reconstruction errors and invalid comparisons. These are foundation checks, not memory experiments.
- `.github/workflows/ci.yml`: severe CI on pushes, pull requests, and manual runs. Rust formatting, strict Clippy, tests, doctests, release build, and documentation; Python 3.12/3.14 formatting, lint, tests, and compilation. Action revisions, Rust, and Ruff are pinned; workflow permissions are read-only.
- `AGENTS.md`: rehydration, research, validation, and delivery rules.
- `HYPOTHESES.md` and `EXPERIMENT_R1.md`: untested R1 and a proposed loss protocol, including paired masks, disabled-neuron semantics, intact controls, full-target scoring, capacity trade-offs, and unresolved preregistration choices. Corruption remains a separate intervention. No engine behavior was added by these documents.
- `docs/FOUNDATIONAL_PAPERS.md`: user-supplied research reading list, copied unchanged from `/home/cognis/Downloads/FOUNDATIONAL_PAPERS.md` and linked from the README. Use this prior work as a foundation for design and comparisons. Its bibliographic claims were not independently verified during import; it includes an explicitly unverified reference. No full-text PDFs were downloaded or added.

## Verification record

Initial repository creation: local `main` and remote `main` both matched `7fc8acee5f304967e953c82d90c63263f60da136`; the worktree was clean. Public visibility was then verified through GitHub.

Local foundation checks passed: Rust formatting, strict Clippy, 1 unit test, doctest execution (0 examples), release build, and documentation; Python Ruff formatting/lint, 2 unit tests, and compilation. Local Python was 3.13.16; the hosted matrix covers 3.12 and 3.14. No scientific memory experiment has run.

Hosted [Severe CI run 43101324092](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/runs/43101324092) passed all three jobs (Rust, Python 3.12, Python 3.14) for foundation commit `102ff79be42b7309100f5f5abf9f16cbb3eda4a5`. Local and remote `main` matched that commit and the worktree was clean before this verification note was added.

Recheck hosted CI for later commits through the [Actions history](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/workflows/ci.yml). No branch protection or required-check policy has been configured.

2026-11-12 correction: Principle 001 was added verbatim, the assumed square geometry was removed from current documentation and Rust code, and the population remains 4,096. All local severe-CI checks passed after the correction (1 Rust unit test, 2 Python unit tests; no memory experiment). Hosted [CI run 43244414084](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/runs/43244414084) passed for correction commit `130d14d6c94afe1dfa9e856fbc46a038fa912c48`.

The subsequent R1 addition changes documentation only. Local document links, the verbatim R1 statement, and diff whitespace checks passed. Verify hosted CI for its pushed commit. No R1 experiment has run.

Foundational reading-list import: source and destination compared byte-for-byte and their SHA-256 hashes matched. This addition changes documentation only.

## Next bounded research step

Specify Experiment 001 before implementing learning: persistent internal neuron state consistent with Principle 001, geometry/connectivity, local interaction and boundary rules, synchronous versus asynchronous updates, the learning rule, cue corruption procedure, controls, and retention/interference metrics. Record parameters and deterministic input generation. Run the complete 4,096-cell substrate and retain reproducible evidence. Avoid selecting a rule merely because it produces an appealing visualization.

Use `EXPERIMENT_R1.md` to settle the resilience protocol before collecting results. First establish meaningful intact retrieval; then run neuron-loss comparisons and capacity/interference trials. Do not treat the written protocol as an implementation or a demonstration.
