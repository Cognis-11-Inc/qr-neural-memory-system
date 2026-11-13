# QR6 observation formats, schema 1

All physics and times are nondimensional, CPU f64. Display spacing is absent
from scientific data. FNV and SHA hashes provide integrity, not authentication.
Registration is frozen in `578870c`, `EXPERIMENT_0006.md`.

## Full signed fields

`*.q6f`: magic8 `QR6FLD01`, then little-endian fields at byte8:
`u64 version=1,u64 tick,f64 dt,u64 seed,u64 initial_configuration,u64 label_length,
u64 injection_module`. Header64 bytes, UTF-8 condition label follows (1..200).
Module is9 or27; dt is .05,.025,.0125; tick<=24576. Then4096*64 palette bytes,
262144 f64 u,262144 f64 v, trailing u64 FNV-1a over all preceding bytes.
Length64+label_length+17*262144+8. Palette0..4; finite fields, magnitudes<=100.
Ordering is stable neuron ID, then module=row*8+column. Configuration code is
initial class; actual P is authoritative after switching. Channel1 alone is
stored, three uncoupled zero channels are absent by exact model independence.

Main folder checkpoints are strictly increasing, maximum12. `switch/pre.q6f`
and `switch/post.q6f` share actual time8 and are stored separately. They never
form two successive playback ticks. Full restart states are retained at registered
checkpoints, not every tick. No interpolation is part of any measured record.

## Local histories and early probe stream

`before.palette`/`after.palette`:4096*64 raw0..4 bytes. `velocity-history.bin`,
`shape-history.bin`,`scores.bin`:262144 little-endian f64, neuron/module order.
They store V,H,S over[0,8] as defined in the registration. `derived/CASE` contains
all-module eligibility/ranks in `eligibility.csv.gz` and all4096 actual before/
after8×8 arrays/differences in `palettes.jsonl`.

`early.q6p`: magic `QR6PRB01`, then contiguous records from tick0 through16
(or horizon if earlier): u64 tick followed by6*64 pairs of f64(u,v). Probe IDs
in order0,2182,2183,2184,2186,2191. Record length6152 bytes. Times use the case's
recorded dt. This is a measured probe stream, not full-network per-tick fields.
Independent Python re-integrates histories for these6 neurons only; elsewhere
it independently checks decisions from the exported complete histories.

## CSVs, controls and provenance

`parameters.csv` identifies seed,class,dt,module,arm,budget,rule,end,amplitude,
actual initial energy. `metrics.csv` records every tick's kinetic/onsite/coupling,
exact cumulative loss,switch work,energy drift,global maxima and nonfinite count.
`probes.csv` records11 sites each tick, signed input-module/9/27 u/v,local energy
and external power. `switch/energy.csv` records pre/post energy per neuron and
explicit local switching work, with half-edge potential attribution.

`REUSED` stores an existing condition ID. Reused entries have no fabricated frames;
resolve that reference, preserving original provenance. `GATES.csv` distinguishes
short and long numerical decisions. `execution-ledger.csv` enumerates all190
logical conditions and COMPLETE/REUSED/GATED/NOT_RUN status. COMPLETE establishes
execution, not scientific success. Source copies, frozen rules, SHA manifests
and independent verification are separate from interpretation and hosted CI.

Energy matching changes only the explicitly reported initial stimulus amplitude.
It never normalizes an outcome. UNIT and MATCH must remain separate comparisons.
R3 preserves each neuron's palette histogram. The shuffled R3 position control
may use non-cardinal swaps; it is a control, not a registered adaptive rule.
