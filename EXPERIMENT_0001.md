# Experiment 0001: a reconfigurable medium and a local baseline

## Registered design (written before implementation/results)

This compares two deliberately small candidate rules, not established memory mechanisms.
The supplied `EXPERIMENT_0001_CODEX_BRIEF.md` is the task basis. Its 64 × 64
network is adopted **for this experiment**; earlier project documentation left
geometry unspecified. Network geometry is separate from each neuron's internals.

### State

4,096 neurons occupy a 64 × 64 grid without wrapping. Each owns an independent
8 × 8 persistent array `p[i,m]`. Palette: 0 empty, 1 red, 2 green, 3 blue,
4 yellow. Shape is the arrangement of occupied modules; module position is
preserved, even between arrays with identical histograms. Initial arrays are
all empty. A nonempty learned module encodes a color transition, rather than
being interpreted directly as the target color.

Each neuron also has an 8 × 8 transient signal array, initially empty. No
pair table, target buffer, learned edge weights, or hidden persistent activations
are part of the substrate. Clearing signals retains only the internal arrays.
Persistence here means across training/recall resets; snapshots also export
these arrays to disk for inspection.

### Messages, time, and boundaries

A message is the sender's previous 64-module signal array. At each cycle,
for each module, count nonempty colors among the eight valid neighboring
neurons. Select the most frequent color, breaking ties by lowest color ID;
no votes means empty. The neuron itself is excluded. No wrapping, global
broadcast, or instantaneous correspondence is used. A signal advances at most
one Chebyshev-neighbor hop per cycle. All next signals and configurations
are derived from one previous-state snapshot and committed together.

Cue injection overwrites the next signals of a fixed 4 × 4 input patch
(rows 30–33, columns 0–3) after the neighbor calculation. It is held throughout
training and inference. This anchoring is an external stimulus, not a broadcast.

### Candidate local plasticity

Let `v` be a module's neighbor-vote color and `t` its local teaching color.
Only the fixed 4 × 4 output patch (rows 30–33, columns 60–63) receives teaching
stimuli. Teaching is external only during training; other neurons receive none.

For a taught module with `v != 0` and `t != 0`, set:

    p_next = 1 + ((t - 1) - (v - 1)) mod 4

For any other taught module, set `p_next = 0` (last-experience overwrite).
Untaught neurons retain their persistent arrays. With learning disabled all
arrays remain unchanged. For signal transmission, use the **previous** `p`:

    if v == 0: next_signal = 0
    else if p == 0: next_signal = v
    else: next_signal = 1 + ((v - 1) + (p - 1)) mod 4

The intended mechanism is that a repeated local cue color might be transformed
into its co-presented teaching color after a reset. This is not guaranteed:
neighbor voting can mix cue identity, transformations can compound within the
output region, and an empty trace passes signals rather than suppressing them.
The rule cannot learn output occupancy at a module with no incoming cue signal.
These are explicit potential failure mechanisms. No training target is copied
directly into inference or retained outside the per-neuron transition arrays.
The complete target has 16 separate neuron tiles; no neuron owns that whole
association. Changes across these tiles may overlap for sequential associations.

### Stimuli and fixed schedule

Registered seeds: **7, 19, 41**. Use SplitMix64 with wrapping u64 arithmetic:
increment state by `0x9e3779b97f4a7c15`, xor-shift/multiply by
`0xbf58476d1ce4e5b9`, then `0x94d049bb133111eb`, and final xor-shift.
For each seed generate A, B, C, D and unrelated U, in that order. Each consists
of 16 independent 8 × 8 tiles. For each module consume one random u64, set
empty if its value modulo 4 is zero; otherwise color `1 + ((value >> 8) mod 4)`.
All tile/module traversal is row-major. These are synthetic patterns, not
standards-compliant QR codes or claims about natural data.

