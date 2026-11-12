# QR Neural Memory Principles

## Principle 001

A learned association is represented by persistent, experience-dependent changes distributed across the internal states of many QR neurons. No individual QR neuron is assigned ownership of the complete association. Retrieval must arise from interactions among these local states under explicitly defined deterministic rules.

## Design consequences

- Define persistent internal neuron state separately from transient activation.
- Define deterministic local interactions, update order, and experience-dependent changes explicitly.
- Do not assign a complete association to one neuron or substitute a central association lookup for retrieval through local interactions.
- Evaluate distributed storage and retrieval experimentally; declaring this principle does not demonstrate either behavior.

The current starting population is **4,096 QR neurons**, each with independently mutable internal color-and-shape state. This count does not specify geometry. [Experiment 0001](EXPERIMENT_0001.md) explicitly chooses a 64 × 64 network with 8 × 8 internal modules and eight nonwrapping external neighbors; these remain experimental choices.
