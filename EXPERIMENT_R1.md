# Experiment R1: attack distributed resilience

## Objective and status

Teach A → B, verify retrieval, then progressively disable neurons. Measure whether the learned association remains retrievable and how reconstruction degrades under random losses and contiguous regional damage. Also measure whether resilience competes with memory capacity and interference.

This is a proposed resilience protocol. Experiment 0001 implements candidate learning/recall dynamics and topology, but failed its intact-association criteria; see [measured results](docs/EXPERIMENT_0001_RESULTS.md). Neuron-loss interventions remain unimplemented. No resilience measurement exists.

## Foundational requirements

1. No individual QR neuron is assigned ownership of a complete association, as stated in [Principle 001](PRINCIPLES.md).
2. Graceful degradation under partial damage is a target to test, not assume.

Seek the smallest explicitly defined local learning rule that can satisfy both. Report failures and trade-offs, including a rule that learns nothing, retrieves only before damage, or requires substantial redundancy.

## Decisions required before running

- Define persistent internal neuron state, transient activation, local learning and retrieval rules, update order, readout, stopping conditions, and deterministic input generation.
- Keep the population at 4,096 cells. Specify geometry and boundary conditions separately; no grid dimensions follow from this count.
- **Eight-neighbor connectivity is a candidate for R1.** Experiment 0001 uses it on a nonwrapping 64 × 64 network; adoption for a future working memory model must be explicit. A spatial embedding and boundary rules must be defined before “contiguous regional damage” has an operational meaning. Local communication bottlenecks are a possible failure mechanism to investigate.
- Freeze the A/B pattern construction, cue completeness, output decoding, reconstruction metric, success threshold, trial count, seed list, damage schedule, memory loads, and any redundancy parameter before evaluating results. These values remain open; do not select them after seeing favorable outcomes.
- Define what counts as redundancy for the selected rule. Compare variants with the same cell population and documented state/training budgets; do not attribute a result to redundancy when other changes are confounded with it.

## Paired loss protocol

1. Train A → B and save the full learned state and reproducibility parameters. Clear transient activity according to the specified reset rule, then present the fixed A cue and measure B reconstruction on the intact substrate.
2. Run untrained and unrelated-cue controls to establish a meaningful task baseline. Report intact-learning failures. Report damage performance conditional on intact success and the fraction of all trained trials that qualified, so failed learning is not hidden.
3. For each trained snapshot and damage schedule, construct both seeded random neuron-loss masks and contiguous regional masks with the **same disabled-neuron counts**. Use multiple random masks and regional locations. Include zero damage and complete loss as controls.
4. Make progression nested within each damage sequence: a neuron disabled at one level remains disabled at higher levels. Restore the same learned snapshot before each measurement and apply the selected mask, so prior retrieval attempts do not silently alter the next condition. Reuse masks across matched rule/load variants where applicable.
5. Disable affected neurons throughout retrieval: they cannot update, emit signals, relay communication, or contribute their internal states to the readout. Their stored state is unavailable. Merely setting an activation bit to zero does not constitute neuron loss. Do not silently reconnect surviving neurons around removed cells.
6. Present the same cue and use the same retrieval budget and readout as in the intact condition. Permit no retraining, target injection, access to the intact snapshot during retrieval, or restoration of lost state. Keep the intact snapshot and B ground truth outside the retrieval engine, for trial setup and measurement only.
7. Record exact masks and outputs, reconstruction error, retrieval success, steps to termination, and change from the intact result. Repeat across the prespecified patterns, seeds, and regional locations.

## Measurement and interpretation

Measure reconstruction over the original full B target. Missing output components count as failures under the predefined readout rule; do not improve a score by shrinking the denominator to surviving outputs. For binary output, the existing normalized mismatch metric is available once missing outputs have an explicit scoring convention.

Plot reconstruction quality and success rate against the actual fraction of disabled neurons, separately for random and regional damage. Show trial counts, variability, intact controls, and task baselines. Identify abrupt collapse, location sensitivity, and regional vulnerability; do not conceal them in a single average or a selected successful run.

Distinguish a failure to encode/retrieve on the intact substrate from a loss of an initially retrievable association. For each tested rule, report damage ranges where the preregistered criterion is met or missed. A failed rule constrains that implementation; a successful finite test does not establish universal resilience.

## Capacity, redundancy, and interference

Repeat the paired protocol at increasing numbers of learned associations, preserving training order and snapshots. Test old and new associations before and after additional learning, both intact and damaged. Use independently specified cue/target associations so overlap in neuron-state changes is not confused with overlap deliberately built into the input patterns.

Measure intact recall, retention of older associations, false retrievals, and resilience at each load and redundancy setting. Define capacity as the association load meeting the prespecified retrieval criterion under the stated conditions; report intact and damaged capacity separately. Determine whether redundancy improves loss tolerance while increasing interference or reducing usable capacity, without assuming the direction of the result.

## Illustrations and evidence

A “20% disabled / 80% available” slider or diagram illustrates a damage level only. It is not retrieval performance, a simulation result, or evidence for R1. Label such visuals **Illustration only — resilience not implemented or measured** until actual measured data exists.

## Deferred intervention

Persistent-state corruption is distinct from neuron loss. A later protocol must define which internal fields are corrupted, valid replacement values, the corruption schedule, and whether affected neurons continue participating. This loss protocol supplies no corruption-resilience claim.
