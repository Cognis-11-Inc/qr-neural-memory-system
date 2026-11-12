//! Frozen Experiment 0002 T/R: configuration-dependent oscillators, no plasticity.

use std::f64::consts::{PI, TAU};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

const MODULES: usize = 64;
const MODULE_SIDE: usize = 8;
const SIDE: usize = 64;
const SEEDS: [u64; 3] = [7, 19, 41];
const BINS: [usize; 7] = [4, 8, 12, 16, 20, 24, 28];

fn number(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("bounded experiment size"))
}

struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}

fn adjacent(index: usize, side: usize, diagonals: bool) -> Vec<usize> {
    let (row, col) = (index / side, index % side);
    let mut result = Vec::new();
    for r in row.saturating_sub(1)..=(row + 1).min(side - 1) {
        for c in col.saturating_sub(1)..=(col + 1).min(side - 1) {
            if (r != row || c != col) && (diagonals || r == row || c == col) {
                result.push(r * side + c);
            }
        }
    }
    result
}

fn configuration(seed: u64, class: &str) -> Vec<u8> {
    let mut random = Random(seed);
    let mut values: Vec<u8> = (0..SIDE * SIDE * MODULES)
        .map(|_| u8::try_from(random.next() % 5).expect("palette"))
        .collect();
    match class {
        "homogeneous" => values.fill(1),
        "rearranged" => {
            for tile in values.as_chunks_mut::<MODULES>().0 {
                let mut rest: Vec<u8> = tile
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &value)| (i != 27).then_some(value))
                    .collect();
                rest.sort_unstable();
                let mut next = rest.into_iter();
                for (index, module) in tile.iter_mut().enumerate() {
                    if index != 27 {
                        *module = next.next().expect("63 modules");
                    }
                }
            }
        }
        "heterogeneous" => (),
        _ => panic!("unregistered class"),
    }
    values
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Scalar,
    Isolated,
    Patch,
    Full,
}

impl Mode {
    fn side(self) -> usize {
        match self {
            Self::Scalar | Self::Isolated => 1,
            Self::Patch => 3,
            Self::Full => SIDE,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Isolated => "isolated",
            Self::Patch => "patch",
            Self::Full => "full",
        }
    }
}

fn extract(full: &[u8], mode: Mode) -> Vec<u8> {
    if mode == Mode::Full {
        return full.to_vec();
    }
    let side = mode.side();
    let mut result = Vec::with_capacity(side * side * MODULES);
    for row in 0..side {
        for col in 0..side {
            let neuron = (SIDE / 2 + row - side / 2) * SIDE + SIDE / 2 + col - side / 2;
            result.extend_from_slice(&full[neuron * MODULES..(neuron + 1) * MODULES]);
        }
    }
    result
}

#[derive(Clone, Copy, Default)]
struct Diagnostic {
    peak: f64,
    velocity_peak: f64,
    square: f64,
    velocity_square: f64,
    nonfinite: usize,
}

impl Diagnostic {
    fn valid(self) -> bool {
        self.nonfinite == 0 && self.peak <= 100.0 && self.velocity_peak <= 100.0
    }
}

struct Medium {
    mode: Mode,
    channel: usize,
    structure: Vec<u8>,
    omega_squared: Vec<f64>,
    damping: Vec<f64>,
    conductance: Vec<f64>,
    u: Vec<[f64; 4]>,
    v: Vec<[f64; 4]>,
    next_u: Vec<[f64; 4]>,
    next_v: Vec<[f64; 4]>,
    internal: Vec<Vec<usize>>,
    external: Vec<Vec<usize>>,
}

