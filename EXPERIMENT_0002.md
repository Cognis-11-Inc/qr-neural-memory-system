# Experiment 0002: intrinsic vibration, transport, and resonance

## Frozen registration — 2026-11-12 16:32:35 UTC

Scope authorized: execute Stages T and R; preserve Experiment 0001 unchanged.
The proposal at `/home/cognis/Downloads/EXPERIMENT_0002.md` supplies the questions;
this document freezes the actual finite design **before code or outcome metrics**.
Stage P is deferred: there is no adaptive update, associative recall criterion,
semantic learning criterion, or resilience claim in this experiment.

## Model and initialization

The full network is 64 × 64 = 4,096 neurons. Every neuron owns its own 8 × 8
persistent array P, palette 0–4, and transient four-channel displacement u and
velocity v. CPU f64, no cross-channel mixing, no wrapping, zero flux on missing
links. All next u/v use the same old snapshot. No target or evaluator input
enters the equations. No persistent structure changes in T/R.

For internal module i, let n(i) count occupied cardinal internal neighbors and
e(i) count empty cardinal internal neighbors; boundaries have fewer neighbors.
For channel k = 1..4:

    omega(i,k) = 0.8 + 0.08*P_i + 0.04*n(i) + 0.12*[P_i=k]
    gamma(i,k) = gamma_factor*(0.08 + 0.01*e(i) + 0.02*[P_i!=k])
    g(i,k) = 0.5 if P_i=0; 1.0 if P_i=k; 0.75 otherwise
    a_i = -omega_i^2*u_i - gamma_i*v_i
          + (0.6/4)*sum_internal g_i*g_j*(u_j-u_i)
          + (0.4/8)*sum_external g_i*g_j*(u_j-u_i) + s_i
    v_next = v + dt*a
    u_next = u + dt*v_next

Internal links join four cardinal modules; external links join matching module
coordinates in eight neighboring neurons. These two spatial scales are distinct.
All quantities and time are nondimensional, mass is 1. Baseline dt=0.1,
gamma_factor=1. No clipping or dynamic normalization. The integrator is
semi-implicit Euler; half-step comparisons test discretization sensitivity.
Maximum omega is bounded by 1.4 from these coefficients; this bound alone is
not a numerical stability proof. Color and local shape both enter intrinsic
frequency, damping, and coupling. No neuron-ID frequency offsets are allowed.

Seeds **7, 19, 41**, SplitMix64 as in Experiment 0001 (separate implementation).
Generate a full 4,096 × 64 array, row-major: each next u64 modulo 5 gives P.
Compare **heterogeneous**, **homogeneous** (every module red/1), and
**rearranged** (sort each heterogeneous tile excluding module 27; keep module 27
fixed). Rearrangement preserves each tile's full histogram and the drive module's
color, while altering spatial arrangement. Configurations are exported before
and after trials and must match exactly. Reset erases u/v and diagnostic traces.

Channel independence permits updating only the driven channel: all four u/v
fields exist, and the other three remain exactly zero by the equations. This is
an explicit exact-zero optimization, not a coupling approximation. No amplitude
threshold is used to prune propagation.

## Stage T: full-network usable transport

Run all 3 seeds × 4 channels × 3 source neurons × 2 stimuli = **72 trials** on
heterogeneous full networks. Source neurons: center (32,32), edge (0,32),
corner (0,0). The source module is (3,3), index 27. Reset between trials.
Duration **16**, dt=0.1 (160 steps). Two inputs:

- Impulse initial condition: source u=1, v=0, all other u/v=0; no forcing.
- Burst: initially zero; source sinusoidal drive at angular frequency pi/2
  for 0 < time <= 8, then zero through time 16. Normalize sampled input so
  `sum(s^2*dt)=1`; record the actual scale and all input samples.

