# Experiment 0001 — measured results and failures

Date: 2026-11-12. Rules: [EXPERIMENT_0001.md](../EXPERIMENT_0001.md).

**Outcome: neither candidate demonstrated associative retrieval.** All 24 registered
seed × engine × variant trials failed the association and retention criteria.
Nine Rust tests and four Python tests pass; that is software evidence, not a
positive memory result. No lesions were run; Hypothesis R1 remains untested.

## Execution and provenance

Seeds 7, 19, 41; local and ripple engines; normal, frozen, color, and shape
variants; 80-cycle training/recall; half the input tiles missing at recall.
Each engine has 4,096 independent 8 × 8 persistent module arrays. The network's
64 × 64 geometry is an explicit Experiment 0001 choice from the supplied brief.
The user's subsequent ripple proposal was incorporated before implementation
or results, while retaining the simpler local rule as the comparison.

An initial complete run is retained at `results/experiment_0001-run-01`.
Instrumentation was then extended to log output-region amplitude and screen
all impulse cycles; no physics, stimulus, plasticity, threshold, seed, duration,
or acceptance criterion was tuned. Final runs are:

- `results/experiment_0001-final-a`
- `results/experiment_0001-final-b`

Both completed 24 trials and 168 inferences. Their exported files, including
sources, specification, parameters, snapshots, and raw outputs, compared
byte-for-byte equal using `diff -qr`. The independent Python analyzer recomputed
all metrics from raw outputs, checked persistent changes against snapshots,
and verified the acceptance decisions. Output-stream SHA-256:
`c6731221dc80e35f35d58a7d99676917d60a0c3808fda6a68cfb1967b0a4d1d9`.
These are same-machine/toolchain replays; cross-platform floating-point identity
was not tested. The initial run's scientific metrics/verdicts match the final
runs; its instrumentation/source files differ as described above.

## Normal-arm palette reconstruction errors

Errors are fractions of all 1,024 original target modules; lower is better.
Success required error <= 0.10 and >= 0.15 improvement over both untrained and
unrelated controls for each seed. Retention additionally required successful
initial learning and post-C→D A→B error increase <= 0.05.

| Seed | Engine | A untrained | A trained | U → B | A after C→D | C untrained | C trained | U → D |
|---|---|---|---|---|---|---|---|---|
| 7 | local | 0.834961 | 0.831055 | 0.824219 | 0.834961 | 0.805664 | 0.816406 | 0.809570 |
| 7 | ripple | 0.723633 | 0.723633 | 0.723633 | 0.723633 | 0.736328 | 0.736328 | 0.736328 |
| 19 | local | 0.816406 | 0.783203 | 0.803711 | 0.793945 | 0.790039 | 0.788086 | 0.786133 |
| 19 | ripple | 0.750977 | 0.750977 | 0.750977 | 0.750977 | 0.760742 | 0.760742 | 0.760742 |
| 41 | local | 0.814453 | 0.789062 | 0.798828 | 0.828125 | 0.791016 | 0.800781 | 0.821289 |
| 41 | ripple | 0.741211 | 0.741211 | 0.741211 | 0.741211 | 0.730469 | 0.730469 | 0.730469 |

The lower ripple palette error is **not** better retrieval: its output is entirely
empty, making its error equal to the fraction of occupied target modules.
It misses every occupied target color. The trained, frozen, untrained, and
unrelated conditions all yield the same empty output for a given target.
The local rule has small changes relative to controls but falls far short of
the registered requirements. For seed 7 its unrelated cue even scores better
than its correct cue; selecting a favorable time or seed would misrepresent this run.

## Shape and occupied-target color errors

Shape error divides occupancy mismatches by 1,024. Color error divides incorrect
or missing colors by the number of target-occupied modules. These are normal-arm
A→B results after training.

| Seed | Engine | Shape error | Occupied-target color error |
|---|---|---|---|
| 7 | local | 0.288086 | 0.769231 |
| 7 | ripple | 0.723633 | 1.000000 |
| 19 | local | 0.249023 | 0.711313 |
| 19 | ripple | 0.750977 | 1.000000 |
| 41 | local | 0.264648 | 0.722003 |
| 41 | ripple | 0.741211 | 1.000000 |

## All variants and persistent changes

A/B and C/D module-change counts are relative to initialization and the post-A→B
snapshot, respectively. All scientific pass flags are false. Unfavorable variants
are retained. Raw seven-condition tables and complete arrays remain in the local
run folders, rather than being replaced by selected examples.