impl Medium {
    fn new(full: &[u8], mode: Mode, channel: usize, gamma_factor: f64) -> Self {
        let structure = extract(full, mode);
        let internal: Vec<_> = (0..MODULES)
            .map(|i| adjacent(i, MODULE_SIDE, false))
            .collect();
        let mut omega_squared = Vec::with_capacity(structure.len());
        let mut damping = Vec::with_capacity(structure.len());
        let mut conductance = Vec::with_capacity(structure.len());
        for (index, &color) in structure.iter().enumerate() {
            let base = index / MODULES * MODULES;
            let links = &internal[index % MODULES];
            let occupied = links.iter().filter(|&&j| structure[base + j] != 0).count();
            let omega = 0.8
                + 0.08 * f64::from(color)
                + 0.04 * number(occupied)
                + if usize::from(color) == channel + 1 {
                    0.12
                } else {
                    0.0
                };
            omega_squared.push(omega * omega);
            damping.push(
                gamma_factor
                    * (0.08
                        + 0.01 * number(links.len() - occupied)
                        + if usize::from(color) == channel + 1 {
                            0.0
                        } else {
                            0.02
                        }),
            );
            conductance.push(if color == 0 {
                0.5
            } else if usize::from(color) == channel + 1 {
                1.0
            } else {
                0.75
            });
        }
        let points = structure.len();
        Self {
            mode,
            channel,
            structure,
            omega_squared,
            damping,
            conductance,
            u: vec![[0.0; 4]; points],
            v: vec![[0.0; 4]; points],
            next_u: vec![[0.0; 4]; points],
            next_v: vec![[0.0; 4]; points],
            internal,
            external: (0..mode.side().pow(2))
                .map(|i| adjacent(i, mode.side(), true))
                .collect(),
        }
    }

    fn reset(&mut self) {
        self.u.fill([0.0; 4]);
        self.v.fill([0.0; 4]);
        self.next_u.fill([0.0; 4]);
        self.next_v.fill([0.0; 4]);
    }

    fn diagnostic(&self) -> Diagnostic {
        let mut result = Diagnostic::default();
        for (&value, &velocity) in self.u.iter().flatten().zip(self.v.iter().flatten()) {
            if !value.is_finite() || !velocity.is_finite() {
                result.nonfinite += 1;
            }
            result.peak = result.peak.max(value.abs());
            result.velocity_peak = result.velocity_peak.max(velocity.abs());
            result.square += value * value;
            result.velocity_square += velocity * velocity;
        }
        result
    }

    fn step(&mut self, dt: f64, source: usize, force: f64, reverse: bool) -> Diagnostic {
        let channel = self.channel;
        for position in 0..self.u.len() {
            let index = if reverse {
                self.u.len() - 1 - position
            } else {
                position
            };
            let value = self.u[index][channel];
            let mut acceleration =
                -self.omega_squared[index] * value - self.damping[index] * self.v[index][channel];
            let module = index % MODULES;
            let base = index - module;
            let g = self.conductance[index];
            if self.mode != Mode::Scalar {
                for &neighbor in &self.internal[module] {
                    let j = base + neighbor;
                    acceleration +=
                        (0.6 / 4.0) * g * self.conductance[j] * (self.u[j][channel] - value);
                }
            }
            if matches!(self.mode, Mode::Patch | Mode::Full) {
                for &neuron in &self.external[index / MODULES] {
                    let j = neuron * MODULES + module;
                    acceleration +=
                        (0.4 / 8.0) * g * self.conductance[j] * (self.u[j][channel] - value);
                }
            }
            if index == source {
                acceleration += force;
            }
            self.next_v[index][channel] = self.v[index][channel] + dt * acceleration;
            self.next_u[index][channel] = value + dt * self.next_v[index][channel];
        }
        std::mem::swap(&mut self.u, &mut self.next_u);
        std::mem::swap(&mut self.v, &mut self.next_v);
        self.diagnostic()
    }
}

#[derive(Clone, Copy, Debug)]
enum Drive {
    Impulse,
    Burst,
    Tone,
    Multi,
    RandomPhase,
    Zero,
}

impl Drive {
    fn name(self) -> &'static str {
        match self {
            Self::Impulse => "impulse",
            Self::Burst => "burst",
            Self::Tone => "tone",
            Self::Multi => "multisine",
            Self::RandomPhase => "random_phase",
            Self::Zero => "zero",
        }
    }
}

#[derive(Clone)]
struct Trial {
    stage: &'static str,
    seed: u64,
    class: &'static str,
    mode: Mode,
    channel: usize,
    source: &'static str,
    drive: Drive,
    bin: usize,
    dt: f64,
    steps: usize,
    gamma: f64,
}