Observe the source module in all valid cardinal/diagonal directions at graph
distances 1,2,4,8,16, plus distance 0; Euclidean distance is recorded separately.
Also observe internal modules 28,29,63 in the source neuron (internal Manhattan
distances 1,2,8). Observe every timestep including initial state. There is no
instant global correspondence: an update can propagate only one module/neighbor
graph edge. A threshold arrival time is an operational detectable arrival, not
the first mathematical nonzero or a physical wavefront speed.

Report peak absolute u, RMS u, RMS v, first |u|>=**1e-6** arrival, source-normalized
RMS transfer, diagnostic sum(u²) and sum(v²), worst amplitude and nonfinite counts
at **every** step; export traces for independent spectra and phase calculations.
Numerical detectability is separately |u|>=**1e-14**. No stochastic noise is
injected: the usability thresholds do not establish a practical real-world SNR.
The old Experiment 0001 1e-12 decoder is not the transport criterion.

Stage T positive at a **stated distance of 4 graph hops** requires every valid
cardinal observation in each center-source trial (all seeds/channels, both
stimuli) to cross 1e-6 and have RMS/source RMS >=1e-3, while zero-drive controls
stay exactly zero and numerical screens pass. Otherwise report failure or
indeterminate numerical validity, while retaining each distance/location's data.
Edge/corner and diagonals are reported separately; do not infer Euclidean
isotropy from equal weights. Ratios at cardinal/diagonal equal graph distance
compare different Euclidean distances and are labeled accordingly.

Run a full-network homogeneous, zero-state/zero-drive 16-time-unit control
(seed 7, center, channel 1). Configurations differ in R as detailed below;
T does not sweep all configuration classes.

### Historical Experiment 0001 reference

Preserve its engine, runner, analysis, specification, results, and raw evidence.
Copy the existing verified `results/experiment_0001-final-a/impulse.csv` into the
new evidence folder with its source/commit provenance when present. It is an
**unchanged historical reference**, not a matched new comparison: the old initial
previous-amplitude condition, integrator, step-time convention, restoring force,
and forcing differ. Its private runner cannot expose a matched transport probe
without changing the frozen source. Make no quantitative superiority claim over
that historical run. No reimplementation is mislabeled as the original engine.

## Stage R: configuration-sensitive frequency response

Use angular frequencies `2*pi*b/64`, bins **[4,8,12,16,20,24,28]**. Duration **64**;
burn-in **32**, measurement window (32,64], dt=0.1. Drive only module 27 and one
channel, `s=A*sin(omega*t)` at each step's end time. Normalize the full sampled
drive so sum(s²*dt)=1. Integer bins also give equal nominal amplitude across
frequencies (up to floating-point roundoff); hence off-resonance tones have both
matched energy and amplitude. Report the actual input energy/RMS, not assumptions.

For every seed/configuration/channel/frequency, compare:

1. **scalar module**: internal and external coupling disabled; intrinsic omega
   and gamma still derive from its full persistent tile;
2. **isolated QR neuron**: all 64 internal modules coupled, no external links;
3. **3 × 3 coupled diagnostic network**: nine separate QR neurons, using the
   central 3 × 3 tiles from that seed's full configuration. Zero flux at this
   diagnostic patch boundary, drive central neuron. This finite patch is an
   explicit diagnostic fixture, not a replacement for the 4,096-neuron T network.

This is **756 baseline sweeps** (3×3×4×7×3). Also run:

- Half-step isolated-neuron sweeps, dt=0.05, same physical duration/input energy,
  all 3 seeds/configurations/channels/frequencies: **252** cases.
- Full 64 × 64 network, heterogeneous seed 7, channel 1, all seven frequencies:
  **7** cases; half-step bins 8,16,24: **3** cases. These explicitly limited
  full-network checks do not generalize spectral results to every seed/channel.
- Isolated heterogeneous seed 7/channel 1, gamma_factor=2, seven frequencies:
  **7** damping controls.