Train A → B for **80 cycles**, starting with empty signals. Clear every transient
signal; retain internal arrays. Recall with A's entire tiles **1, 3, 5, …, 15
missing** (8 of 16 missing input tiles), held for **80 cycles**. Decode the final
signals directly from the output patch, in row-major tile/module order.
Expected B is consulted only by the evaluator after inference. There is no
search over intermediate times or target-dependent stopping/decoding.

Then clear signals, train full C → D for 80 cycles on the same persistent
substrate, reset, and separately recall incomplete A and incomplete C. Training
order, durations, cue removal and readout remain fixed for every seed.

### Controls and ablations

Run each seed with normal learning, frozen configurations, color ablation,
and shape ablation. Color ablation maps every nonempty stimulus module to red.
Shape ablation sorts each stimulus tile's 64 palette values, preserving its
histogram but removing the original spatial arrangement. Apply ablations to
both cue and teaching stimuli, and to unrelated cues; no additional compute
or memory is given. Always score against the **original** B/D targets, explicitly
measuring fidelity lost by removing input information. These ablations do not
match information content; they are diagnostic interventions, not proofs that
color or shape is necessary. The persistent-state readout transformation remains
unchanged; no original target is used to reverse an ablation.

For each variant measure untrained incomplete A → B; trained incomplete A → B;
trained incomplete U → B; A → B after C → D; C → D after C → D; and unrelated
U → D after C → D. Frozen configuration is a complete no-plasticity baseline.
All recalls reset transient signals and never modify persistent arrays.

### Measurements and preregistered decision

Each output comprises 1,024 colored modules. Report:

- Full palette mismatch / 1,024.
- Occupancy mismatch / 1,024 (shape error).
- Color mismatch on target-occupied modules / number of target-occupied modules;
  missing predicted color counts as wrong. For an empty target this metric is
  zero, alongside the full and occupancy errors that still penalize false output.
- Changed persistent neurons/modules vs initialization and vs pre-C→D state;
  exact changed neuron IDs, bounding box, and full before/after arrays.

For **each** registered seed, require trained A → B palette error <= 0.10,
and improvement >= 0.15 over both untrained and unrelated controls, before
calling this candidate a positive intact-association result. Require the same
conditions for post-training C → D, and A → B error increase <= 0.05 after C → D
for sequential retention. Report every seed/variant and criterion failure;
do not tune thresholds, select successful seeds, or reinterpret a negative run.
These finite synthetic trials would still not establish general learning.

No lesion experiment is scheduled here. R1 resilience remains untested even if
intact retrieval meets these criteria. Distributed configuration changes alone
do not establish distributed causal storage or resilience.

### Reproducibility and delivery

Rust owns execution; no external engine dependencies are required. The runner
will write parameters, raw metrics, trial verdicts, change locations, and complete
u8 persistent-array snapshots in an ignored local output directory. Snapshots
are 4,096 × 64 bytes in neuron-row-major/module-row-major order; palette 0–4.
Transient arrays are intentionally not persisted as learned state. Compare
repeat outputs byte-for-byte. Tests cover local state independence, shape/color
distinction, synchronous traversal independence, boundaries, finite-speed
propagation, reset, determinism, controls, and sequential mutation.

## Pre-implementation amendment: two-scale ripple arm

After the initial specification and before implementation or data collection,
the user introduced the reconfigurable-medium hypothesis: information propagates
as local disturbances within and between neurons; experience modifies the
persistent medium, influencing future propagation. This amendment adds the
primary **ripple** arm, retaining the rule above as the **local** baseline.
Every seed and ablation is run for both arms. No empirical tuning has occurred.

Each internal module has four independent f64 transient amplitudes `u[k]` and
four previous amplitudes `u_prev[k]`. Channels correspond to stimulus colors;
amplitude and persistent color are distinct quantities. Current-minus-previous
amplitude represents momentum. Internal coupling uses four orthogonal neighboring
modules, without wrapping. External coupling connects the same module position
in each of the eight neighboring neurons. Thus the model explicitly contains
both internal-module propagation and inter-neuron propagation; it is not a
fluid simulation. External links are a modeling choice, not a claim about
physical distance or port wiring.