impl Trial {
    fn omega(&self) -> f64 {
        if self.stage == "T" {
            PI / 2.0
        } else {
            TAU * number(self.bin) / 64.0
        }
    }
    fn source_point(&self) -> usize {
        let side = self.mode.side();
        let neuron = match self.source {
            "edge" => side / 2,
            "corner" => 0,
            _ => side / 2 * side + side / 2,
        };
        neuron * MODULES + 27
    }
    fn input(&self) -> (Vec<f64>, f64) {
        let mut random = Random(7);
        let phases: [f64; 3] = std::array::from_fn(|_| {
            TAU * number(usize::try_from(random.next() % 1_000_000).expect("phase")) / 1_000_000.0
        });
        let mut samples = vec![0.0; self.steps + 1];
        for (step, value) in samples.iter_mut().enumerate().skip(1) {
            let time = number(step) * self.dt;
            *value = match self.drive {
                Drive::Impulse | Drive::Zero => 0.0,
                Drive::Burst if time > 8.0 => 0.0,
                Drive::Burst | Drive::Tone => (self.omega() * time).sin(),
                Drive::Multi | Drive::RandomPhase => [8, 16, 24]
                    .iter()
                    .enumerate()
                    .map(|(i, &bin)| {
                        (TAU * number(bin) / 64.0 * time
                            + if matches!(self.drive, Drive::RandomPhase) {
                                phases[i]
                            } else {
                                0.0
                            })
                        .sin()
                    })
                    .sum(),
            };
        }
        let energy: f64 = samples.iter().map(|x| x * x * self.dt).sum();
        let scale = if energy > 0.0 {
            energy.sqrt().recip()
        } else {
            0.0
        };
        for value in &mut samples {
            *value *= scale;
        }
        (samples, scale)
    }
}

struct Probe {
    label: String,
    point: usize,
    row: usize,
    col: usize,
    module: usize,
    hops: usize,
    distance: f64,
    scope: &'static str,
}

fn probes(trial: &Trial) -> Vec<Probe> {
    let point = trial.source_point();
    let side = trial.mode.side();
    let (row, col) = (point / MODULES / side, point / MODULES % side);
    let mut result = vec![Probe {
        label: "origin".into(),
        point,
        row,
        col,
        module: 27,
        hops: 0,
        distance: 0.0,
        scope: "external",
    }];
    if trial.stage != "T" {
        return result;
    }
    for (name, dr, dc) in [
        ("N", -1, 0),
        ("S", 1, 0),
        ("W", 0, -1),
        ("E", 0, 1),
        ("NE", -1, 1),
        ("NW", -1, -1),
        ("SE", 1, 1),
        ("SW", 1, -1),
    ] {
        for hops in [1usize, 2, 4, 8, 16] {
            let offset = isize::try_from(hops).expect("small distance");
            let Some(r) = row.checked_add_signed(dr * offset).filter(|&x| x < side) else {
                continue;
            };
            let Some(c) = col.checked_add_signed(dc * offset).filter(|&x| x < side) else {
                continue;
            };
            result.push(Probe {
                label: format!("{name}{hops}"),
                point: (r * side + c) * MODULES + 27,
                row: r,
                col: c,
                module: 27,
                hops,
                distance: number(hops)
                    * if dr != 0 && dc != 0 {
                        2.0_f64.sqrt()
                    } else {
                        1.0
                    },
                scope: "external",
            });
        }
    }
    for (module, hops) in [(28, 1), (29, 2), (63, 8)] {
        result.push(Probe {
            label: format!("internal{module}"),
            point: point - 27 + module,
            row,
            col,
            module,
            hops,
            distance: number(hops),
            scope: "internal",
        });
    }
    result
}

