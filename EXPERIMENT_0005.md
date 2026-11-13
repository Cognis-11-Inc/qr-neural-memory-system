# Experiment 0005: morphic response and adaptive excitation

Registered 2026-11-13, America/Toronto. The registration Git commit and this
file's SHA-256 freeze the design before confirmatory data. No pilot outcomes
have been viewed. Software/unit-test fixtures are not confirmatory evidence.
No amendments at registration. Any later change must record timing and whether
outcomes were viewed; preserve this original registration.

## Authority and question

Principle 002 is proposed and falsifiable. Its preserved text is:

> A neuron's vibrational activity is sustained or diminished through its intrinsic response and energy exchange with neighboring neurons. Its internal morphology determines how it receives, retains, and transmits excitation. Sustained interactions may modify that morphology, subject to an explicit plasticity law.

Principle 001's persistent distributed association/retrieval claim is distinct.
This experiment cannot establish memory, retrieval, R1, consciousness, new physics
or unprogrammed cognition. Prior 0004 accuracy screens failed 0/8 (27.76–30.42%
worst signed-field L2 disagreement at102.4). Extension307.2 had no refinement.
Those reports, sources, registrations and existing raw data remain frozen.
`docs/EXPERIMENT_0005_PROTECTED_SHA256.txt` records baseline protected files.

## Model, precision, geometry and initialization

Serial CPU f64;4096 independent persistent8x8 palettes P=0..4. Transient u/v
are separate. Open16^3 Cubic6; IDs=x+16(y+16z). Internal graph is open8x8
cardinal adjacency, distinct from external matching-module links. One active
independent channel1; other channels exactly zero. No pruning, clipping or
normalization. Coefficients inherit0002/0003, with Cubic6 divisor6:

    omega_i=.8+.08*P_i+.04*occupied_internal_neighbors+.12*[P_i=1]
    gamma_i=.08+.01*empty_internal_neighbors+.02*[P_i!=1]
    g_i=.5 if P_i=0;1 if P_i=1;.75 otherwise
    k_internal=.6/4*g_i*g_j; k_external=.4/6*g_i*g_j
    du/dt=v; dv/dt=-omega^2*u-gamma*v+sum_j k_ij*(u_j-u_i)+f

Each undirected edge has symmetric k, unit mass, no wrapping and no boundary
renormalization. All sums deterministic in registered ascending construction
order. New code; historical semi-implicit Euler is unchanged.

SplitMix64 initialization via unmodified lattice constructor. Homogeneous all1;
heterogeneous seeded0..4; rearranged sorts each tile excluding27, retaining its
value and each tile's histogram. Seeds7,19 (two heterogeneous fixtures, limited
generality); homogeneous only7, never counted as independent repeated seeds.
Sources A2182=(6,8,8),B2186=(10,8,8),M2184; module27,unit initial u,v0.
A,B,AB+,AB- signs as0004;zero has all0. No subsequent forcing except labeled
M5-0 constant-force test f_A27=.1 through time16, starting from zero.
Singles/duals differ in initial energy. Across morphologies equal displacement
need not equal energy; report actual E0 and normalize energy fractions to E0.

## M5-0: energy and stronger integrator

Use second-order Strang splitting: exact damping for half dt, velocity-Verlet
for undamped stiffness plus constant forcing, exact damping for half dt.
Each kick reads one unchanged u vector; drift reads completed half-step v.
No in-place neighbor-dependent update. Coefficients frozen within each step.

    E = sum_i(.5*v_i^2+.5*omega_i^2*u_i^2)
        +sum_undirected_edges .5*k_ij*(u_i-u_j)^2

