# QR Neural Memory System — handoff

Updated: 2026-11-11 (America/Toronto).

## Session starting point

The user requested Git setup, a repository under Cognis 11 Inc., the usual agent/handoff documents, severe CI, and public visibility. Git was already installed. The empty project folder was initialized on `main` and the initial research brief was pushed in commit `7fc8ace`.

- Local directory: `/home/cognis/Cognis_Lab/QR neural memory system`
- Repository: https://github.com/Cognis-11-Inc/qr-neural-memory-system
- Owner: `Cognis-11-Inc` (Cognis 11 Inc.)
- Visibility: public, explicitly requested by the user.
- Default branch: `main`; `origin` uses HTTPS.

## Decisions and boundaries

The current target is approximately **80% Rust and 20% Python**, flexible as the research develops. Rust owns the engine and deterministic experiments; Python handles scientific analysis and visualization. Start with 4,096 binary computational cells in a 64 × 64 matrix, CPU first. QR-inspired does not mean standards-compliant QR codes or one QR image per cell.

The first scientific objective is association learning, retrieval from partial cues, and measurement of interference with previously learned associations. Retrieval-induced substrate change and computation beyond retrieval remain research questions. No conventional learning architecture or QR error correction is imposed.

## Current implementation

- `src/lib.rs`: initial row-major binary matrix and empty-state construction. No propagation, learning, retrieval, or persistence yet.
- `analysis/metrics.py`: normalized binary reconstruction error, with invalid-input checks.
- Rust test checks the 4,096-cell initial state and independent mutations. Python tests cover known reconstruction errors and invalid comparisons. These are foundation checks, not memory experiments.
- `.github/workflows/ci.yml`: severe CI on pushes, pull requests, and manual runs. Rust formatting, strict Clippy, tests, doctests, release build, and documentation; Python 3.12/3.14 formatting, lint, tests, and compilation. Action revisions, Rust, and Ruff are pinned; workflow permissions are read-only.
- `AGENTS.md`: rehydration, research, validation, and delivery rules.

## Verification record

Initial repository creation: local `main` and remote `main` both matched `7fc8acee5f304967e953c82d90c63263f60da136`; the worktree was clean. Public visibility was then verified through GitHub.

Local foundation checks passed: Rust formatting, strict Clippy, 1 unit test, doctest execution (0 examples), release build, and documentation; Python Ruff formatting/lint, 2 unit tests, and compilation. Local Python was 3.13.16; the hosted matrix covers 3.12 and 3.14. No scientific memory experiment has run.

Hosted CI must be verified for the current commit after pushing; consult the [Actions history](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/workflows/ci.yml) rather than interpreting local checks as hosted evidence. No branch protection or required-check policy has been configured.

## Next bounded research step

Specify Experiment 001 before implementing learning: the representation of an association, local neighborhood and edge behavior, synchronous versus asynchronous updates, the learning rule, cue corruption procedure, controls, and retention/interference metrics. Record parameters and deterministic input generation. Run the complete 4,096-cell substrate and retain reproducible evidence. Avoid selecting a rule merely because it produces an appealing visualization.
