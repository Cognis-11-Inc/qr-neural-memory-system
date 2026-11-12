<!-- Imported proposal; trailing whitespace normalized. Actual authorized T/R design is EXPERIMENT_0002.md. -->

# QR Neural Network — Experiment 0002

**Working title:** Intrinsic Vibration, Resonance, and Adaptive Reconfiguration
**Status:** Proposed experiment specification / Codex handoff; **not implemented or measured**.
**Prepared:** 2026-11-12.
**Project:** `Cognis-11-Inc/qr-neural-memory-system`

## 0. Intent and correction of scope

Experiment 0002 studies **a reconfigurable dynamical neural network**, not primarily associative memory. Memory is a possible downstream outcome, not a pass/fail criterion here. Unexpected dynamics must be characterized rather than described as emergence without evidence.

User's conceptual requirement: individual QR neurons possess an **intrinsic vibrational quality** dependent on internal color-and-shape configuration. External activity excites those intrinsic dynamics. Neurons influence neighboring neurons. Sustained, selective resonance **may** change persistent local configuration, in turn changing future responses.

Use *vibrational quality* operationally: frequency response, phase, damping, mode structure, selective amplification, and coupling. No subjective qualia or biological/neural equivalence is claimed.

**Core loop (hypothesis):** persistent configuration → intrinsic dynamics → stimulus response and resonance → possible experience-dependent reconfiguration → modified intrinsic dynamics.

## 1. Evidence inherited from Experiment 0001 (do not rewrite history)

Inspect `AGENTS.md`, `HANDOFF.md`, `README.md`, `PRINCIPLES.md`, `EXPERIMENT_0001.md`, and `docs/EXPERIMENT_0001_RESULTS.md` against the current checkout before edits.

- Experimental network: 64 × 64 = **4,096** neurons, each with an independently mutable **8 × 8** matrix of 5 module values: 0 empty; 1–4 palette colors. This geometry is a test fixture, **not** an immutable architectural law.
- Experiment 0001 used transient channel amplitudes with previous-step amplitudes, four internal cardinal neighbors per module, and eight nonwrapping adjacent neurons at corresponding module coordinates, with synchronous snapshot updates.
- Registered Experiment 0001 results were negative: no demonstrated associative retrieval. The ripple arm produced no persistent modifications; usable remote amplitude at its output was extremely small at cycle 80. The local arm changed only the 16 taught output neurons and did not retrieve target patterns reliably.
- Experiment 0001's local results and acceptance criteria are frozen. Do not modify its sources, parameters, or interpretation to manufacture a positive result. New dynamics require a new runner and new output directory.

## 2. Central research questions

Q1. Can distinct persistent color-and-shape configurations produce measurably different **intrinsic frequency responses** under identical forcing?

Q2. Can local disturbances excite these responses **inside each neuron** and propagate **between neurons** with measurable speed, attenuation, phase shift, and directional effects?

Q3. Can matched-frequency or phase-coherent stimuli yield a selective resonant response beyond matched-energy off-resonance controls?

Q4. Under an explicit, local, bounded plasticity rule, can sustained resonance change configurations *outside the directly stimulated sites*, and do those changes modify subsequent dynamics after all transient activity is reset?

Q5. Does the system exhibit any reproducible unanticipated dynamics? If so, define the observable, reproduce it, compare it to controls, and identify candidate causal mechanisms before naming it.

**Do not require A → B recall, sequential association retention, intelligence, consciousness, or resilience to pass Experiment 0002.** Keep their research questions alive separately.

## 3. Two distinct forms of neuron state

### Persistent structure

Keep each neuron's 8 × 8 matrix of module values, where both **palette assignment** and **spatial arrangement** affect the equations. This is the structural state that survives resets and can reconfigure through locally measured excitation.

Initial configurations **must be explicitly defined** and seeded; an all-zero homogeneous medium cannot demonstrate individual intrinsic differences. Compare:

1. homogeneous configuration control;
2. heterogeneous, reproducibly seeded configurations;
3. spatially rearranged configurations retaining module-color histograms.

Do not inject arbitrary neuron-ID-based frequency differences unless separately identified as an intervention. Any “individual signature” should be traceable to configuration or explicitly recorded parameters.

### Transient dynamical state

For every neuron, internal module, and four color channels, distinguish:

- `u`: displacement/amplitude;
- `v`: velocity or momentum-like time derivative;
- `s`: externally applied or locally transmitted drive;
- optional *diagnostic* accumulators for phase/power/correlation that are not silently treated as learned memory.