fn trials() -> Vec<Trial> {
    let mut result = Vec::new();
    for seed in SEEDS {
        for channel in 0..4 {
            for source in ["center", "edge", "corner"] {
                for drive in [Drive::Impulse, Drive::Burst] {
                    result.push(Trial {
                        stage: "T",
                        seed,
                        class: "heterogeneous",
                        mode: Mode::Full,
                        channel,
                        source,
                        drive,
                        bin: 16,
                        dt: 0.1,
                        steps: 160,
                        gamma: 1.0,
                    });
                }
            }
        }
    }
    result.push(Trial {
        stage: "T",
        seed: 7,
        class: "homogeneous",
        mode: Mode::Full,
        channel: 0,
        source: "center",
        drive: Drive::Zero,
        bin: 16,
        dt: 0.1,
        steps: 160,
        gamma: 1.0,
    });
    spectral_trials(&mut result);

    spectral_controls(&mut result);
    assert_eq!(result.len(), 1101);
    result
}

fn spectral_controls(result: &mut Vec<Trial>) {
    for bin in BINS {
        result.push(Trial {
            stage: "R",
            seed: 7,
            class: "heterogeneous",
            mode: Mode::Full,
            channel: 0,
            source: "center",
            drive: Drive::Tone,
            bin,
            dt: 0.1,
            steps: 640,
            gamma: 1.0,
        });
        result.push(Trial {
            stage: "R",
            seed: 7,
            class: "heterogeneous",
            mode: Mode::Isolated,
            channel: 0,
            source: "center",
            drive: Drive::Tone,
            bin,
            dt: 0.1,
            steps: 640,
            gamma: 2.0,
        });
        if [8, 16, 24].contains(&bin) {
            result.push(Trial {
                stage: "R",
                seed: 7,
                class: "heterogeneous",
                mode: Mode::Full,
                channel: 0,
                source: "center",
                drive: Drive::Tone,
                bin,
                dt: 0.05,
                steps: 1280,
                gamma: 1.0,
            });
        }
    }
    for drive in [Drive::Multi, Drive::RandomPhase, Drive::Zero] {
        result.push(Trial {
            stage: "R",
            seed: 7,
            class: if matches!(drive, Drive::Zero) {
                "homogeneous"
            } else {
                "heterogeneous"
            },
            mode: if matches!(drive, Drive::Zero) {
                Mode::Isolated
            } else {
                Mode::Patch
            },
            channel: 0,
            source: "center",
            drive,
            bin: 16,
            dt: 0.1,
            steps: 640,
            gamma: 1.0,
        });
    }
}

fn spectral_trials(result: &mut Vec<Trial>) {
    for seed in SEEDS {
        for class in ["homogeneous", "heterogeneous", "rearranged"] {
            for channel in 0..4 {
                for bin in BINS {
                    for mode in [Mode::Scalar, Mode::Isolated, Mode::Patch] {
                        result.push(Trial {
                            stage: "R",
                            seed,
                            class,
                            mode,
                            channel,
                            source: "center",
                            drive: Drive::Tone,
                            bin,
                            dt: 0.1,
                            steps: 640,
                            gamma: 1.0,
                        });
                    }
                    result.push(Trial {
                        stage: "R",
                        seed,
                        class,
                        mode: Mode::Isolated,
                        channel,
                        source: "center",
                        drive: Drive::Tone,
                        bin,
                        dt: 0.05,
                        steps: 1280,
                        gamma: 1.0,
                    });
                }
            }
        }
    }
}

#[derive(Default)]
struct Measurement {
    peak: f64,
    square: f64,
    velocity_square: f64,
    drive_square: f64,
    response_sin: f64,
    response_cos: f64,
    drive_sin: f64,
    drive_cos: f64,
    count: usize,
    arrival: Option<f64>,
    detection: Option<f64>,
}

