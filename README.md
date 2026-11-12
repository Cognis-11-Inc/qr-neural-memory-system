# QR Neural Memory System

An experimental, Rust-first investigation of a mutable, QR-inspired computational matrix.

We investigate what the substrate's rules actually permit. Useful memory behavior and any unexpected computation must be measured, reproduced, and explained before they become claims.

## Initial direction

- **4,096 computational cells**. Grid dimensions, geometry, and connectivity remain unspecified.
- Approximately **80% Rust / 20% Python**, a flexible architectural target. Rust owns the engine and deterministic experiments; Python supports analysis, statistics, plots, and comparisons.
- CPU execution and deterministic experiments first.
- Distributed representations, local interactions, and experimentally defined plasticity.
- No conventional QR error correction or conventional neural-network architecture imposed at the start.
- Persistence and deterministic replay as research requirements for the engine.

The internal representation of a QR neuron remains to be specified. Compatibility with standard QR codes is not an initial objective.

## QR Neural Memory Principle 001

A learned association is represented by persistent, experience-dependent changes distributed across the internal states of many QR neurons. No individual QR neuron is assigned ownership of the complete association. Retrieval must arise from interactions among these local states under explicitly defined deterministic rules.

See [PRINCIPLES.md](PRINCIPLES.md) for the design consequences.

## Hypothesis R1: distributed resilience

Associations encoded through overlapping, persistent changes across multiple QR neurons can remain retrievable after partial loss or corruption of the substrate.

**Untested.** We will attack this hypothesis by verifying A → B retrieval, then progressively disabling neurons under matched random and contiguous regional damage. Graceful degradation is a target to test. Capacity, redundancy, and interference must be measured together. Eight-neighbor connectivity remains a proposal; geometry is unspecified.

See [HYPOTHESES.md](HYPOTHESES.md) and the proposed [R1 experiment protocol](EXPERIMENT_R1.md). Damage illustrations are not measured retrieval results.

## Experiment 001: first measurable memory behavior

1. Teach the substrate associations using an explicit, documented update rule.
2. Attempt reconstruction from incomplete cues.
3. Measure whether learning new associations damages older ones.
4. Compare results against controls and record the initial state, parameters, input sequence, and results needed to reproduce them.

A further research question is whether retrieving a memory can change the substrate and alter subsequent propagation. This remains a hypothesis to test, as does computation beyond associative retrieval.

## Status

The foundation provides a 4,096-cell activation-bit representation indexed by cell ID and a Python reconstruction-error metric. The activation bits are scaffolding; persistent internal neuron state, learning, propagation, retrieval, persistence, and Experiment 001 are not implemented yet. No memory-performance or emergence claim has been demonstrated.

Read [AGENTS.md](AGENTS.md) and [HANDOFF.md](HANDOFF.md) when resuming work.

## Development checks

Rust is pinned in `rust-toolchain.toml`. Python requires 3.12 or newer; hosted CI checks 3.12 and 3.14. Create an isolated Python environment and install the pinned development tools:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements-dev.txt
```

Run the same checks used by severe CI:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
cargo build --locked --release --all-targets --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps --all-features
.venv/bin/python -m ruff format --check .
.venv/bin/python -m ruff check .
.venv/bin/python -W error -m unittest discover -s analysis -t . -v
.venv/bin/python -W error -m compileall -q analysis
```

CI additionally sets `RUSTFLAGS='-D warnings'`. Workflow: [Severe CI](https://github.com/Cognis-11-Inc/qr-neural-memory-system/actions/workflows/ci.yml).
