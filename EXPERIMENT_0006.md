# Experiment 0006: morphological freedom and numerical reliability

Registered 2026-11-13 (America/Toronto), before implementation/confirmatory runs.
The registration Git commit and file SHA-256 identify the freeze. No pilot or
calibration outcomes were viewed. User's Downloads command is a proposed brief;
this document freezes the actual equations, matrix and decisions. No amendments.
Baseline main77be6ef; inventory `docs/EXPERIMENT_0006_INVENTORY.md`. Preserve all
0001–0005 scientific sources/specifications/results and raw evidence. Principle
002 is proposed. No associative memory, cognition, R1 resilience or novelty claim.

## Frozen physical model

4096 independent persistent8×8 palettes P=0..4; serial CPU f64, open16³ Cubic6,
ID=x+16(y+16z). Internal cardinal8×8 graph and external matching-module graph
are distinct. Active independent channel1 only; other three remain exactly0.
Use M5 coefficients, edge construction order, split integrator and energy exactly:

    omega_i=.8+.08*P_i+.04*occupied_internal_neighbors+.12*[P_i=1]
    gamma_i=.08+.01*empty_internal_neighbors+.02*[P_i!=1]
    g_i=.5 if P_i=0;1 if P_i=1;.75 otherwise
    k_internal=.15*g_i*g_j; k_external=(.4/6)*g_i*g_j
    du=v; dv=-omega²*u-gamma*v+sum_j k_ij*(u_j-u_i)
    E=sum_i(.5*v_i²+.5*omega_i²*u_i²)
       +sum_undirected .5*k_ij*(u_i-u_j)²

Exact half damping; velocity-Verlet stiffness kick/drift/kick; exact half damping.
Synchronous force evaluations use unchanged u; no clipping, pruning, physical
coefficient tuning, forcing after initialization, GPU physics or parallel updates.
Loss is exact damping-substep kinetic loss. Drift=E-E0+loss-switch_work.
Every module finite and |u|,|v|<=100, otherwise stop/retain FAILED evidence.
At every synchronous palette commit, hold u/v fixed, rebuild all coefficients
and symmetric links, record pre/post total and per-neuron energy and work.
Adaptive work is externally actuated; no energy-neutrality claim.

Same SplitMix64 palettes from unchanged lattice initializer; heterogeneous7/19,
one homogeneous all-red7. Source A2182=(6,8,8), B2186=(10,8,8), M2184. Source
module27 or9 as explicit factor. Reinitialize all transient/history/audit arrays
for every independent run. No special neuron ID in adaptive laws.

## M6-N: fixed-grid numerical gate first

Three fixtures: homogeneous7, heterogeneous7, heterogeneous19, AB+ at module27,
initial A/B u=+1, initial v=0. Each dt .05,.025,.0125 through307.2:9 runs.
Three heterogeneous7 zero controls, same grids through32:3 runs. Total12.
No extrapolation from partial horizon. Checkpoints0,.8,3.2,8,16,32,51.2,102.4,
153.6,204.8,256,307.2 (all exact integer-grid alignment). Stream every tick
energies/loss/drift/maxima/nonfinite and signed probes. Full signed snapshots only
at checkpoints. Serialize all actual P, never a texture or histogram surrogate.

At every nonzero checkpoint compare unaligned full signed u/v norm relative to
finer vector, denominator floor1e-30, and absolute RMS over2*262144 fields.
At times<=102.4 require .05/.025<=5%, .025/.0125<=1.5%; at ALL later checkpoints
require<=10% and<=3%. Ratio fine/coarse<=.35 if coarse>1e-10. All three fixtures
must pass the applicable window. Energy drift max|drift|/max(E0,1e-12)<=1%
all grids; zeros exact0 and frozen P unchanged. These limits are deliberately
informed by M5's known observations, not independent validation of M5. Failure
is FAILED, never retuned. Short downstream time32 requires these tests through32
only; no long inference if long gate fails. Analytic homogeneous uniform-mode
unit fixture separately checks refinement of this integrator, not new physics.

Diagnostics: every-checkpoint errors, source/midpoint/boundary waveform max
absolute disagreement at matched ticks, strictly local |u_source| peaks>=1e-6
with separation>=1 time unit, and nearest peak-time differences (absence reported).
Peak alignment never replaces unaligned field acceptance. No phase-adjusted pass.

## M6-D: diagnosis, no adaptive changes