impl Measurement {
    fn sample(&mut self, u: f64, v: f64, drive: f64, time: f64, omega: f64, window: bool) {
        self.peak = self.peak.max(u.abs());
        if self.arrival.is_none() && u.abs() >= 1e-6 {
            self.arrival = Some(time);
        }
        if self.detection.is_none() && u.abs() >= 1e-14 {
            self.detection = Some(time);
        }
        if window {
            self.count += 1;
            self.square += u * u;
            self.velocity_square += v * v;
            self.drive_square += drive * drive;
            self.response_sin += u * (omega * time).sin();
            self.response_cos += u * (omega * time).cos();
            self.drive_sin += drive * (omega * time).sin();
            self.drive_cos += drive * (omega * time).cos();
        }
    }
    fn rms(&self) -> f64 {
        (self.square / number(self.count.max(1))).sqrt()
    }
    fn transfer(&self) -> (f64, f64) {
        let den = self.drive_sin.powi(2) + self.drive_cos.powi(2);
        if den == 0.0 {
            return (0.0, 0.0);
        }
        (
            (self.response_sin * self.drive_sin + self.response_cos * self.drive_cos) / den,
            (self.response_cos * self.drive_sin - self.response_sin * self.drive_cos) / den,
        )
    }
}

fn optional(value: Option<f64>) -> String {
    value.map_or_else(String::new, |v| format!("{v:.17e}"))
}

fn run_trial(
    id: usize,
    trial: &Trial,
    folder: &Path,
    summary: &mut impl Write,
    probe_table: &mut impl Write,
) -> io::Result<()> {
    let full = configuration(trial.seed, trial.class);
    let mut medium = Medium::new(&full, trial.mode, trial.channel, trial.gamma);
    medium.reset();
    let before = medium.structure.clone();
    fs::write(folder.join(format!("state/{id}-p-before.bin")), &before)?;
    let source = trial.source_point();
    if matches!(trial.drive, Drive::Impulse) {
        medium.u[source][trial.channel] = 1.0;
    }
    let (input, _scale) = trial.input();
    let mut input_file = BufWriter::new(File::create(folder.join(format!("inputs/{id}.csv")))?);
    writeln!(input_file, "step,time,drive")?;
    for (step, value) in input.iter().enumerate() {
        writeln!(
            input_file,
            "{step},{:.17e},{value:.17e}",
            number(step) * trial.dt
        )?;
    }
    let observations = probes(trial);
    let mut measures: Vec<_> = (0..observations.len())
        .map(|_| Measurement::default())
        .collect();
    for (index, probe) in observations.iter().enumerate() {
        writeln!(
            probe_table,
            "{id},{index},{},{},{},{},{},{},{:.17e},{}",
            probe.label,
            probe.point,
            probe.row,
            probe.col,
            probe.module,
            probe.hops,
            probe.distance,
            probe.scope
        )?;
    }
    let mut trace = BufWriter::new(File::create(folder.join(format!("traces/{id}.csv")))?);
    let mut diagnostics =
        BufWriter::new(File::create(folder.join(format!("diagnostics/{id}.csv")))?);
    writeln!(trace, "step,probe,u,v")?;
    writeln!(
        diagnostics,
        "step,peak,velocity_peak,sum_u2,sum_v2,nonfinite"
    )?;
    let mut valid = true;
    let mut completed = 0;
    for (step, &drive) in input.iter().enumerate() {
        let diagnostic = if step == 0 {
            medium.diagnostic()
        } else {
            medium.step(trial.dt, source, drive, false)
        };
        valid &= diagnostic.valid();
        completed = step;
        writeln!(
            diagnostics,
            "{step},{:.17e},{:.17e},{:.17e},{:.17e},{}",
            diagnostic.peak,
            diagnostic.velocity_peak,
            diagnostic.square,
            diagnostic.velocity_square,
            diagnostic.nonfinite
        )?;
        for (index, probe) in observations.iter().enumerate() {
            let value = medium.u[probe.point][trial.channel];
            let velocity = medium.v[probe.point][trial.channel];
            writeln!(trace, "{step},{index},{value:.17e},{velocity:.17e}")?;
            measures[index].sample(
                value,
                velocity,
                drive,
                number(step) * trial.dt,
                trial.omega(),
                trial.stage == "T" || step > trial.steps / 2,
            );
        }
        if !valid {
            break;
        }
    }
    export_result(
        id,
        trial,
        folder,
        &medium,
        TrialOutcome {
            measures: &measures,
            valid,
            completed,
            before: &before,
        },
        summary,
    )
}

#[derive(Clone, Copy)]
struct TrialOutcome<'a> {
    measures: &'a [Measurement],
    valid: bool,
    completed: usize,
    before: &'a [u8],
}

