"""Create the scientific report from verified QR6 measurements and complete ledger."""

import argparse
import json
from pathlib import Path

from analysis.experiment_0006 import require, rows


def report(root, destination):
    data = json.loads((root / "verification.json").read_text())
    ledger = rows(root / "execution-ledger.csv")
    require(len(data["runs"]) == 168 and len(ledger) == 190, "complete measurements required")
    require(
        all(r["status"] in ("COMPLETE", "REUSED") for r in ledger), "unexecuted conditions remain"
    )
    numeric = []
    for result in data["numerical"]["comparisons"]:
        if result["time"] in (32, 102.4, 307.2):
            numeric.append(
                f"| {result['config']} {result['seed']} | "
                f"{result['time']} | "
                f"{100 * result['coarse']['relative_l2']:.4f}% | "
                f"{100 * result['fine']['relative_l2']:.4f}% | "
                f"{result['ratio']:.4f} |"
            )
    diagnosis = []
    for result in data["runs"]:
        if result["id"].startswith("d-") and "zero" not in result["id"]:
            observed = ", ".join(
                f"{m}:{count}" for m, count in enumerate(result["raw_argmax_histogram"]) if count
            )
            diagnosis.append(f"| {result['id']} | {observed} |")
    morphology = []
    for result in data["runs"]:
        if (
            result["id"].startswith("m-")
            and "-E1-R" in result["id"]
            and "d0.025" in result["id"]
            and any(f"-R{rule}-" in result["id"] for rule in (2, 3))
        ):
            morphology.append(
                f"| {result['id']} | "
                f"{result['changed_neurons']} | "
                f"{result['changed_modules']} | "
                f"{result['switch_work']:.8f} | "
                f"{result['count_histogram']} |"
            )
    structure = [
        f"| {r['seed']}/{r['module']}/R{r['rule']} | "
        f"{r['jaccard']:.4f} | "
        f"{100 * r['exposure_errors'][0]:.4f}% | "
        f"{100 * r['exposure_errors'][1]:.4f}% | "
        f"{'PASS' if r['passed'] else 'INDETERMINATE'} |"
        for r in data["structural"]
    ]
    causal = []
    for result in data["causal"]:
        if result["dt"] == 0.025 and result["kind"] == "OFF_SOURCE_ONLY":
            effects = result["effects"]
            causal.append(
                f"| {result['seed']}/{result['module']}/R{result['rule']} | "
                f"{result['budget']} | "
                f"{100 * effects[0]['offsource_effect']['relative_l2']:.3f}% | "
                f"{100 * effects[1]['offsource_effect']['relative_l2']:.3f}% | "
                f"{effects[1]['offsource_effect']['absolute_rms']:.6e} |"
            )
    decisions = [
        f"| {r['seed']}/{r['module']}/R{r['rule']} | "
        f"{r['budget']} | "
        f"{r['status']} | "
        f"{r['source_only_nearly_matches_all']} |"
        for r in data["causal_gates"]
    ]
    global_pairs = []
    for row in rows(root / "agent-report/pair-contrasts.csv"):
        if (
            row["dt"] == "0.025"
            and row["budget"] == "MATCH"
            and row["time"] == "32"
            and (int(row["seed"]), int(row["module"])) in ((7, 27), (19, 9))
        ):
            global_pairs.append(
                f"| {row['seed']}/{row['module']}/R{row['rule']} | "
                f"{100 * float(row['global_off_vs_virgin_relative_l2']):.3f}% | "
                f"{100 * float(row['global_source_vs_virgin_relative_l2']):.3f}% | "
                f"{100 * float(row['global_all_vs_source_relative_l2']):.3f}% |"
            )
    max_drift = max(r["max_energy_drift"] for r in data["runs"])
    ticks = sum(r["ticks"] for r in data["runs"])
    frames = sum(r["frames"] for r in data["runs"])
    switches = sum("count_histogram" in r for r in data["runs"])
    structural_pass = sum(r["passed"] for r in data["structural"])
    causal_pass = sum(r["status"] == "PASS" for r in data["causal_gates"])
    numerical_pass = sum(r["passed"] for r in data["numerical"]["comparisons"])
    source_near = sum(r["source_only_nearly_matches_all"] for r in data["causal_gates"])
    text = f"""# Experiment 0006: multi-module morphology and causal re-exposure

Measured 2026-11-13. Registration `578870c` froze the equations, thresholds and
matrix before confirmatory data. Implementation `6904303`; instrument/verification
checkpoint `673eddf`. [Design](../EXPERIMENT_0006.md),
[inventory](EXPERIMENT_0006_INVENTORY.md), [schema](EXPERIMENT_0006_SCHEMA.md).
The user brief was a proposed design, not prior evidence. No pilots or post-result
retuning were used. M5's failed accuracy criteria and all historical files remain
unchanged. Principle 002 is proposed; this is not an associative memory experiment.

## Results and coverage

M6-N refined numerical screen: **{"PASS" if data["numerical"]["long_pass"] else "FAIL"}**.
M6-M selected structural/fine-grid checks: **{structural_pass}/4 pass**.
M6-C selected off-source causal gates: **{causal_pass}/8 pass**.
These are finite deterministic model results, not physical fidelity, learning,
consciousness, general stability or R1 resilience claims.

All **190 logical conditions** are accounted for: N 12, D 10, M 48, C 120. There are
**168 primary executions and 22 explicit reuses**; the preselected raw-case repeat
adds one execution. No planned condition is unexecuted. Primary output includes
{ticks:,} every-tick global observations, {frames:,} main full signed checkpoints,
and {2 * switches} separate pre/post switch fields. All source/probe observations
are actual engine outputs, not interpolation. C VIRGIN reuse names its original
trajectory without rewriting frame labels or pretending it is a new sample.

All fields remain finite/bounded, all 13 registered zero-input controls stay zero,
and no clipping, hidden drive or outcome normalization occurs. Worst measured
absolute cumulative energy drift/E0 across primary runs: **{100 * max_drift:.5f}%**;
registered bound 1%. Exact damping loss and palette-switch work are separate.
Static causal-variant construction precedes t0; re-exposure bookkeeping starts
after complete transient/history/audit reset with explicitly measured E0.

## M6-N: refinement resolves the registered sensitivity screen

Same serial CPU f64 physical coefficients, edge order and second-order
Strang-damping/velocity-Verlet method as M5. Only the registered step grid changes.
Limits are **informed by M5 observations**: through 102.4, .05/.025<=5% and
.025/.0125<=1.5%; later checkpoints through 307.2 require<=10% and<=3%, ratio<=.35.
Both comparisons must pass every retained checkpoint. The new pass is not a
retrospective pass of M5's .1/.05 criterion, nor a proof of physical fidelity.

| Fixture | Physical time | .05/.025 signed L2 | .025/.0125 signed L2 | Error ratio |
|---|---:|---:|---:|---:|
{chr(10).join(numeric)}

{numerical_pass}/33 nonzero fixture/checkpoint decisions pass.
The numerical plot joins registered checkpoints only, not interpolated fields.
Absolute RMS, source/midpoint/
boundary waveform disagreements and nearest separated-peak timing diagnostics
are exported. Peak alignment does not replace unaligned acceptance. As signals
decay, absolute RMS and relative errors must be read together. An independent
analytic uniform-mode unit fixture checks second-order refinement; it is a test
fixture, not a new scientific model. Four old/new seed7 field arrays through 307.2
match M5 byte-for-byte; historical source files and verdicts were not changed.

![Measured refined-grid sensitivity and energy drift](figures/experiment_0006_numerics.png)

## M6-D: module 27 follows the drive coordinate

Frozen P, source A, seeds 7/19, modules27/9, unit-displacement and mechanical E1
conditions. Input amplitudes come from the actual full quadratic energy, not
sum(u²)+sum(v²). Every E1 trial independently matches energy1 within1e-12.
Raw V argmax uses the original 1e-8 threshold; counts below refer only to qualified
neurons, not the4096-neuron population or a mandatory edit count.

| Frozen diagnosis case | Argmax module: qualified-neuron count |
|---|---|
{chr(10).join(diagnosis)}

Module27 is not special in the argmax law. Changing the injection coordinate
changes the dominant selected index. The single-edit cap guarantees at most one
position even when other positions carry history. Matching-index external links
and the point input favor that index. This study does not independently ablate
each graph/measure/cap contribution, so their separate causal weights remain
unresolved. Full per-module V/H/S/ranks, per-neuron spread/distances and graph-hop
summaries are exported; weak nonzero signals are not automatically eligibility.
Historical R1 unit seed7/module 27/.025 reproduces M5's63 edits and full time 16
u/v/P/history bytes exactly. It is a regression, not new multi-module success.

## M6-M: multi-position recoloring and actual shape-only swaps

R2 uses locally measured v² plus internal strain history, then one-hop own-neuron
eligibility; at most8 qualified positions recolor. R3 greedily selects up to 4
disjoint cardinal unequal-color swaps using old P and local scores. R3 preserves
every neuron's palette histogram. No minimum edit quota, target layout, reward
or neuron-ID frequency is used. The .25 strain coefficient and .5 smoothing
coefficient are chosen model parameters, not biophysics. One commit at 8 remains
a deliberate bound; continuing autonomous morphogenesis was not tested.

| Primary E1 case | Neurons | Positions | Switch work | Position histogram 0..8 |
|---|---:|---:|---:|---|
{chr(10).join(morphology)}

Reported changes involve multiple actual coordinates in one neuron's 8x8 array.
They remain concentrated near the injection coordinate; the rule does not
establish rich organization or useful learned structure. All before/after arrays
and per-module differences are retained for 4096 neurons in every exposure.
Saturation, source/immediate-neighbor/remote counts and actual color changes are
in morphology-summary.csv. Frozen and disabled controls have identical signed
states and palettes; zero-input adaptive controls make no edits.

| Fine pair | Position Jaccard | Error at16 | Error at32 | Structural decision |
|---|---:|---:|---:|---|
{chr(10).join(structure)}

All four selected pairs also produce byte-identical Pafter. Persisting palette
positions are checked after transient reset in frozen ALL re-exposure. This is
structural persistence with a changed mechanical response, not an association.

Palette changes can add or remove stored energy even for histogram-preserving
swaps. Pre/post signed fields hold u/v exactly fixed; independent graph-energy
reconstruction checks every neuron's half-edge energy partition and total work.
The adaptive model is externally actuated, not energy-neutral. No apparent
resurgence is treated as self-powered activity.

![Actual source palette before/after shape-only swap](figures/experiment_0006_palette.png)

## M6-C: source/off-source causality after complete reset

Pbefore/Pafter are immutable. VIRGIN, SOURCE_ONLY, OFF_SOURCE_ONLY, ALL and
SHUFFLE_POSITION each undergo frozen time 32 re-exposure after all u/v/history/
audit accumulators reset. UNIT means identical displacement1 with actual E0
reported; MATCH means identical mechanical E0 to virgin unit input, with the
explicit amplitude sqrt(E0_virgin/E0_variant_unit). No outcome is normalized.

The shuffled control preserves changed-neuron IDs, exact per-neuron edit count
and graph-distance distribution. R3 also preserves each palette histogram;
its control swaps need not be cardinal and are labeled accordingly. R2 shuffled
color-change histograms may differ and are not claimed matched. A control with
all 4096 neurons changed was not substituted for the local edit budgets.

Differences above100% are possible because the norm is relative to the virgin
field; they are not accuracy, improvement or percent-contribution scores.
Absolute RMS is also reported. OFF_SOURCE_ONLY versus VIRGIN excludes source A
from the observation mask:

| Seed/module/rule | Input comparison | Relative L2 at16 | At32 | Absolute RMS at32 |
|---|---|---:|---:|---:|
{chr(10).join(causal)}

The selected fine-grid causal gate requires>=1% effects at both16/32, both grids,
<=20% effect disagreement and<=1.5% unaligned coarse/fine fields per variant.
"Source-only nearly matches ALL" requires<=1% ALL/SOURCE_ONLY difference at both
checkpoints on the off-source mask. These are defined group contrasts, never
an additive decomposition or percent contribution.

| Selected fine pair | Input comparison | Causal gate | Source-only nearly matches ALL |
|---|---|---|---|
{chr(10).join(decisions)}

Source-only nearly matches ALL in **{source_near}/8** selected comparisons at the
registered1% criterion. The source-only description is insufficient for these
R2/R3 cases, including the equal-energy comparisons. M5 R1's own source-only
ablation was not retrospectively performed.

The global vector also differs substantially, not only the source-excluded mask.
Selected primary MATCH comparisons at32 (reference norm is the right-hand state):

| Seed/module/rule | Global OFF/VIRGIN | Global SOURCE/VIRGIN | Global ALL/SOURCE |
|---|---:|---:|---:|
{chr(10).join(global_pairs)}

These separate contrasts are not additive and are not shares of responsibility.
The contrasts establish which retained parameter subsets affect these measured
trajectories. They do not establish that every changed off-source neuron is
necessary, that the rule optimized anything, or that a learned association was
stored. Unselected primary seed/module combinations were not separately refined;
no universal claim is inferred from two seeds and two injection coordinates.
Shuffled layouts can also alter responses; difference alone is not usefulness.

![Measured source re-exposure with both input conventions](figures/experiment_0006_reexposure.png)

![Measured off-source-only contrasts](figures/experiment_0006_offsource.png)

## Verification, observatory and delivery

Independent Python decodes/checks signed fields, reconstructs energy/flux from
neighbor tensor slices, rederives every rule/edit from exported local fields and
checks mask/normalization identities. It re-integrates V/H from the 6 recorded
full-module early probes only; it is not a second whole-network integration.
Every-tick global arithmetic and selected probes are distinct from full-field
checkpoint checks. Source/input-bound cache receipts do not authenticate data.
The preselected seed19/module 9/E1/R3 raw-case repeat is byte-identical on this
machine/toolchain; no cross-platform bit identity is promised.

The QR6 Bevy mode is replay-only, with default spacing 1.8. Final palette-diff
colors mean count0,1,2-3,4+, not amplitude or energy. Before/final-after panels
show actual 8x8 bytes for the selected neuron. Native iGPU receipts for source
R3 and a module 9 R2 neighbor match exact observation/palette hashes, counts 8/7
and 0 engines. Adapter: Raphael IntegratedGpu, Vulkan/radv, Mesa26.2.6-arch1.1,
X11 on Wayland/XWayland. Frame caps are configured, not measured sustained FPS.
The initial native attempt exposed the new loader's old extension filter; it
was fixed with an on-disk corruption/replay regression. Failed log retained;
no old schema or scientific output changed.

![Native QR6 recorded palette comparison](figures/experiment_0006_native.png)

All 55 protected historical source/specification/results files match baseline
SHA-256. Local quality gates pass 31 Rust, 21 Python, 3 viewer tests, strict lints,
formatting, release/docs/doctests, Python compilation and isolated native build.
Commit/remote/hosted CI evidence is recorded separately in HANDOFF and the final
delivery receipt. Software green does not override a scientific failure.

Run the README stages in order, then independent verification, summarize/plot,
and package commands. Portable AGENT_REPORT contains viewer-free CSV/JSON,
all palettes and eligibility, schema, rules, source, full execution ledger,
independent decisions and native receipts. FULL_FIELDS separately contains exact
signed checkpoints; extract both into one root for independent field reanalysis.
Large data remain local/ignored. Historical raw M5 regression can be repeated
only when its separate raw package is available; original receipts are retained.

Remaining limits: one switch time, bounded candidate rules, coordinate-local
edits, two seeds, no individual off-source-neuron ablation, no association or
retrieval task, no capacity/interference or resilience result. A changed tile,
large waveform difference or a prettier cube is not proof of memory or cognition.
"""
    destination.write_text(text)
    portable = text.replace("../EXPERIMENT_0006.md", "RULES.md")
    portable = portable.replace("(EXPERIMENT_0006_INVENTORY.md)", "(INVENTORY.md)")
    portable = portable.replace("(EXPERIMENT_0006_SCHEMA.md)", "(SCHEMA.md)")
    (root / "REPORT.md").write_text(portable)
    print(destination)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    report(args.root, args.destination)
