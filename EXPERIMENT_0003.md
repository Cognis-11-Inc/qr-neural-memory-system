# Experiment 0003: volumetric topology and scientific observatory

## Registration — draft clock 2026-11-13 03:32:54 UTC

Client-facing date: 2026-11-12 (America/Toronto). The Git registration commit
is the exact freeze checkpoint, preceding measurements. User authorized the
full 0003 scope, preserving 0001/0002. The [proposal](docs/EXPERIMENT_0003_PROPOSAL.md)
is source material; its outdated claim that 0002 was absent is superseded by
the verified checkout. No tuning of either prior experiment is allowed.

## Scientific model and topology

4,096 neurons, each owning 64 independent persistent palette modules arranged
8 × 8. Values 0–4 mean empty/red/green/blue/yellow. Four independent f64 u/v
channels are transient. Plasticity is disabled. Reuse the exact 0002 semi-implicit
Euler model and configuration generator, with dt=0.1, gamma_factor=1:

    omega(i,k) = 0.8 + 0.08*P_i + 0.04*occupied_internal_neighbors + 0.12*[P_i=k]
    gamma(i,k) = 0.08 + 0.01*empty_internal_neighbors + 0.02*[P_i!=k]
    g(i,k) = 0.5 if empty, 1 if P_i=k, 0.75 otherwise
    a_i = -omega_i²*u_i - gamma_i*v_i
          + (0.6/4)*sum_internal g_i*g_j*(u_j-u_i)
          + (0.4/D)*sum_external g_i*g_j*(u_j-u_i) + s_i
    v_next = v + dt*a; u_next = u + dt*v_next

Internal neighbors: four cardinal modules, no wrapping. All updates use one
old snapshot, commit together. Update only driven channel 1, while storing
four u/v channels and verifying inactive channels stay zero (exact independence).
Coefficients and operations follow 0002; the new adapter is independently
implemented because the frozen old engine is private. A measured legacy fixture
will check Grid2D8 regression without editing the previous engine.

Topology policies, ordered by ascending stable neighbor ID:

- **Cubic6**, primary: 16³, six face-sharing neighbors, fixed D=6.
- **Cubic26**, secondary: 16³, all surrounding offsets, fixed D=26.
- **Grid2D8**, regression: 64×64×1, original eight-neighbor order, fixed D=8.

Open boundaries; missing links contribute nothing. Never divide by actual
degree. Global maximum-degree normalization equalizes the nominal external
coupling budget, not per-edge strength or boundary response. No link-length
weighting: Cubic26 includes links of lengths 1,sqrt(2),sqrt(3); report their
counts and directional effects. No isotropy or fair 2D/3D superiority claim.
Mapping: id=x+nx*(y+ny*z); inverse round-trips every ID. All edges are reciprocal.

## Registered scientific conditions

Seeds **7,19,41**, SplitMix64; generate 4096×64 palette values using next-u64
modulo 5. Three classes inherited from 0002: heterogeneous; homogeneous all-red;
rearranged (sort each tile excluding module 27, preserving that module's color
and the full histogram). Source module 27=(3,3), channel 1. Duration 16,
160 steps. Source sites in 3D: center (8,8,8), corner (0,0,0), face (0,8,8),
edge (0,0,8). Common seeds/palette ordering do not make 2D and 3D geometrically
equivalent.

- **T1/T2/T4:** 2 cubic policies × 3 seeds × 3 classes × 4 source sites =
  **72 impulse runs**, initial source u=1, v=0, all else zero, no subsequent drive.
- **T3:** 2 cubic policies × 3 seeds × 3 classes × 2 tones = **36 runs**.
  Center source, initially zero. Angular frequencies pi/4 and pi/2, held for
  all 160 steps, forcing sampled at step-end time. Normalize sampled s so
  sum(s² dt)=1. These have equal amplitude and energy over integer periods.
  Analyze the final 8 time units for RMS gain and phase relative to the drive.
  These are two **fixed frequency probes**, not a sufficient new resonance sweep.
  Neither frequency is selected after outcome inspection or called resonant.
- **Zero drive:** homogeneous seed 7 center, one per cubic policy = **2 runs**.
- **Grid2D8 reference:** heterogeneous seed 7 center (32,32,0), same impulse,
  160 steps = **1 run**. Compare against existing 0002 case-0 evidence and a
  small curated initial-tick regression fixture; do not alter either old runner.

Total **111 cases**. Repeat the complete suite into two fresh ignored folders.
No parameter or acceptance amendment after seeing outcomes. Preserve invalid
runs as failures and account for every scheduled condition.

## Measurements and scientific interpretation