fn export_result(
    id: usize,
    trial: &Trial,
    folder: &Path,
    medium: &Medium,
    outcome: TrialOutcome<'_>,
    summary: &mut impl Write,
) -> io::Result<()> {
    let TrialOutcome {
        measures,
        valid,
        completed,
        before,
    } = outcome;
    let source_rms = measures[0].rms();
    for (index, m) in measures.iter().enumerate() {
        let rms = m.rms();
        let drive_rms = (m.drive_square / number(m.count.max(1))).sqrt();
        let (real, imag) = m.transfer();
        writeln!(
            summary,
            "{id},{index},{valid},{completed},{:.17e},{rms:.17e},{:.17e},{:.17e},{drive_rms:.17e},{:.17e},{real:.17e},{imag:.17e},{:.17e},{},{}",
            m.peak,
            (m.velocity_square / number(m.count.max(1))).sqrt(),
            if source_rms > 0.0 {
                rms / source_rms
            } else {
                0.0
            },
            if drive_rms > 0.0 {
                rms / drive_rms
            } else {
                0.0
            },
            imag.atan2(real),
            optional(m.arrival),
            optional(m.detection)
        )?;
    }
    if before != medium.structure.as_slice() {
        return Err(io::Error::other("T/R changed persistent state"));
    }
    fs::write(
        folder.join(format!("state/{id}-p-after.bin")),
        &medium.structure,
    )?;
    let mut state = BufWriter::new(File::create(
        folder.join(format!("state/{id}-uv-final.bin")),
    )?);
    for (u, v) in medium.u.iter().zip(&medium.v) {
        state.write_all(&u[trial.channel].to_le_bytes())?;
        state.write_all(&v[trial.channel].to_le_bytes())?;
    }
    if trial.stage == "T" || id.is_multiple_of(100) || trial.mode == Mode::Full {
        eprintln!(
            "0002 trial {id}: {} {} {} ch{} {}, valid={valid}",
            trial.stage,
            trial.class,
            trial.mode.name(),
            trial.channel + 1,
            trial.drive.name()
        );
    }
    Ok(())
}

