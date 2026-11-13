# QR Neural Memory Principles

## Principle 001

A learned association is represented by persistent, experience-dependent changes distributed across the internal states of many QR neurons. No individual QR neuron is assigned ownership of the complete association. Retrieval must arise from interactions among these local states under explicitly defined deterministic rules.

## Design consequences

- Define persistent internal neuron state separately from transient activation.
- Define deterministic local interactions, update order, and experience-dependent changes explicitly.
- Do not assign a complete association to one neuron or substitute a central association lookup for retrieval through local interactions.
- Evaluate distributed storage and retrieval experimentally; declaring this principle does not demonstrate either behavior.

The current starting population is **4,096 QR neurons**, each with independently mutable internal color-and-shape state. This count does not specify geometry. [Experiment 0001](EXPERIMENT_0001.md) explicitly chooses a 64 × 64 network with 8 × 8 internal modules and eight nonwrapping external neighbors; these remain experimental choices.

## Morphic QR Neuron Principle 002 — proposed

Proposed by the user on 2026-11-13. Preserved wording:

> A neuron's vibrational activity is sustained or diminished through its intrinsic response and energy exchange with neighboring neurons. Its internal morphology determines how it receives, retains, and transmits excitation. Sustained interactions may modify that morphology, subject to an explicit plasticity law.

### Interpretation and experimental consequences

- Morphology refers to each neuron's independently owned internal color-and-shape configuration. Its mapping to intrinsic response and neighbor coupling must be defined by the experiment, separately from transient displacement and velocity.
- Retaining excitation means a transient dynamical response. Persistent associative memory still requires the distributed, experience-dependent changes and retrieval specified by Principle 001.
- Energy exchange requires an explicit energy definition and measured balance or transfer rule. Existing sums of squared displacement and velocity are diagnostic proxies; they do not establish physical energy conservation or measured energy transfer.
- Interaction-driven morphology changes require a separately registered plasticity law, with explicit persistent variables, local update conditions, bounds, and a frozen-morphology control. Proposing this principle does not enable plasticity in existing experiments.

The current fixed-morphology experiments measure configuration-dependent response,
propagation, interference, and decay. They do not demonstrate that sustained
interactions produce useful persistent morphological changes or learned memory.
The previously reported timestep-accuracy failures remain unresolved. See the
[experiment progression](docs/EXPERIMENTS.md) for measured evidence and limits.
