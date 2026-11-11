# QR Neural Memory System

An experimental, Rust-first investigation of a mutable, QR-inspired computational matrix.

We investigate what the substrate's rules actually permit. Useful memory behavior and any unexpected computation must be measured, reproduced, and explained before they become claims.

## Initial direction

- **4,096 computational cells** arranged in a **64 × 64 mutable binary matrix**.
- Rust implementation, CPU execution, and deterministic experiments first.
- Distributed representations, local interactions, and experimentally defined plasticity.
- No conventional QR error correction or conventional neural-network architecture imposed at the start.
- Persistence and deterministic replay as research requirements for the engine.

“QR-inspired” describes the matrix concept; these cells are computational units, not individual QR-code images. Compatibility with standard QR codes is not an initial objective.

## Experiment 001: first measurable memory behavior

1. Teach the substrate associations using an explicit, documented update rule.
2. Attempt reconstruction from incomplete cues.
3. Measure whether learning new associations damages older ones.
4. Compare results against controls and record the initial state, parameters, input sequence, and results needed to reproduce them.

A further research question is whether retrieving a memory can change the substrate and alter subsequent propagation. This remains a hypothesis to test, as does computation beyond associative retrieval.

## Status

Repository initialized. The engine, learning rules, experiments, and validation results are not implemented yet. The specifications above describe the intended starting point, not demonstrated capabilities.