/// Execute only the frozen Experiment 0002 transport and resonance stages.
///
/// # Errors
/// Returns an error if the directory already exists or evidence cannot be written.
pub fn run(folder: &Path) -> io::Result<()> {
    fs::create_dir(folder)?;
    for sub in ["inputs", "traces", "diagnostics", "state"] {
        fs::create_dir(folder.join(sub))?;
    }
    fs::write(
        folder.join("registration.md"),
        include_str!("../EXPERIMENT_0002.md"),
    )?;
    fs::write(
        folder.join("engine_source.rs"),
        include_str!("experiment_0002.rs"),
    )?;
    fs::write(
        folder.join("proposal.md"),
        include_str!("../docs/EXPERIMENT_0002_PROPOSAL.md"),
    )?;
    let reference = Path::new("results/experiment_0001-final-a/impulse.csv");
    if reference.is_file() {
        fs::copy(reference, folder.join("historical-0001-impulse.csv"))?;
    }
    fs::write(
        folder.join("parameters.json"),
        "{\"experiment\":\"0002\",\"seeds\":[7,19,41],\"bins\":[4,8,12,16,20,24,28],\"T_trials\":73,\"R_trials\":1028,\"plasticity\":false,\"internal_coefficient\":0.6,\"external_coefficient\":0.4,\"usable_amplitude\":1e-6,\"usable_transfer\":1e-3,\"detectable_amplitude\":1e-14,\"peak_bound\":100.0,\"source_energy\":1.0}\n",
    )?;
    let schedule = trials();
    let mut ledger = BufWriter::new(File::create(folder.join("trials.csv"))?);
    writeln!(
        ledger,
        "id,stage,seed,configuration,mode,side,channel,source,source_point,drive,bin,omega,dt,steps,gamma_factor,input_scale"
    )?;
    for (id, t) in schedule.iter().enumerate() {
        let (_, scale) = t.input();
        writeln!(
            ledger,
            "{id},{},{},{},{},{},{},{},{},{},{},{:.17e},{:.17e},{},{:.17e},{scale:.17e}",
            t.stage,
            t.seed,
            t.class,
            t.mode.name(),
            t.mode.side(),
            t.channel + 1,
            t.source,
            t.source_point(),
            t.drive.name(),
            t.bin,
            t.omega(),
            t.dt,
            t.steps,
            t.gamma
        )?;
    }
    ledger.flush()?;
    let mut summary = BufWriter::new(File::create(folder.join("summary.csv"))?);
    let mut probe_table = BufWriter::new(File::create(folder.join("probes.csv"))?);
    writeln!(
        summary,
        "id,probe,valid,completed_steps,peak,rms,velocity_rms,source_transfer,drive_rms,gain,transfer_re,transfer_im,phase,arrival,detect_arrival"
    )?;
    writeln!(
        probe_table,
        "id,probe,label,point,row,col,module,hops,distance,scope"
    )?;
    for (id, t) in schedule.iter().enumerate() {
        run_trial(id, t, folder, &mut summary, &mut probe_table)?;
        summary.flush()?;
        probe_table.flush()?;
    }
    fs::write(
        folder.join("COMPLETE"),
        "All 1101 T/R cases attempted; scientific decisions require independent analysis. Stage P not run.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_coverage_and_independent_histogram_preserving_structures() {
        let schedule = trials();
        assert_eq!(schedule.iter().filter(|t| t.stage == "T").count(), 73);
        assert_eq!(schedule.iter().filter(|t| t.stage == "R").count(), 1028);
        let original = configuration(7, "heterogeneous");
        let rearranged = configuration(7, "rearranged");
        assert_eq!(original.len(), 4096 * 64);
        assert_ne!(original, rearranged);
        assert_eq!(original, configuration(7, "heterogeneous"));
        for (old, new) in original
            .as_chunks::<64>()
            .0
            .iter()
            .zip(rearranged.as_chunks::<64>().0)
        {
            assert_eq!(old[27], new[27]);
            let (mut old, mut new) = (*old, *new);
            old.sort_unstable();
            new.sort_unstable();
            assert_eq!(old, new);
        }
        let mut changed = original.clone();
        changed[123 * 64 + 5] = (changed[123 * 64 + 5] + 1) % 5;
        assert_eq!(&changed[..123 * 64], &original[..123 * 64]);
        assert_eq!(&changed[124 * 64..], &original[124 * 64..]);
    }

    #[test]
    fn two_scale_neighbor_boundaries_and_finite_speed() {
        assert_eq!(adjacent(0, 64, true), vec![1, 64, 65]);
        assert_eq!(adjacent(27, 8, false).len(), 4);
        assert_eq!(adjacent(0, 8, false), vec![1, 8]);
        let full = configuration(7, "homogeneous");
        let mut medium = Medium::new(&full, Mode::Full, 0, 1.0);
        let source = (32 * 64 + 32) * 64 + 27;
        medium.u[source][0] = 1.0;
        medium.step(0.1, source, 0.0, false);
        assert!(medium.u[source + 1][0] > 0.0);
        assert!(medium.u[source + 64][0] > 0.0);
        assert!(medium.u[source + 2][0].abs() < 1e-15);
        assert!(medium.u[source + 128][0].abs() < 1e-15);
        assert!(medium.u[27][0].abs() < 1e-15);
    }

    #[test]
    fn snapshot_updates_are_traversal_independent_and_reset_preserves_structure() {
        let full = configuration(19, "heterogeneous");
        let mut forward = Medium::new(&full, Mode::Patch, 2, 1.0);
        let mut reverse = Medium::new(&full, Mode::Patch, 2, 1.0);
        forward.u[4 * 64 + 27][2] = 1.0;
        reverse.u = forward.u.clone();
        forward.step(0.1, 4 * 64 + 27, 0.2, false);
        reverse.step(0.1, 4 * 64 + 27, 0.2, true);
        assert_eq!(forward.u, reverse.u);
        assert_eq!(forward.v, reverse.v);
        let structure = forward.structure.clone();
        forward.reset();
        assert_eq!(forward.structure, structure);
        assert!(
            forward
                .u
                .iter()
                .flatten()
                .chain(forward.v.iter().flatten())
                .all(|x| x.abs() < 1e-15)
        );
    }

    #[test]
    fn independent_channels_zero_drive_and_all_field_invalid_detection() {
        let full = configuration(7, "heterogeneous");
        let mut medium = Medium::new(&full, Mode::Patch, 1, 1.0);
        for _ in 0..5 {
            medium.step(0.1, 27, 0.0, false);
        }
        assert!(medium.diagnostic().square.abs() < 1e-15);
        medium.step(0.1, 27, 0.3, false);
        assert!(medium.u[27][1] > 0.0);
        for values in &medium.u {
            assert!(values[0].abs() < 1e-15 && values[2].abs() < 1e-15 && values[3].abs() < 1e-15);
        }
        medium.v[0][3] = f64::NAN;
        assert!(!medium.diagnostic().valid());
        medium.reset();
        medium.u[0][0] = 101.0;
        assert!(!medium.diagnostic().valid());
    }

    fn magnitude(samples: &[f64], bin: usize, dt: f64) -> f64 {
        let (mut real, mut imag) = (0.0, 0.0);
        for (index, value) in samples.iter().enumerate().skip(1) {
            let angle = TAU * number(bin) / 64.0 * number(index) * dt;
            real += value * angle.cos();
            imag += value * angle.sin();
        }
        real.hypot(imag)
    }

    #[test]
    fn equal_energy_equal_amplitude_and_phase_randomization_controls() {
        let mut tone = trials().into_iter().find(|t| t.stage == "R").unwrap();
        let mut scales = Vec::new();
        for bin in BINS {
            tone.bin = bin;
            let (values, scale) = tone.input();
            scales.push(scale);
            let energy: f64 = values.iter().map(|x| x * x * tone.dt).sum();
            assert!((energy - 1.0).abs() < 1e-12);
        }
        assert!(scales.iter().all(|x| (x - scales[0]).abs() < 1e-12));
        tone.drive = Drive::Multi;
        let (coherent, _) = tone.input();
        tone.drive = Drive::RandomPhase;
        let (random, _) = tone.input();
        assert_ne!(coherent, random);
        for bin in [8, 16, 24] {
            assert!(
                (magnitude(&coherent, bin, tone.dt) - magnitude(&random, bin, tone.dt)).abs()
                    < 1e-10
            );
        }
    }

    #[test]
    fn scalar_half_step_converges_toward_independent_analytic_solution() {
        let full = configuration(7, "homogeneous");
        let mut errors = Vec::new();
        for (dt, steps) in [(0.1, 10), (0.05, 20)] {
            let mut medium = Medium::new(&full, Mode::Scalar, 0, 1.0);
            let omega2 = medium.omega_squared[27];
            let gamma = medium.damping[27];
            let wd = (omega2 - gamma * gamma / 4.0).sqrt();
            let exact = (-gamma / 2.0).exp() * (wd.cos() + gamma / (2.0 * wd) * wd.sin());
            medium.u[27][0] = 1.0;
            for _ in 0..steps {
                medium.step(dt, 27, 0.0, false);
            }
            errors.push((medium.u[27][0] - exact).abs());
        }
        assert!(errors[1] < errors[0]);
    }

    #[test]
    fn same_histogram_shapes_and_palette_change_intrinsic_response() {
        let original = configuration(7, "heterogeneous");
        let reshaped = configuration(7, "rearranged");
        let mut normal = Medium::new(&original, Mode::Isolated, 0, 1.0);
        let mut shape = Medium::new(&reshaped, Mode::Isolated, 0, 1.0);
        let different_channel = Medium::new(&original, Mode::Isolated, 1, 1.0);
        assert_ne!(normal.omega_squared, different_channel.omega_squared);
        for step in 1..=640 {
            let drive = (1.2 * number(step) * 0.1).sin() * 0.1;
            normal.step(0.1, 27, drive, false);
            shape.step(0.1, 27, drive, false);
        }
        assert!((normal.u[27][0] - shape.u[27][0]).abs() > 1e-6);
    }
}