Every tick: global peak |u|, sum(u²), peak |v|, nonfinite counts, and source
module signed u/v. Per neuron: max absolute module amplitude and velocity per
channel, module-27 signed u/v, sum(u²)/sum(v²), nonfinite count. Threshold arrival
is the first tick with neuron max|u|>=**1e-6**, not a physical wavefront speed.
Report peaks and arrivals by shortest graph distance and Euclidean distance,
source site, axis/diagonal direction and class. Uncrossed thresholds are absent,
not assigned the final tick as an arrival. Spectral phase is a finite-window
measurement, not coherent memory. Squared fields are diagnostics, not physical
conserved energy. Log degree histogram and undirected link-length counts.

Boundedness screen at **every step**: no nonfinite u/v and |u|,|v|<=100. A breach
stops that case and is retained; no numerical clipping or display gain feeds back
into physics. Zero-input cases must stay exactly zero. Persistent P must not
change. Different configuration-dependent results characterize these imposed
equations; they do not demonstrate adaptation, learning or emergence.

Report T1–T4 observations across all cases, including extinction/attenuation and
indistinguishable configuration conditions. Software acceptance is separate:
geometry, numerical/causal tests, replay, and viewer correctness gates. A completed
diagnostic topology experiment is not automatically positive transport, resonance,
memory or resilience. 0002's negative T and indeterminate R classifications remain.

## Versioned observation and replay boundary

Root engine has no graphics dependencies. Bevy is isolated in `viewer/` with its
own manifest and lockfile, pinned 0.20.0 from the verified official release/local
examples. No GPU simulation or parallel scientific updates. Viewer-specific
compilation is single-job; root severe CI remains headless.

Snapshot schema **QR3OBS01 / version 1** records tick, topology/dimensions,
seed, class, source/forcing, dt, oscillator model version/provenance and 4,096
stable neuron IDs/positions. Each neuron includes actual persistent 8×8 palette,
four max-module-|u| / |v| values, four signed module-27 u/v values, squared-field
diagnostics and a nonfinite count. Fields are derived **measured engine state**;
synthetic test fixtures must carry a different provenance label. No resonance
field is fabricated. Snapshot checksum is FNV-1a for corruption detection, not a
cryptographic authenticity claim; export SHA-256 evidence separately where used.

Record at ticks **0,8,16,...,160** (21 frames per case), with all-tick metrics
and arrival/peak maps. This is a deliberately bounded timeline, not every-tick
module-displacement recording. Replay reads data, never re-executes dynamics.
Require exact encoding/decoding and strict increasing ticks/provenance agreement;
reject wrong version/length/checksum and corrupted recordings gracefully.

## Viewer vertical slice and controls

Display all 4,096 individually addressable compact glyphs in a 3D volume;
do not instantiate every internal module. Orbit, pan, zoom, reset; X/Y/Z axes.
Keyboard ID entry or picking selects any neuron; selection shows exact ID,
coordinates, degree, numeric measured fields, actual 8×8 colored module panel,
and a bounded selected-neuron amplitude history. Named overlays:

- activity: max-module max-channel |u|, fixed explicit scale (linear 0–1 or
  log10 1e-8–1, labeled; never hidden auto-scale);
- persistent palette: separately labeled summary glyph, actual modules in panel;
- nonfinite: visibly flagged, recorded, never silently color-mapped.

Pause, single-step, bounded seek/replay and reset operate on explicit commands.
REPLAY selection is pure snapshot lookup; LIVE commands step the same engine
using fixed dt, independent of render delta/camera/scale. Rendering time controls
only command cadence. Node, selected-node edge and activity overlays independent;
slice X/Y/Z and shell/interior filters must not alter the authoritative graph.
Show source mode LIVE / REPLAY / SYNTHETIC TEST FIXTURE, tick and pause state.

## T5/T6 and acceptance evidence

- Unit tests exhaust geometry degrees/reciprocity/ID mapping, independent P,
  boundary/cause limits, snapshot-order invariance, reset, unchanged legacy
  adapter fixture, exact round-trip, corruption rejection and pure display
  selection/filter commands.
- Launch a normal native Bevy window on this workstation; inspect rendered
  output and report actual adapter/backend. Measure, do not assume, GPU choice.
- Automated witness sessions drive the real viewer through pause/step, seek,
  selection, slicing, overlay/camera changes, and LIVE versus REPLAY, at two
  frame caps (30 and 60). Export tick-indexed snapshot hashes and selected panel
  bytes/fields. Compare identical commanded ticks with headless case 0 and
  with each other. Automated witness is not claimed to be human playtesting.
- Saved replay launches without an engine; corrupted input exits with a clear
  error. Verify panel bytes for sample IDs 0, source, 4095 and sample ticks.
- Root format/strict Clippy/tests/docs/build/Python gates remain; viewer has
  separate format/build/test checks and may add a headless compile CI job.

Large recordings/screenshots/witness logs remain ignored under `results/`.
Small scientific report figures may be curated. Record exact failures, incomplete
gates, source hashes, commits/push/CI separately. Preserve 0001/0002 protected
source, spec and results files byte-for-byte. No new learning, plasticity, GPU
compute, dynamic topology, fluid fidelity, or unbounded recorder is in scope.