Damping loss for each exact substep is sum(.5*(v_before^2-v_after^2));
forcing work is f dot(u_after-u_before), exact for the constant force potential.
Cumulative drift=E-E0+loss-work-switch_work. This is measured discretization
error, not unexplained drive. Recompute energy from signed fields/actual graph.
Per-neuron energy assigns half of each edge's potential to each endpoint.
External edge instantaneous power into endpoint i is
`.5*k*(u_j-u_i)*(v_i+v_j)`; opposite sign for j. Sum cancels. This is the
continuous-time local energy-balance flux, sampled at ticks, not exact integrated
step work. Damping is local sink; boundary-damping loss reported separately.

Full-network M5-0: homogeneous7 and heterogeneous7 AB+, each dt .1,.05,.025
through307.2 (6 runs). Common full signed checkpoints0,.8,3.2,16,32,102.4,307.2.
Additional homogeneous7 undamped AB+ through32, each3 dt; constant-force from
zero through16,each3 dt. Zero control in A;12 numerical-study runs total.
Every-tick energies and signed probe traces. Compare combined full-field norm
sqrt(sum(delta_u^2+delta_v^2)/sum(finer_u^2+finer_v^2)), floor denominator1e-30.
Accept only if BOTH successive-grid errors <=5% through102.4,<=10% at307.2,
and fine/coarse error ratio<=.35 at32,102.4,307.2 (applicable windows only;
errors<1e-10 exempt ratio). Energy: max |cumulative drift|/max(E0,1e-12)<=1%
at primary dt.05 for impulses, undamped and forcing controls; forcing case
uses max(initial E,maximum E,1e-12) instead. Undamped drift must decrease under
refinement. Failure keeps downstream long-time physical interpretation
indeterminate, without retuning. B/C execution is gated on applicable M5-0
accuracy/energy tests; A can be exported as discrete-model characterization.
Hard stop/retained failure on any nonfinite or |u|,|v|>100;never clip.

## M5-A: bounded morphology matrix

Primary dt.05,time32. Five configurations:homogeneous7,heterogeneous7/19,
rearranged7/19; each A,B,AB+,AB-,zero =25 executions. This is a predeclared
representative suite, not all possible seeds/topologies. Full signed states at
0,.8,3.2,16,32. Verify both signed superpositions at each full checkpoint,
max residual<=1e-10*max(1,max|A|+|B|), separately u/v. In-run boundedness every
tick. Frozen P invariant. Rerun representative heterogeneous7 AB+ byte-identical.

Every tick global kinetic/onsite/coupling energy, actual damping losses, force
work, update work, drift, global amplitude/velocity, active neuron count and
selected probes. Probe IDs:0,2180,2182,2183,2184,2185,2186,2188,2191,2440,4095.
Export signed u27/v27, max|u|/|v|, per-neuron energy/net external power. All-node
spatial summaries:coords,hops from A,peak,max-energy,first crossing,subsequent
crossings,separated peaks,final response. Threshold is max-module|u|>=1e-6;
subsequent crossing requires a below-threshold tick since prior crossing.
Peak event is strict local maximum envelope>=1e-6 separated by>=1 time unit;
first eligible peak wins inside separation window. This rule is fixed before
results. A re-crossing is not proof of return from neighbors.
Probe finite-window DFT response at angular .8,1.2,1.6 (time0..32,raw transform
with dt weighting);phase atan2(-sine,cosine). This is transient spectral content,
not steady-state gain or a resolved resonance sweep. Spatial concentration:
energy participation (sum E_i)^2/sum E_i^2, and peak versus graph shell.

## M5-B: long response and boundary diagnostic

Through307.2 at dt.05: homogeneous7 and heterogeneous7 AB+ each open and boundary
sponge; heterogeneous7 AB- open and sponge =6 executions. The open AB+ data may
reuse M5-0 dt.05 exactly; ledger distinguishes reuse from new executions.
Sponge adds gamma_boundary=.5*((3-d)/3)^2 for d<3, else0, where d is minimum
integer distance to any of6 faces. Same P,initial u/v,couplings; no renormalized
boundary links. Separate exact damping losses for intrinsic and sponge sinks
using their gamma fractions. Report measured absorption fraction(sponge_loss/E0)
and late peak ratios(time>=32);a weaker/different pattern alone is not proof
of complete absorption or unambiguous reflection attribution.
Every tick global/probes;full checkpoints as M5-0. Source availability follows
passive evolution (no sustained drive). AB- is polarity control. Long patterns
may include local ringing,interference and reflections; boundary control only
provides diagnostic evidence,not unique attribution. No indefinitely sustained
or self-powered oscillation claim.

