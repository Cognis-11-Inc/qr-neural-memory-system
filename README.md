# QR Neural Memory System

An experimental, Rust-first investigation of a mutable, QR-inspired computational matrix.

We investigate what the substrate's rules actually permit. Useful memory behavior and any unexpected computation must be measured, reproduced, and explained before they become claims.

## Initial direction

- **4,096 independently reconfigurable QR neurons**. Experiment 0001 uses a **64 × 64 network**, each neuron containing its own **8 × 8 spatial array of colored modules**. These are explicit experiment parameters; the population count alone does not define geometry.
- Approximately **80% Rust / 20% Python**, a flexible architectural target. Rust owns the engine and deterministic experiments; Python supports analysis, statistics, plots, and comparisons.
- CPU execution and deterministic experiments first.
- Distributed representations, local interactions, and experimentally defined plasticity.
- No conventional QR error correction or conventional neural-network architecture imposed at the start.
- Persistence and deterministic replay as research requirements for the engine.

Experiment 0001 implements five module values (empty, red, green, blue, yellow). Their spatial arrangement is the neuron's shape. Color and shape affect computation. Ripple amplitudes and momentum are transient and separate from persistent module configuration. Compatibility with standard QR codes is not an initial objective.

## QR Neural Memory Principle 001

A learned association is represented by persistent, experience-dependent changes distributed across the internal states of many QR neurons. No individual QR neuron is assigned ownership of the complete association. Retrieval must arise from interactions among these local states under explicitly defined deterministic rules.

See [PRINCIPLES.md](PRINCIPLES.md) for the design consequences.

## Hypothesis R1: distributed resilience

Associations encoded through overlapping, persistent changes across multiple QR neurons can remain retrievable after partial loss or corruption of the substrate.

**Untested.** We will attack this hypothesis by verifying A → B retrieval, then progressively disabling neurons under matched random and contiguous regional damage. Graceful degradation is a target to test. Capacity, redundancy, and interference must be measured together. Experiment 0001 implements eight-neighbor connectivity without demonstrating working association retrieval; lesion testing is deferred.

See [HYPOTHESES.md](HYPOTHESES.md) and the proposed [R1 experiment protocol](EXPERIMENT_R1.md). Damage illustrations are not measured retrieval results.

## Experiment 001: first measurable memory behavior

1. Teach the substrate associations using an explicit, documented update rule.
2. Attempt reconstruction from incomplete cues.
3. Measure whether learning new associations damages older ones.
4. Compare results against controls and record the initial state, parameters, input sequence, and results needed to reproduce them.

A further research question is whether retrieving a memory can change the substrate and alter subsequent propagation. This remains a hypothesis to test, as does computation beyond associative retrieval.

## Status

Experiment 0001 is implemented and measured: a local color-transition baseline and a two-scale damped-wave candidate, with persistent per-neuron module arrays, synchronous updates, transient reset, partial-cue recall, sequential training, controls, ablations, and exported snapshots. **None of the 24 registered trials met the association or retention criteria.** The ripple arm produced empty outputs and no persistent changes at the registered distance/duration; the local baseline changed 16 neurons but reconstructed targets poorly. No successful learning, distributed-memory mechanism, or resilience is demonstrated.

Read the [rules and preregistered criteria](EXPERIMENT_0001.md) and [actual results and failures](docs/EXPERIMENT_0001_RESULTS.md). Raw evidence remains local under the ignored `results/` directory.

To reproduce, use a fresh output directory for each run:

```sh
cargo run --locked --release --bin experiment_0001 -- results/my-experiment-0001
.venv/bin/python -m analysis.experiment_0001 results/my-experiment-0001
```

The runner refuses an existing output directory. The analyzer independently checks raw outputs, metrics, snapshots, change locations/counts, and trial decisions. A completed execution is distinct from a positive scientific result.

Read [AGENTS.md](AGENTS.md) and [HANDOFF.md](HANDOFF.md) when resuming work.

## Research foundations

The [foundational research reading list](docs/FOUNDATIONAL_PAPERS.md) collects the prior work we will build upon when designing neuron state, local learning rules, and experimental comparisons. It is a research foundation, not evidence that our substrate works. Paper-specific mechanisms and assumptions must be distinguished from our own hypotheses and measured results.

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