For persistent module color p and channel k (1–4), define conductance:

    g(p,k) = 0.25 if p=0; 1.0 if p=k; 0.5 otherwise
    w(i,j,k) = g(p_i,k) * g(p_j,k)
    u_next[i,k] = 1.9*u[i,k] - 0.9*u_prev[i,k]
                 + 0.2/12 * sum_j w(i,j,k)*(u[j,k]-u[i,k]) + s[i,k]

There are at most 12 links (4 internal + 8 external). Missing links contribute
nothing; the divisor remains 12. The pairwise weights are symmetric, nonnegative,
and at most 1. These choices constrain the explicit step but are **not** a
substitute for numerical checks. Boundary behavior is zero flux on missing links.
Use no clipping, amplitude normalization, automatic gain control, or hidden state.
At each nonempty input module inject `s=0.02` in its stimulus-color channel on
every one of the 80 cycles. Signals still advance at most one graph edge per
cycle; there is no instantaneous diffusion. The local baseline's held cue and
the ripple arm's forcing share duration and pattern, but their numerical dynamics
and memory/compute costs differ; report this limitation.

Decode a raw module by its largest positive channel exceeding **1e-12**, ties
resolved by lowest channel ID. No positive channel means empty. Plasticity uses
the raw local previous amplitude's decoded color in place of the baseline's
neighbor vote `v`; the transition update equation and teacher patch are unchanged.
Final output applies the stored color transition to that raw decoded color.
Persistent color and shape therefore affect both propagation conductance and
decoding. During training, both amplitude steps and new configurations use the
old snapshot; learning neither changes the current step's weights nor reads
future signals. These are experimental choices, not a validated association rule.

For **both** arms, refine the ablations to remove persistent-state information as
well as stimulus information: color ablation additionally maps every nonempty
learned module to red after each synchronous learning step; shape ablation
additionally sorts each learned internal array after each learning step.
Sorting erases the learned arrangement while retaining palette counts. It creates
a canonical artificial arrangement, so it is a diagnostic rather than a matched
alternative physics model. Frozen configurations remain empty. Use the same
incomplete-cue selection and original-target scoring for all arms.

Before association interpretation, run an impulse stability diagnostic with
learning disabled: one unit-amplitude impulse at the center module of a central
neuron, zero previous amplitude, no subsequent forcing, for **160 cycles**.
Log maximum amplitude, sum of squared amplitudes, sum of squared momentum,
nonfinite counts, and cardinal/diagonal propagation differences at cycles 0, 1,
2, 4, 8, 16, 32, 64, 160. Check no nonfinite amplitudes and peak amplitude <= 10;
these are finite-run screens, not a proof of stability or isotropy. Eight-neighbor
coupling does not imply isotropic Euclidean propagation. Test finite speed, local
coupling dependence on shape/color, damping versus gamma=0, and traversal order.

All acceptance thresholds above are unchanged. Record any additional failure
(including numerical failure or near-total extinction at the output) explicitly.

### Evidence layout

`metrics.csv` contains seven conditions per seed/arm/variant, including the
additional untrained C → D baseline. `outputs.bin` concatenates the raw 1,024-byte
outputs in exactly that CSV's row order (excluding its header).
`seed-N-stimulus-{a,b,c,d,u}.bin` contains each original 1,024-byte pattern.
Per-trial `initial`, `ab`, and `cd` snapshots contain the 262,144 persistent
module values; `changes.csv` lists changed neuron coordinates and module counts.
`activity.csv` records final whole-network/output-region absolute amplitude peaks
and nonfinite counts for each recall. The impulse `stability.json` additionally
screens every cycle for the registered finite-run bounds, not just logged sample
cycles. `verdicts.csv` records the unmodified acceptance decisions. A `COMPLETE`
marker means all 24 trials executed, not that their scientific criteria passed.
The runner exports its specification and engine source. Numerical replay is
claimed only for byte comparisons on the same machine/toolchain, not across all
floating-point implementations.
