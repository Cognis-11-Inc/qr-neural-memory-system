# QR5 signed checkpoints and portable observations, version 1

The fixed registration is `EXPERIMENT_0005.md`, commit `5b97a75`.
All numbers are nondimensional. Channel 1 alone is stored; the other three
uncoupled channels remain exactly zero. Neither renderer spacing nor wall time
enters a scientific record. FNV/SHA hashes check integrity, not authentication.

## Full-state checkpoint (`*.q5f`)

Little endian, no padding. Magic 8 bytes `QR5FLD01`; then 6 unsigned 64 fields:
schema=1, tick, f64 timestep bits, seed, initial configuration code (0 seeded,
1 all-red, 2 rearranged), UTF-8 label byte count (1..200). Label follows at byte 56.
The label is the condition ID from the execution ledger, not a visual alias.
Then 4096*64 palette bytes in stable neuron/module order (0..4), 262144 f64 signed
u, 262144 f64 signed v. Final u64 FNV-1 a over all preceding bytes. Length is
56+label_length+17*262144+8. All indices are zero-based; module=row*8+column.
Tick<=12288, dt in{.1,.05,.025}. Reject nonfinite or absolute fields>100.

Each file records actual P, not a histogram. Initial configuration code remains
its initial class after plasticity; P bytes are authoritative. Files contain full
active-channel states at registered checkpoints, not every tick. The reader
checks identity, strictly increasing ticks and maximum 7 frames. Replay never
starts physics. Missing checkpoints are not interpolated. `n-*` and `a-*` IDs map
to the exact model/controls in the frozen registration and stage ledgers.

## Streamed CSVs and palette/history exports

- `metrics.csv`:every integer tick including 0; tick, time, kinetic, onsite, coupling,
  cumulative intrinsic loss, sponge loss, constant-force work, switch work, drift,
  max|u|/|v|, instantaneous active-neuron count, energy participation, edit count.
  Participation=(sum local energy)^2/sum(local energy^2), 0 if zero energy.
  Drift=E-E0+loss+sponge_loss-work-switch_work, not a hidden balance correction.
- `probes.csv`:11 registered sites every tick; signed module 27 u/v, max-module
  amplitudes, local energy and net external power. Local energy assigns half of
  each edge potential to each endpoint. Power is an instantaneous continuous
  energy-balance diagnostic, not integrated discrete work.
- `spatial.csv`:4096 stable IDs, coords, hops from A, cumulative max-module peak,
  largest local energy, first threshold crossing (blank if absent), number of
  upward crossings, separated envelope maxima, final max-module displacement.
- `before.palette`,`after.palette`:4096*64 raw 0..4 bytes. No transient included.
- `history.bin`:for C only; 262144 little-endian f64 integrals of own v² over[0, 8],
  using trapezoidal dt weights. `switch.csv` records exact pre/post energy at 8,
  work and edit count. These permit independent rule reconstruction.
- `ledger-STAGE.csv`:attempted IDs with seed, class, dt, arm, horizon and control
  switches. Per-case COMPLETE records execution only. `execution-ledger.csv`
  additionally enumerates unexecuted gated conditions and reused fixtures.
- `verification.json`:independent decoding/energy checks, convergence, events,
  finite-window DFT magnitudes/phases, boundary/plastic decisions. Full-field
  independence is at checkpoints; selected probe event checks are every tick.
- `NUMERICAL_GATES.csv`:explicit short-window and long-window gates. The runner
  rejects C without a short-window pass and B without a long-window pass.
- `sources/`:actual frozen rules and implementation copies. Numerical runs
  preceded additional history-export/gate instrumentation; original source
  receipt is preserved alongside subsequent source receipts. Dynamics unchanged.

The portable package omits large full fields but includes their SHA-256 ledger,
CSV series, palettes, local histories, rules, source, schema, independent decisions
and native receipts. `FULL_FIELDS.zip` separately retains all signed checkpoints.
Re-running full-field verification requires extracting both into the same root.
