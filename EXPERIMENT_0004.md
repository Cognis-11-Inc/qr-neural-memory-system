# Experiment 0004: extended propagation, dual waves and superposition

Registered 2026-11-13 (America/Toronto), before implementation and measurements.
The Git registration commit is the freeze checkpoint. Preserve Experiments
0001–0003 sources, parameters and measured reports. The shared viewer may gain
an explicitly labeled, replay-only 0004 input path; old recordings stay valid.

## Question and expected null

Can two disturbances propagate through the frozen lattice, overlap, reinforce
or cancel, and continue evolving consistently with linear superposition?
Fixed configuration makes the inherited equations linear. Therefore the expected
null is AB+ = A + B and AB- = A - B, to floating-point tolerance. A visible
interference pattern is not a nonlinear interaction, learning or memory trace.
Persistent changes and plasticity are explicitly deferred to a later experiment.

## Fixed rules and conditions

- 4,096 independently owned 8×8 palette arrays; frozen throughout each run.
- Cubic6, open 16³ lattice; ascending neighbors, fixed external divisor 6.
  Inherit Experiment 0003/0002 omega, damping, conductance, internal four-neighbor
  coupling 0.6/4 and external 0.4/6 exactly. No parameter tuning or GPU physics.
- Only independent channel 1 is active. Scalar u/v storage for this channel is
  exact because the other three channels have no forcing or cross-channel links.
- Same SplitMix64 palette generator, seeds 7,19,41; heterogeneous and homogeneous
  all-red configurations. No rearranged or Cubic26 arms in this bounded study.
- Source A=(6,8,8), ID2182; B=(10,8,8), ID2186; module27=(3,3), channel1.
  Midpoint M=(8,8,8), ID2184. Sources are four graph/Euclidean units apart.
- Four independently simulated arms per configuration/seed: A only (+1 initial
  displacement), B only (+1), AB+ (+1,+1), AB- (+1,-1). Initial v=0 everywhere;
  no subsequent forcing. AB- is opposite initial polarity, not an ongoing tone's
  phase shift. Initial sum(u²)=1 for singles, 2 for duals; these are not matched
  total-energy experiments, and squared displacement is not physical energy.
- dt=0.1, **1024 steps / duration102.4** (6.4× Experiment 0003 duration).
  Six primary groups = 24 arm cases. Half-step dt=0.05,2048steps, same duration,
  for seed7/both configurations = two further groups/eight arm cases.
- Two zero controls (heterogeneous seed7, one per dt) and one 160-step center
  impulse regression against unchanged 0003: **35 executions total per suite**.
- Repeat the complete suite into two fresh ignored folders. Same-machine
  byte-identity is tested; no cross-platform identity claim.

## Every-step evidence and fixed decisions

For all four arms, measure max|u|, max|v|, sum(u²), sum(v²), nonfinite counts;
per-neuron max-module peak and first arrival ≥1e-6, plus cumulative peak/reach at
physical time16 and102.4. Log signed midpoint module27 u/v for A,B,AB+,AB-.
Retain all-tick global data and midpoint traces. No comparison of max|u| fields
is allowed to stand in for addition of signed module state.

Compute full-field residuals **each tick**, for u and v separately:
max|AB± - (A ± B)|, and normalized L2 residual. Absolute acceptance:
max residual <=1e-10 × max(1, max over field |A|+|B|), for both fields/signs.
Export signed dot(A,B), sum(AB+²)-sum(A²)-sum(B²) and minus counterpart;
expected cross terms are +2dot and -2dot. These are squared-field identities,
not conserved-energy or nonlinear-interaction measurements.

A measurable midpoint overlap requires both single-source midpoint neurons'
max-module |u|≥1e-6 at the **same tick**. Log first simultaneous crossing or
explicitly absent. Do not tune source spacing or thresholds if overlap is weak.
Reinforcement/cancellation is assessed against the signed single-source sum;
heterogeneous paths need not cancel perfectly at the geometrical midpoint.

Numerical screen: every step, every active module, finite u/v and |u|,|v|<=100.
A breach stops/retains the group as invalid; no clipping. Palettes must remain
byte-identical. Exact zero controls must remain zero. Finite horizons do not
prove general stability or learned persistence.

At matched physical recording times compare dt0.1/0.05 full signed u/v fields
using relative combined L2 error sqrt(sum(delta_u²+delta_v²)/sum(u_half²+v_half²)).
Exclude t=0 (zero denominator handled explicitly). Convergence screen passes
only if every retained nonzero-time checkpoint in each arm is <=10%; report
failures and exact maxima. This sensitivity screen does not prove convergence
order or resolve Experiment 0002 resonance indeterminacy.

## Recording, observation and visual checks

Record physical times **0,0.8,1.6,3.2,6.4,12.8,16,25.6,38.4,51.2,76.8,102.4**
(base ticks0,8,16,32,64,128,160,256,384,512,768,1024), all four arms/group.
New bounded `QR4FLD01` format, version1, includes topology/configuration/seed,
source IDs, arm, dt, exact step, actual palettes, full active-channel u/v and
trailing FNV corruption checksum. Provenance identifies 0004; never mislabel as
0003 or synthetic. SHA-256 file ledger supplements noncryptographic FNV.

Independent Python decodes all frames, verifies checksums/ledger, signed-field
superposition and cross terms, persistent palettes, coverage and zero controls,
and recomputes matched-time convergence. All-tick full-field checks are Rust
measurements; independent full-state recomputation is at the recorded times.

Viewer input is replay-only for 0004; LIVE controls must not start a single-source
0003 engine under a dual-source label. Header names 0004, both sources, arm, dt,
time, frozen configuration. Selection shows the real internal palette and signed
module27 fields. Observe central Z=8 plane; activity magnitude is not a signed
superposition test. Provide a signed central-plane comparison figure/animation
and actual native replay captures on Raphael. Do not manufacture stronger waves
with hidden per-frame gain. Fixed, labeled plotting scales may differ from the
legacy viewer's fixed activity scale.

## Delivery and boundaries

Separate Rust runner, analyzer, tests and results report; preserve old experiment
files/hashes. Run root severe CI gates and isolated viewer checks, inspect native
rendering, document observed failures, update README/progression/HANDOFF. Record
commit/push/hosted CI separately. Large recordings and movies stay ignored.
No plasticity, persistent interaction trace, memory, resilience, resonance peak,
physical-fluid or topology-superiority result is sought or presumed here.
