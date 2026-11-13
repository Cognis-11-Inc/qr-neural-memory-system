# Experiment 0004 extension: 3,072 ticks, every-tick observation

Registered 2026-11-13 before collecting extension data. User explicitly requested
triple the maximum tick and see every tick to investigate whether the signal
dies away. Preserve the original 0004 registration, suite, sources and report.
This is an explicitly named extension of the currently viewed case, not a
retrospective change to the 35-execution experiment.

## Fixed scope and rules

One fixture: homogeneous/all-red seed7, Cubic6 open16³, 4,096 independently owned
8×8 palettes frozen throughout, AB+ with initial unit displacements at
A2182=(6,8,8), B2186=(10,8,8), module27/channel1. Initial velocities0, no later
forcing. Reuse the exact existing `propagation_0004::Medium` with dt0.1 and all
coefficients/update order unchanged. Integrate **3,072 steps, duration307.2**.
Record **every integer tick including0:3,073 observations**. Do not change damping
or coupling to obtain decay. Run twice into fresh ignored folders; compare bytes.

No new superposition sweep, dt-convergence claim, plasticity or memory test.
The original long-window timestep screen failed all8arms; extending dt0.1
further does not repair that failure. Decay describes this discrete trajectory,
not an established converged physical model. If original arm data are supplied,
compare full signed u/v/palette against its12checkpoints through tick1024.

## Measurements and meaning of decay

Every tick: global max|u| and max|v|, sum(u²), sum(v²), nonfinite counts, midpoint
signed module27 u/v and count of neurons with max-module|u|>=1e-6. Persistent
palettes must remain exactly unchanged; numerical screen requires all modules
finite and |u|,|v|<=100; stop and retain evidence on violation, without clipping.

Predefined quiet threshold: both global max|u| and max|v|<**1e-6**. Report the
last tick at/above either threshold and length of the remaining below-threshold
tail. Call a **sustained measured quiet tail** only if the final consecutive tail
has at least128ticks. This is not mathematical zero, an attractor/stability proof,
absence of all information, or learned memory. Also report exact final values
and zero/nonzero status. All squared fields are diagnostics, not physical energy.

## Bounded recording and playback

New `QR4DEC01`/schema1 stream, distinct from 0004's bounded full-module format.
Store fixed provenance/dt/seed/configuration/arm/sources and actual palettes once,
then every tick's stable-ID per-neuron fields: max-module|u|/|v|, signed module27
u/v, sum(u²)/sum(v²), nonfinite count. Other channels are exactly independent
and inactive. Each frame has its tick and FNV corruption checksum; header also
checksummed. Export SHA-256 of the complete stream. These are actual observations,
not interpolated states or full module-state restart checkpoints.

Reader validates exact bounded file size, provenance, dimensions and tick index,
reads frames on demand, and keeps bounded recent history rather than loading
3,073 frames into RAM. Replay has no scientific engine. Space/pause, one-tick
step and arbitrary seek remain; no LIVE/synthetic/old witness substitution.

Default full cube, source/midpoint inspection and fixed activity scales remain.
Expose global max|u|/|v| and numerical quiet threshold. Automatic playback advances
at most one recorded tick per rendered application update: when rendering is
slow, slow playback instead of skipping/catching up. No interpolation, auto-gain,
new forcing or changed physics. Paused manual single-step advances exactly1tick.

Independently decode/checksum all3,073frames, verify contiguous timeline, frozen
palette, finite/nonnegative aggregate fields, per-tick global metrics and quiet
tail decisions. Compare two completed runs, old prefix if supplied, local checks
and native iGPU playback evidence. Report software checks, native display,
scientific observations and push/CI separately; update handoff/operator docs.