## M5-C stage registration: local bounded plasticity

This section freezes C before any C outcomes. No candidate pilots.
Seed7 heterogeneous, dt.05 and.025. Expose S=A throughtime16. Each module
integrates its own v^2 using trapezoidal physical-time quadrature over[0,8].
At exactlytime8,each neuron picks the module with largest integral (lowest
index on ties). If integral>=1e-8,edit that single module P->(P+1)%5;otherwise
no edit. Maximum1 edit/neuron/exposure;single commit,implicit cooldown forever.
Available information:own64 palettes,own64 integrated local velocities,local
clock;no targets,global reward,neuron-ID exceptions or future state. This
measures response-driven local history,not specifically neighbor-only history.
Rebuild onsite/damping and every affected symmetric link synchronously after
all edits. At unchanged u/v record E_pre/E_post,increment switch_work by
E_post-E_pre. Adaptive model is externally actuated,not energy-neutral.

At each dt run4 exposure conditions:frozen P;plasticity disabled using same
history/clock code;active local rule;generic-time control(change module0 of
EVERY neuron at8,independent of history). Add zero-input active-rule exposure
through16 (must have0 edits). Five exposures/grid. Save actual Pbefore/Pafter
and module differences for all4096;report saturation counts/spatial distributions.
Reset ALL transient u/v/history to zero,retain Pafter,apply identical S=A and
run frozen (plasticity off) through32 for active-Pafter and time-control-Pafter.
Virgin Pbefore A response reuses frozen exposure extended to32; disabled also
through32. Seven runs/grid,14 total. Reexposure frozen-Pafter isolates persistent
causal response from ongoing transient activity. Difference measured full signed
L2 against virgin at16,32;descriptive positive effect requires>=1% both grids,
change-count difference<=5%,and response-effect relative difference<=20%.
If refined schedules select different edits/report criteria fail,the effect is
numerically indeterminate;palette change alone is never a memory result.
Zero-drive and generic-time controls test dependence on local drive;generic
control is deliberately NOT change-count or histogram matched. No association,
recall or resilience test. Exposure traces also retained. No hidden energy.

## Records, provenance, replay and delivery

Dedicated versioned QR5 full-field snapshots: magic/schema,tick,dt,seed,
configuration,condition identity,actual P,full signed u/v,FNV integrity checksum;
SHA-256 manifest and frozen registration/source copies in ignored outputs.
Every-tick CSV metrics/probes;full frames only at checkpoints,never interpolate
or imply every-tick full-module recording. RAM bounded by field/graph arrays,
not horizon. Python independently decodes/checks snapshots,recomputes energy,
checks histogram/source matching,superposition,convergence,events/spectra and
plastic reexposure decisions;not a second time integrator. Explicit execution
ledger lists all planned/attempted/completed/reused/unexecuted conditions.
Viewer-free ZIP includes CSVs,schemas,actual palettes,source/rules and hashes;
full frames may ship separately to keep it portable. Integrity is not authentication.

Viewer spacing is purely visual:default1.8,safe1..3 CLI,live adjustment/reset.
Audit positions,picking,edges,axes,slices,camera fitting and witness. Native iGPU
before/after capture. Old modes preserved.0005 replay only when honestly labeled
with new provenance;never route to old LIVE physics. Software format/lint/test/
build/docs and Python independent checks,viewer tests/native witness,protected
hashes,commit/push/remote and hosted CI reported separately. If resources or
numerical gates prevent execution,retain exact partial ledger and report why;
never claim full completion. No arbitrary compute truncation disguised as failure.