Seeds7/19 × modules27/9 × unit displacement or equal initial mechanical energy
E*=1:8 runs, dt.025,time16. Two seed-specific zero controls at module27:2 runs.
All require N time32 pass. Integrate BOTH V_m=integral(v_m²)dt and H below over
[0,8], trapezoidal physical-time quadrature. Export all262144 V/H/S/ranks and
above-threshold counts; signed all64 modules at probes every tick in a versioned
binary stream plus selected-module CSV. Store P unchanged. M5-diagnostic argmax
uses V, threshold1e-8, lowest index ties; compare drive coordinate27 versus9.
Equal-displacement is descriptive; energy-matched amplitude=sqrt(1/E_unit),
require energy finite positive and relative mismatch<=1e-12. No normalization of
outcomes. Frozen no-physics regression at seed7/module27/unit/dt.025,time8/16
must match M5 u/v/history/P and its single-module edits where histories align.

## M6-M: separate registered local rules

One commit at exactly8, exposure through32, dt.025. All modes accumulate the
same histories on the same old transient states. Before/after commit fields and
all4096 actual8×8 palettes exported. Zero minimum edits; no target or reward.

V_m=integral v_m² dt. H_m=integral[v_m²+.25*avg_existing_internal_q(u_m-u_q)²]dt,
with trapezoidal endpoint weights over[0,8]. S_m=H_m+.5*avg_existing_q H_q,
computed simultaneously from own64 H. Empty-neighbor division never used;
all8×8 modules have2..4 internal neighbors. Smax is own-neuron maximum.

R0: frozen, same histories/commit schedule, no edit.
R1: historical V argmax, lowest-index ties, V>=1e-8; cyclic P->(P+1)%5.
R2: S>=1e-8 AND S>=.05*Smax. Rank score descending, module index ascending;
select at most8 distinct modules, recolor each cyclically. No minimum count.
R3: own internal undirected cardinal edges with unequal old palette values;
both endpoint S>=1e-8 and at least one>=.05*Smax. Rank sum S descending, then
ascending (m,n) with m<n; greedily choose at most4 edges sharing no modules.
Synchronously swap each selected pair using old P. Exact per-neuron histogram
preserved; changed positions=2*swaps. No substitute edits if candidates absent.
R2_DISABLED: compute eligibility but commit no edits, identical history schedule.

Primary matrix: seeds7/19 × modules27/9 × budgets UNIT/E1 × R0/R1/R2/R3 =32.
Zeros: seeds7/19 × rules R0/R1/R2/R3, module27:8. Disabled: seeds7/19 × modules27/9,
E1,R2_DISABLED:4. Fine confirmatory exposure: seed7/module27 and seed19/module9,
E1,R2/R3,dt.0125:4. Total48 exposure executions. Historical R1 unit seed7/module27
at.025 compared with M5's identical physical fixture. One preselected repeat:
seed19/module9/E1/R3/.025 into a fresh root, byte-identical case files required.

Descriptive structural PASS per R2/R3: at least one neuron changes>=2 positions,
zero-input changes0, edits persist through transient reset, selected fine cases
retain>=2 edits and satisfy their short-grid diagnostics. Report complete0..8
position histograms, swap counts, eligibility rankings, source/near/remote spatial
counts and palette-histogram changes. Different fine edit sets are not suppressed:
require >=90% Jaccard position agreement on selected paired cases, and .025/.0125
field error<=1.5% at16/32 for each adaptive exposure; otherwise INDETERMINATE.
Rules may fail or yield only coordinate-local edits. No claim of rich shape or
learned association from count alone. Disabled controls must match R0 E1 bytes.

## M6-C: reset, masks, displacement and energy-matched ablations

Use E1-exposed Pafter for BOTH rules, BOTH seeds and BOTH modules at primary dt:
8 base fixtures. Fine4 registered exposure fixtures also undergo all ablations.
For each12 base fixture: variants VIRGIN,SOURCE_ONLY,OFF_SOURCE_ONLY,ALL,
SHUFFLE_POSITION × input comparisons UNIT/MATCH =120 logical conditions through32.
No further plasticity. Explicitly reset u/v/V/H, loss/work/tick. SOURCE_ONLY
retains only A's Pafter; OFF_SOURCE_ONLY restores only A; ALL retains all.
Immutable Pbefore/Pafter are never changed while constructing variants.

UNIT: initial displacement1 at A/module. Actual E0 reported separately.
MATCH: target E0 is VIRGIN's energy at displacement1, amplitude=sqrt(target/
variant_unit_energy), require finite positive unit energy and <=1e-12 relative
matching error. This differs from E1 exposure, which uses target1. Publish all
amplitudes and energies, never normalize response traces. No dual re-exposure.