| Seed | Engine | Variant | A trained error | C trained error | A/B changed neurons/modules | C/D changed neurons/modules |
|---|---|---|---|---|---|---|
| 7 | local | normal | 0.831055 | 0.816406 | 16/741 | 16/802 |
| 7 | local | frozen | 0.834961 | 0.805664 | 0/0 | 0/0 |
| 7 | local | color | 0.818359 | 0.808594 | 16/741 | 16/401 |
| 7 | local | shape | 0.806641 | 0.797852 | 16/738 | 16/352 |
| 7 | ripple | normal | 0.723633 | 0.736328 | 0/0 | 0/0 |
| 7 | ripple | frozen | 0.723633 | 0.736328 | 0/0 | 0/0 |
| 7 | ripple | color | 0.723633 | 0.736328 | 0/0 | 0/0 |
| 7 | ripple | shape | 0.723633 | 0.736328 | 0/0 | 0/0 |
| 19 | local | normal | 0.783203 | 0.788086 | 16/769 | 16/827 |
| 19 | local | frozen | 0.816406 | 0.790039 | 0/0 | 0/0 |
| 19 | local | color | 0.812500 | 0.804688 | 16/769 | 16/384 |
| 19 | local | shape | 0.819336 | 0.803711 | 16/764 | 16/219 |
| 19 | ripple | normal | 0.750977 | 0.760742 | 0/0 | 0/0 |
| 19 | ripple | frozen | 0.750977 | 0.760742 | 0/0 | 0/0 |
| 19 | ripple | color | 0.750977 | 0.760742 | 0/0 | 0/0 |
| 19 | ripple | shape | 0.750977 | 0.760742 | 0/0 | 0/0 |
| 41 | local | normal | 0.789062 | 0.800781 | 16/759 | 16/809 |
| 41 | local | frozen | 0.814453 | 0.791016 | 0/0 | 0/0 |
| 41 | local | color | 0.803711 | 0.823242 | 16/759 | 16/387 |
| 41 | local | shape | 0.797852 | 0.792969 | 16/757 | 16/247 |
| 41 | ripple | normal | 0.741211 | 0.730469 | 0/0 | 0/0 |
| 41 | ripple | frozen | 0.741211 | 0.730469 | 0/0 | 0/0 |
| 41 | ripple | color | 0.741211 | 0.730469 | 0/0 | 0/0 |
| 41 | ripple | shape | 0.741211 | 0.730469 | 0/0 | 0/0 |

For the local normal arm, changes are confined to the 16 taught output neurons:
rows 30–33, columns 60–63 (zero-based). IDs are 1980–1983, 2044–2047,
2108–2111, and 2172–2175. `changes.csv` records every location; full initial,
A/B, and C/D snapshots are each 262,144 bytes. The same output neurons are
modified again by C→D. The remaining 4,080 neurons are relay sites under this
rule, not observed learned traces. This narrow teaching layout is a limitation.
Changes at multiple sites demonstrate distributed **mutation**, not distributed
causal storage or a complete retrievable memory. Frozen configurations never
change. Color/shape ablations do not rescue retrieval; failed normal performance
prevents a claim that these ablations establish necessity of either variable.

## Ripple transport and stability diagnostic

Persistent color controls conductance by channel; occupied-module placement
controls local coupling, and persistent traces affect readout. Unit tests show
that color changes and shape relocation affect the numerical step. Amplitude and
previous amplitude are separate transient fields, reset before each recall.

For normal A→B recalls at cycle 80, output-region absolute amplitude peaks were
2.416338525161e-124 (seed 7), 3.085221863868e-124 (seed 19), and
2.384823026304e-124 (seed 41). These are far below the preregistered 1e-12 decoding
threshold, despite whole-network peaks around 5.1–5.7. The result is consistent
with inadequate usable signal reaching the remote teaching/output patch in this
window. There is no observed persistent change in the ripple arm's exported
learned configurations. We did not lower the threshold to manufacture a result.

The unforced 160-cycle impulse diagnostic screened every cycle: peak amplitude
5.234435563972, zero nonfinite observations, below the registered limit of 10.
Sampled squared-amplitude sum falls from 26.3171 at cycle 8 to 0.0223921 at
cycle 160; squared momentum falls to 1.46677e-6 at cycle 160. A unit test also
checks the damped versus undamped step for uniform initial velocity. Squared
amplitude is not claimed to be a physical conserved energy.

At cycle 32, one-hop cardinal and diagonal samples are 0.459673 and 0.398691.
At cycle 1 they are equal (0.00104167), despite different Euclidean distances.
These measurements expose lattice/directional effects; they do not establish
isotropy. Finite-speed tests verify immediate internal and external neighbors
respond while two-edge-away modules remain unchanged on the first step.
This is a finite empty-medium stability screen, not proof of stability for all
learned configurations, durations, forcing schedules, or parameters.

## Limitations and next hypothesis

- No working association, capacity result, distributed-memory claim, or resilience
  result follows from this experiment. Retention cannot be established when
  initial retrieval fails, even if an error happens to change little.
- The local rule stores color offsets at corresponding module positions. Neighbor
  voting mixes cue information, and the rule lacks a mechanism for learning
  missing output occupancy or routing arbitrary source shapes to target shapes.
- The ripple candidate fails first at usable remote transport under its fixed
  parameters. State mutation at nearby sites in tests does not fix that failure
  in the full-distance experiment.
- Baseline and ripple variants use the same trials and population, but have
  different transient storage, numerical scales, and compute costs. Ablations
  remove information and create canonical artificial shapes. They are diagnostics,
  not matched-budget proof of an architectural advantage.
- A/B and C/D are synthetic random tiles. There is no generalization or biological
  claim, and no QR-standard decoding or error correction.

The next bounded hypothesis should concern **usable transport before memory**:
with plasticity disabled, preregister impulse/forced-cue amplitude and channel
fidelity versus distance and time, using paired damping/coupling settings and
boundary checks. Any revised transport/plasticity rule belongs to a new registered
experiment. Then test cue identity and shape-sensitive local plasticity; do not
reinterpret this failed rule as evidence for learning.

## Verification and reproduction

Local checks passed: Rust formatting, strict Clippy, nine Rust tests, doctest
execution (zero examples), release build and rustdoc with warnings denied;
Python Ruff format/lint, four unit tests and byte compilation. Byte-identical
full-run replay and independent raw-evidence checks are separate from those tests.
Hosted CI must be checked against the final pushed commit; local checks alone
are not hosted evidence.

Use the README run/analyze commands with a fresh directory. Snapshots and raw
outputs are ignored by Git under `results/`. The runner exports exact parameters,
specification, engine source, original stimuli, outputs, changes and verdicts.