- Coupled 3 × 3 heterogeneous seed 7/channel 1: three-frequency multisine bins
  8,16,24 with phases all zero versus seeded independent phases. Both have
  the same full-window DFT magnitudes and input energy 1: **2** controls.
  Draw phases from a fresh SplitMix64 seed 7, using next-u64 modulo 1,000,000
  divided by 1,000,000 times 2*pi. These are single-source spectral controls,
  not evidence about spatial multi-source coherence.
- Zero-state/zero-drive, isolated homogeneous channel 1: **1** case.

Total R cases **1,028**. Keep every seed/frequency/control, not just the best peak.
All comparisons reset transients; plasticity and diagnostic eligibility are absent.

### R metrics and decisions

Primary gain = steady-window RMS displacement at the driven module divided by
steady-window RMS drive. Also fit complex single-frequency transfer and relative
phase using sine/cosine sums in the integer-period window. Multisine gain uses
total RMS; phase at each of its three components is independently analyzed.
Export raw drive/u/v; Python independently calculates DFT spectra, gains, phases,
energies, transport summaries and numerical-validity decisions.

A sweep has a resolved selective peak if its maximum is an interior sampled
frequency, exceeds both adjacent gains by **20%**, and exceeds the lowest
off-frequency gain by **2×**. Boundary maxima are unresolved/indeterminate, not
proof of no resonance. For intrinsic configuration sensitivity, require the
isolated-neuron heterogeneous vs rearranged gain curves' relative L2 difference
to exceed **5%** per seed/channel, preserving both histogram and drive-module
color. Report color/homogeneous controls and phases, not only maxima.

Numerical convergence: relative complex-transfer L2 difference between isolated
dt=0.1 and dt=0.05 sweeps <=**10%** per seed/configuration/channel, and peak-bin
agreement. Half-step full-network checks must differ by <=**15%** in complex
transfer at their registered frequencies. Larger differences make associated
spectral claims numerically **indeterminate**; do not silently refine dt or
change which grid is primary after inspection.

Stage R positive, within these fixtures, requires resolved selective peaks for
all isolated-neuron normal sweeps, configuration sensitivity for every paired
seed/channel, equal-energy errors <=1e-10, exact zero-drive response, no numerical
failure, and the stated convergence screens. Otherwise report which components
passed/failed/are indeterminate. A positive R result establishes finite
configuration-dependent spectral selectivity only, not memory or biological
resonance. Multisine/damping controls are reported even when the headline
criterion fails. Damping should reduce maximum gain; failure is retained.

## Numerical validity, output, tests, and preservation

At every step, screen **all** u/v for nonfinite values and |u| or |v|>100.
Stop an invalid trial, export its failure, and continue the registered suite;
do not label the suite complete without accounting for every scheduled trial.
Source force is deterministic; there are no arbitrary ID-dependent frequencies.

Fresh ignored `results/experiment_0002-*` directories contain this registration,
engine source, parameters, full configuration snapshots, all raw inputs/probes,
every-step global diagnostics, independent summaries and a complete trial ledger.
Reset must preserve P exactly; exported post-run structures must equal pre-run.
Compare two complete runs byte-for-byte on the same machine/toolchain. No
cross-platform bit-identity claim is made. Original proposal is retained as
source material in each evidence folder, not executed as an authority itself.

Test internal/external boundaries, local causality, synchronous traversal order,
u/v reset, shape and color dependence, independent channels, source-energy
normalization, all-step invalid detection, reproducible seeded configurations,
and step convergence against an independent scalar analytic check where practical.
Run all existing quality gates without altering Experiment 0001. Its preserved
file hashes are checked before delivery. Authored design, measured results,
software checks, commit/push, and hosted CI are reported separately.

## Amendments

None at registration. Any later correction must be timestamped, explain whether
outcomes had been viewed, and retain unsuccessful trials. Stage P requires a
new, separate registration and is outside the present authorization.