Reset must erase `u`, `v`, and all transient diagnostic/eligibility traces while preserving only the declared persistent structure.

## 4. Candidate oscillatory dynamics: make explicit before trials

A **candidate numerical model** for module `i`, color channel `k`, time step `t`:

    a[i,k] = -omega[i,k](P)^2 * u[i,k]
             - gamma[i,k](P) * v[i,k]
             + kappa_in * sum_{j in internal(i)} w_in[i,j,k](P) * (u[j,k] - u[i,k])
             + kappa_out * sum_{j in external(i)} w_out[i,j,k](P) * (u[j,k] - u[i,k])
             + s[i,k]

    v_next[i,k] = v[i,k] + dt * a[i,k]
    u_next[i,k] = u[i,k] + dt * v_next[i,k]

Here `P` is the **persistent module configuration**. This is a *semi-implicit Euler candidate*, not a claim that it is stable for the chosen parameters. Codex must explicitly set all coefficients, units/nondimensionalization, initialization, boundaries, precision, and forcing in a **registered design section before viewing outcome metrics**. It may select another documented integrator for stability, with rationale and appropriate numerical convergence comparisons.

Design requirements:

- **Intrinsic dynamics:** `omega` and/or `gamma` must depend on local color/shape. A candidate is a reproducible function of palette and surrounding module arrangement, e.g., local occupied-neighbor count and channel match. Do not collapse shape into a single histogram.
- **Internal interactions:** four cardinal modules initially, with nonwrapping edges, unless a justified controlled alternative is preregistered.
- **External interactions:** the eight surrounding neurons initially, with nonwrapping boundaries and clear module-to-module mapping. A distance-sensitive alternative should be a separately identified comparison, not a silent change.
- **Synchronous update:** all next states calculated from one previous-state snapshot.
- **Identical drive schedule** for paired comparisons.
- **Local causality:** no target pattern, global learned pair table, or evaluator data inside updates.
- **Channel coupling:** start with four independent channels; cross-channel mixing, if introduced, is a separate explicitly tested design change.

A damped oscillator can exhibit a frequency-dependent response even without an identifiable learned association. Test that first.

## 5. Staged plan: transport → resonance → reconfiguration

### Stage T — Usable transport without learning

Plasticity disabled. Run preregistered impulse and finite-duration tone/burst stimuli from localized inputs. Compare propagation against distance (graph hop and Euclidean lattice distance), elapsed cycles, channel, and direction; test center, edge, and corner injections.

Record per-channel peak absolute amplitude, arrival latency under an externally fixed threshold, RMS amplitude, total squared amplitude (call it a diagnostic **not** conserved physical energy), velocity RMS, frequency spectrum, nonfinite counts, and cardinal-versus-diagonal anisotropy. Inspect intermediate distances, not only the opposite edge of the 64 × 64 medium.

Include the Experiment 0001 transport rule as an **unchanged reference** when feasible, with matched stimulation and explicit acknowledgment of unequal state dimension and compute costs.

### Stage R — Intrinsic frequency response and resonance

With plasticity still disabled, use a fixed frequency sweep for single-neuron, isolated internal-neuron, and network-coupled conditions. Distinguish “neuron” (8 × 8 internal modules) from “module” (one cell within the neuron).

For each configuration and channel, measure the frequency-response curve and response phase relative to the drive. Define a resonance metric before measuring (for example: steady-window RMS response divided by matched-drive RMS, with startup transients excluded). Identify peaks only when repeatable and greater than preregistered neighboring-frequency or off-resonance controls.

Required controls:

- homogeneous vs heterogeneous configurations;
- different shape, same histogram of colors;
- equal-amplitude and **equal-energy** off-resonant forcing;
- same spectrum with randomized phase or timing, where applicable;
- uncoupled internal-only versus coupled network;
- damping and zero-drive controls.

A larger amplitude caused only by higher injected energy **does not establish resonance**.

### Stage P — Resonance-gated persistent plasticity

Only after Stage T/R diagnostics, create a **separately registered** adaptive arm. Specify an observable eligibility variable and a deterministic mapping to module changes before running comparative trials. Candidate concepts include time-integrated positive drive-response work or coherence, but no rule is selected by the user yet.

Minimum requirements:

- local-only eligibility; no evaluator targets or arbitrary global rewriting;
- local bounded update rate and a reproducible tie-break rule;
- a stated threshold or smooth update function chosen *before* outcome inspection;
- state-dependent reconfiguration of color **and spatial pattern**, not only display changes;
- prevent uncontrolled drift under zero forcing;
- report persistent mutation locations, not merely totals;
- clear all transients, replay identical probes, and compare pre-/post-training frequency-response curves;
- compare resonance-gated, frequency-mismatched, energy-matched, frozen, and random/nonselective plasticity controls.

If transport is still inadequate, **report the failure and do not imply Stage P proves distributed learning**. It is acceptable to complete T/R and defer P. Stage P does not need to prove associative memory.

## 6. Preregistration and decision criteria

Codex must write a **frozen preregistration before data collection** giving: parameter grid, seeds, initial configuration generation, timestep/integrator, stimulus frequencies and amplitudes, burn-in/readout windows, observation coordinates, boundary conditions, resonance metric, numerical stability criteria, error handling, and criteria for a positive/negative/indeterminate result for each stage.

Do **not** reuse Experiment 0001's 1e-12 decoder threshold as the scientific definition of transport. Separate numerical detectability, practically usable signal-to-noise, and any eventual decoding objective. Record raw amplitudes and normalized transfer ratios; preregister any transport threshold independently.

A positive Stage T result means **reproducible measurable signal transmission to a stated distance under controlled conditions**, not retrieval. A positive Stage R result means **repeatable, configuration-sensitive spectral selectivity under matched-input controls**, not biological resonance. A positive Stage P result means **resonance-conditioned persistent reconfiguration demonstrably changes subsequent response after reset**, not learned semantic associations.

Keep a negative result as a real deliverable; retain all variants and seeds. No cherry-picking favorable seeds or changing the threshold after inspection.

## 7. Robustness and implementation tests

- Fully specified Rust CPU execution, determinism and reproducible seeds.
- Synchronous update traversal-independence and no new-state leakage.
- Internal/external neighbor counts and no wraparound.
- Finite-speed response from isolated impulses and no nonlocal instantaneous effect.
- Reset clears both `u` and `v`, preserving persistent configuration.
- Same color histogram, different shape can change internal dynamics as specified.
- Changed persistent configurations alter frequency response in a controlled test.
- Nonfinite detection; evaluate stability on *every* time step, not selected samples.
- Fixed-step convergence/step-size sensitivity; monitor whether apparent resonance peaks are numerical artifacts.
- Verify fixed source energy and equal-energy controls.
- Output paths are fresh; raw seeds, inputs, parameters, full observables and state snapshots are exported.
- Python independently verifies summary metrics from raw numerical traces where practical.
- Existing Experiment 0001 remains reproducible and unchanged.

## 8. Proposed file/delivery structure

Codex may reorganize file names only with a clear rationale and reference updates.

- `EXPERIMENT_0002.md`: frozen actual design, implementation decisions, and registration amendments with timestamps.
- `src/experiment_0002.rs`: separate Rust engine/module; may reuse factored pure helpers without changing Experiment 0001 behavior.
- `src/bin/experiment_0002.rs`: isolated runner.
- `analysis/experiment_0002.py`: independent verification and analysis of raw traces.
- `docs/EXPERIMENT_0002_RESULTS.md`: every positive, negative, and indeterminate observation with controls.
- `results/experiment_0002-.../`: ignored raw output and snapshots, exported sources/parameters, reproducibility evidence.
- `HANDOFF.md`: what actually ran, maturity level, limitations, next falsification.

Run the repository's applicable formatting, Clippy, tests, release build, documentation, and Python validation. Keep the severe CI green. Clearly distinguish authored spec, local runs, independent verification, commits, pushed SHA, and hosted CI status.

## 9. Instructions for local Codex

> Read AGENTS.md, HANDOFF.md, README.md, PRINCIPLES.md, EXPERIMENT_0001.md, and docs/EXPERIMENT_0001_RESULTS.md. Use this document as the **proposal**, not a record of implemented code. First write and freeze the detailed Experiment 0002 registration for the two-scale **intrinsic vibration/resonance** network. Keep memory retrieval out of the pass/fail definition. Implement and measure Stage T (usable transport), then Stage R (configuration-dependent resonance) with controls. Proceed to Stage P only under a separately registered plasticity rule and after adequate diagnostic evidence. Preserve Experiment 0001 unchanged. Report unexpected or negative results without tuning them away. Update HANDOFF.md with evidence and run the full quality gates. Do not claim novelty or memory capabilities without comparison and experiments.