SHUFFLE_POSITION is a deterministic non-stimulus position control, preserving
changed-neuron IDs and per-neuron edit counts, hence exact source-distance counts.
Within each neuron order modules by SplitMix64(seed ^ (id*64+m) ^ 0x6006), then
module index. R2 applies cyclic edits at first K of that ordering (K=actual count).
Color-change histograms may differ; report mismatch. R3 constructs K/2 unequal
color pairs: repeatedly choose the two largest remaining color groups (ties color
ascending), choosing earliest-ranked module in each group, then swap. Maximum
possible unequal pair count is not reduced by this greedy color-count rule;
if K cannot be achieved, retain FAILED control with actual counts, no padding.
Pairs need not be cardinal in this non-stimulus control; label that distinction.
Palette histograms and K are exact for R3; no claim this control was trained.

VIRGIN reuse: primary C VIRGIN cases reference the corresponding M R0 UNIT
trajectory (both comparison inputs have amplitude1); no duplicate execution.
Fine VIRGIN runs once per seed/module fixture and is referenced for both rules
and budgets. Thus120 logical C IDs require98 new executions (64primary,34fine),
with22 explicit REUSED entries. Never relabel stored frames as a new simulation.
Full planned matrix: N12+D10+M48+C120=190 logical IDs,168 new primary executions,
plus one repeat=169. Reuse is a ledger reference, not an independent sample.

Compare signed fields at0,.8,3.2,8,16,32; field L2 and absolute RMS relative to
VIRGIN, separately source-neuron mask and remaining4095-neuron mask. Also compare
ALL versus SOURCE_ONLY and OFF_SOURCE_ONLY versus VIRGIN; no additive percent
contribution interpretation. Causal descriptive threshold>=1% on both16/32
OFF_SOURCE_ONLY versus VIRGIN (off-source observation mask), both budgets;
applicable selected fine pairs require effects agree within20% and unaligned
coarse/fine fields<=1.5% at16/32. If source-only nearly matches ALL (<=1% on
both times/off-source mask), say so. Report each fixture separately, no universal
positive unless all registered selected fixtures pass. Multi-module structure
and causal off-source necessity are separate gates, not memory evidence.

## Records, implementation, preservation and delivery

Dedicated QR6FLD01 schema1: metadata tick/dt/seed/configuration/condition/module,
actual palette/full signed u/v/FNV; SHA source/rule/file receipts. New modules
may copy/factor M5 equations but may not edit frozen M5 files. Unit tests compare
exact M5 paths at identical dt; no old frame schema is widened. Every tick global
energies/amplitudes/nonfinite and signed probe CSV. Selected early probes
0,A2182,2183,M2184,B2186,2191 also retain all64 signed u/v in QR6PRB01 stream.
Probe stream stores actual ticks, no interpolation. H/V/S and P arrays are binary
with documented little-endian dimensions; export all-neuron palette JSONL and
module eligibility CSV/ranks to portable analysis where practical.

Full checkpoints in normal runs: registered N set clipped to horizon, including
post-commit time8. Synchronous pre/post switch states in separate switch folder
(no duplicated time in replay sequence), per-neuron switch energy differences.
Python independently decodes/checks fields, reconstructs energy/edits, verifies
histogram/swap/mask/energy-normalization and numerical decisions. It is not a
second integration. Source-bound caches include all input integrity hashes.
Keep entire field arrays/probe streaming bounded in RAM; serial trials.

Freeze/commit before data; preserve hashes in new protected manifest. Ledger
records COMPLETE/REUSED/GATED/FAILED/NOT_RUN, with exact reason and source IDs.
Do not infer completion from green unit tests. Short failure gates D/M/C; long
failure forbids long interpretation. No retrospective criteria changes.

Viewer remains isolated, iGPU default,spacing1.8 unchanged. QR6 checkpoint replay
only, labeled rule/module/actual time, no old LIVE substitution. Authoritative
before/after P drives morphology overlay(count0,1,2–3,4+) and selectable panels.
Native captures/receipt compare P bytes,0replay engines and scientific hashes;
not FPS or cognition evidence. Graphics failure cannot change core results.

Run root and isolated viewer formatting/strict Clippy/tests/build/docs, Python
format/lint/tests/independent checks; inspect diff/targets. Update README,
progression,HANDOFF,new results/schema/figures. Push separate registration,
implementation/results commits,verify remote SHA and hosted CI. Portable ignored
AGENT_REPORT contains actual arrays/CSV/JSON/rules/source/ledger/hashes;optional
full signed archive separate. Preserve every failed and unexecuted condition.
