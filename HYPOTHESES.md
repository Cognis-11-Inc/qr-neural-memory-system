# QR Neural Memory Hypotheses

## R1: Distributed resilience

Associations encoded through overlapping, persistent changes across multiple QR neurons can remain retrievable after partial loss or corruption of the substrate.

**Status: untested.** R1 is a hypothesis to attack experimentally. Graceful degradation is a design target, not an established property or a consequence guaranteed by Principle 001.

The first test is specified in [EXPERIMENT_R1.md](EXPERIMENT_R1.md). It tests simulated neuron loss. Persistent-state corruption requires a separate, explicitly defined intervention; loss results alone do not establish corruption resilience.
