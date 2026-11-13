# Experiment progression

| Experiment | Question | Status | Design | Evidence |
|---|---|---|---|---|
| 0001 | Can local color-transition traces or ripple dynamics retrieve associations? | Implemented and measured; negative registered result | [Frozen design](../EXPERIMENT_0001.md) | [Results and failures](EXPERIMENT_0001_RESULTS.md) |
| 0002 T/R | Can configuration-dependent intrinsic oscillations transmit signals and show controlled spectral selectivity? | Implemented and measured: T negative at four hops; R indeterminate | [Frozen registration](../EXPERIMENT_0002.md) | [Results, controls and figures](EXPERIMENT_0002_RESULTS.md) |
| 0002 P | Can resonance condition persistent reconfiguration? | Deferred; not authorized for this delivery | Requires separate registration | No results |
| 0003 | How do volumetric topology and configuration affect dynamics, and can a viewer reproduce measured state? | Implemented and measured: 111 cases × two identical runs; native iGPU viewer verified | [Frozen registration](../EXPERIMENT_0003.md) | [Results and instrument checks](EXPERIMENT_0003_RESULTS.md) |
| 0004 | Do extended dual-source fields obey signed superposition in a frozen medium? | Measured: superposition within roundoff; 0/8 long-window timestep screens pass | [Frozen registration](../EXPERIMENT_0004.md) | [Results and signed fields](EXPERIMENT_0004_RESULTS.md) |
| 0004 extension | Does the current frozen AB+ signal decay over 3,072 ticks, observed every tick? | Measured: 3,073 frames; 494-tick final quiet tail, nonzero signal; prior accuracy limitation retained | [Extension registration](../EXPERIMENT_0004_EXTENSION.md) | [Results and portable report](EXPERIMENT_0004_EXTENSION_RESULTS.md) |
| R1 | Does a working distributed association survive partial loss? | Proposed; intact retrieval prerequisite unmet | [Proposed protocol](../EXPERIMENT_R1.md) | No resilience measurements |

Experiments 0001/0002's sources, parameters, results, and raw evidence remain
unchanged. New experiments have separate runners and output folders. A software
test pass is distinct from a scientific result; an executed negative experiment
is a completed research deliverable.

## Dated record

- 2026-11-12: Experiment 0001 implemented, measured, independently verified and
  replayed. No successful association retrieval or resilience demonstrated.
- 2026-11-12 16:32:35 UTC: Experiment 0002 registration draft started; Stage P deferred.
- 2026-11-12 16:38:24 UTC: T/R registration frozen in commit `f3167f1` before
  implementation or data collection.
- 2026-11-12: T/R executed twice: 73 transport and 1,028 spectral/control cases
  per run, byte-identical evidence independently checked in Python. T fails the
  four-hop normalized-transfer criterion; R remains numerically indeterminate.
  Neither failed criteria nor thresholds were revised after inspection.
- 2026-11-13 03:34:26 UTC (Nov 12 local): Experiment 0003 registration frozen
  in `5a112c1`, before measurements; implementation checkpoint `e9d9dca`.
- 2026-11-13 UTC: 111 volumetric/legacy/control cases executed twice with
  identical evidence. Native Bevy live/replay checks matched headless data on
  Raphael iGPU. No plasticity, resonance success, memory or resilience claimed.
- 2026-11-13: Experiment 0004 frozen in `4a2283a` before measurements; two
  identical 35-execution suites, 408 full signed frames per suite. Superposition
  passes the registered tolerance; all eight long-window timestep screens fail.
  Palettes unchanged; persistent interaction traces remain deferred.
- 2026-11-13: User requested triple horizon and every tick. Extension frozen
  in `2c39004`; two byte-identical3,073-frame runs,12original-prefix comparisons,
  independently verified native every-tick replay. Quiet tail begins2579;
  no exact zero or converged-physical-model claim. Portable agent report added.
