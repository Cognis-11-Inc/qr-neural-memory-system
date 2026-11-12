//! Experiment 0001: independently mutable neurons, local and ripple candidate rules.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

const MODULE_SIDE: usize = 8;
const MODULES: usize = MODULE_SIDE * MODULE_SIDE;
const SIDE: usize = 64;
const CYCLES: usize = 80;
const SEEDS: [u64; 3] = [7, 19, 41];
type Tile = [u8; MODULES];
type WaveTile = [[f64; 4]; MODULES];
type Pattern = [Tile; 16];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Local,
    Ripple,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Ripple => "ripple",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    Normal,
    Frozen,
    Color,
    Shape,
}

impl Variant {
    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Frozen => "frozen",
            Self::Color => "color",
            Self::Shape => "shape",
        }
    }

    fn transform(self, tile: &mut Tile) {
        match self {
            Self::Color => tile.iter_mut().for_each(|x| *x = u8::from(*x != 0)),
            Self::Shape => tile.sort_unstable(),
            Self::Normal | Self::Frozen => (),
        }
    }

    fn pattern(self, mut pattern: Pattern) -> Pattern {
        for tile in &mut pattern {
            self.transform(tile);
        }
        pattern
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn pattern(&mut self) -> Pattern {
        let mut pattern = [[0; MODULES]; 16];
        for tile in &mut pattern {
            for module in tile {
                let value = self.next();
                *module = if value.is_multiple_of(4) {
                    0
                } else {
                    1 + u8::try_from((value >> 8) % 4).expect("palette range")
                };
            }
        }
        pattern
    }
}

fn neighbors(index: usize, side: usize, diagonal: bool) -> Vec<usize> {
    let row = index / side;
    let col = index % side;
    let mut result = Vec::with_capacity(8);
    for r in row.saturating_sub(1)..=(row + 1).min(side - 1) {
        for c in col.saturating_sub(1)..=(col + 1).min(side - 1) {
            if (r != row || c != col) && (diagonal || r == row || c == col) {
                result.push(r * side + c);
            }
        }
    }
    result
}

fn patch(side: usize, output: bool) -> [usize; 16] {
    std::array::from_fn(|i| {
        (side / 2 - 2 + i / 4) * side + if output { side - 4 } else { 0 } + i % 4
    })
}

fn incomplete(mut pattern: Pattern) -> Pattern {
    for index in (1..16).step_by(2) {
        pattern[index] = [0; MODULES];
    }
    pattern
}

fn transition(color: u8, trace: u8) -> u8 {
    if color == 0 || trace == 0 {
        color
    } else {
        1 + (color - 1 + trace - 1) % 4
    }
}

fn learn(incoming: u8, target: u8) -> u8 {
    if incoming == 0 || target == 0 {
        0
    } else {
        1 + (target + 4 - incoming) % 4
    }
}

fn dominant(channels: &[f64; 4]) -> u8 {
    let mut largest = 1e-12;
    let mut color = 0;
    for (index, &amplitude) in channels.iter().enumerate() {
        if amplitude > largest {
            largest = amplitude;
            color = u8::try_from(index + 1).expect("four channels");
        }
    }
    color
}

fn conductance(color: u8, channel: usize) -> f64 {
    if color == 0 {
        0.25
    } else if usize::from(color) == channel + 1 {
        1.0
    } else {
        0.5
    }
}

#[derive(Clone)]
struct Medium {
    side: usize,
    kind: Kind,
    configuration: Vec<Tile>,
    signals: Vec<Tile>,
    amplitude: Vec<WaveTile>,
    previous: Vec<WaveTile>,
    adjacent: Vec<Vec<usize>>,
    internal: Vec<Vec<usize>>,
}

impl Medium {
    fn new(side: usize, kind: Kind) -> Self {
        assert!(side >= 8);
        let waves = if kind == Kind::Ripple { side * side } else { 0 };
        Self {
            side,
            kind,
            configuration: vec![[0; MODULES]; side * side],
            signals: vec![[0; MODULES]; side * side],
            amplitude: vec![[[0.0; 4]; MODULES]; waves],
            previous: vec![[[0.0; 4]; MODULES]; waves],
            adjacent: (0..side * side).map(|i| neighbors(i, side, true)).collect(),
            internal: (0..MODULES)
                .map(|i| neighbors(i, MODULE_SIDE, false))
                .collect(),
        }
    }

    fn reset(&mut self) {
        self.signals.fill([0; MODULES]);
        self.amplitude.fill([[0.0; 4]; MODULES]);
        self.previous.fill([[0.0; 4]; MODULES]);
    }

    fn vote(&self, neuron: usize, module: usize) -> u8 {
        let mut counts = [0u8; 5];
        for &neighbor in &self.adjacent[neuron] {
            counts[usize::from(self.signals[neighbor][module])] += 1;
        }
        let mut best = 0;
        for color in 1..5 {
            if counts[color] > counts[best] || best == 0 {
                // Empty votes never compete with colors, including at boundaries.
                if counts[color] > 0 {
                    best = color;
                }
            }
        }
        u8::try_from(best).expect("palette range")
    }

    fn wave_step(&self, neuron: usize, module: usize, gamma: f64) -> [f64; 4] {
        std::array::from_fn(|channel| {
            let current = self.amplitude[neuron][module][channel];
            let g = conductance(self.configuration[neuron][module], channel);
            let mut laplacian = 0.0;
            for &neighbor in &self.adjacent[neuron] {
                let weight = g * conductance(self.configuration[neighbor][module], channel);
                laplacian += weight * (self.amplitude[neighbor][module][channel] - current);
            }
            for &neighbor in &self.internal[module] {
                let weight = g * conductance(self.configuration[neuron][neighbor], channel);
                laplacian += weight * (self.amplitude[neuron][neighbor][channel] - current);
            }
            (2.0 - gamma) * current - (1.0 - gamma) * self.previous[neuron][module][channel]
                + (0.2 / 12.0) * laplacian
        })
    }

    fn step(&mut self, cue: &Pattern, teacher: Option<&Pattern>, variant: Variant, reverse: bool) {
        let mut next_signals = vec![[0; MODULES]; self.configuration.len()];
        let mut next_waves = vec![[[0.0; 4]; MODULES]; self.amplitude.len()];
        let mut next_configuration = self.configuration.clone();
        for position in 0..self.configuration.len() {
            let neuron = if reverse {
                self.configuration.len() - 1 - position
            } else {
                position
            };
            for module in 0..MODULES {
                match self.kind {
                    Kind::Local => {
                        next_signals[neuron][module] = transition(
                            self.vote(neuron, module),
                            self.configuration[neuron][module],
                        );
                    }
                    Kind::Ripple => {
                        next_waves[neuron][module] = self.wave_step(neuron, module, 0.1);
                    }
                }
            }
        }
        if let Some(target) = teacher.filter(|_| variant != Variant::Frozen) {
            for (tile, neuron) in patch(self.side, true).into_iter().enumerate() {
                for module in 0..MODULES {
                    let raw = match self.kind {
                        Kind::Local => self.vote(neuron, module),
                        Kind::Ripple => dominant(&self.amplitude[neuron][module]),
                    };
                    next_configuration[neuron][module] = learn(raw, target[tile][module]);
                }
                variant.transform(&mut next_configuration[neuron]);
            }
        }
        for (tile, neuron) in patch(self.side, false).into_iter().enumerate() {
            for module in 0..MODULES {
                match self.kind {
                    Kind::Local => next_signals[neuron][module] = cue[tile][module],
                    Kind::Ripple if cue[tile][module] != 0 => {
                        next_waves[neuron][module][usize::from(cue[tile][module] - 1)] += 0.02;
                    }
                    Kind::Ripple => (),
                }
            }
        }
        self.configuration = next_configuration;
        self.signals = next_signals;
        self.previous = std::mem::replace(&mut self.amplitude, next_waves);
    }

    fn train(&mut self, cue: &Pattern, target: &Pattern, variant: Variant, cycles: usize) {
        self.reset();
        for _ in 0..cycles {
            self.step(cue, Some(target), variant, false);
        }
    }

    // This API has no target, pair table, evaluator, or training-data argument.
    fn recall(&mut self, cue: &Pattern, cycles: usize) -> Pattern {
        self.reset();
        for _ in 0..cycles {
            self.step(cue, None, Variant::Normal, false);
        }
        std::array::from_fn(|tile| {
            let neuron = patch(self.side, true)[tile];
            std::array::from_fn(|module| match self.kind {
                Kind::Local => self.signals[neuron][module],
                Kind::Ripple => transition(
                    dominant(&self.amplitude[neuron][module]),
                    self.configuration[neuron][module],
                ),
            })
        })
    }

    fn snapshot(&self, path: &Path) -> io::Result<()> {
        let bytes: Vec<u8> = self.configuration.iter().flatten().copied().collect();
        fs::write(path, bytes)
    }
}

#[derive(Clone, Copy, Debug)]
struct Metrics {
    palette: f64,
    shape: f64,
    color: f64,
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        f64::from(u32::try_from(numerator).expect("small count"))
            / f64::from(u32::try_from(denominator).expect("small count"))
    }
}

fn metrics(actual: &Pattern, target: &Pattern) -> Metrics {
    let mut palette = 0;
    let mut shape = 0;
    let mut color = 0;
    let mut occupied = 0;
    for (&a, &t) in actual.iter().flatten().zip(target.iter().flatten()) {
        palette += usize::from(a != t);
        shape += usize::from((a == 0) != (t == 0));
        if t != 0 {
            occupied += 1;
            color += usize::from(a != t);
        }
    }
    Metrics {
        palette: ratio(palette, 16 * MODULES),
        shape: ratio(shape, 16 * MODULES),
        color: ratio(color, occupied),
    }
}

fn record_changes(
    before: &[Tile],
    after: &[Tile],
    prefix: &str,
    csv: &mut String,
) -> (usize, usize) {
    let mut neurons = 0;
    let mut modules = 0;
    for (index, (a, b)) in before.iter().zip(after).enumerate() {
        let changed = a.iter().zip(b).filter(|(x, y)| x != y).count();
        if changed > 0 {
            neurons += 1;
            modules += changed;
            writeln!(
                csv,
                "{prefix},{index},{},{},{changed}",
                index / SIDE,
                index % SIDE
            )
            .expect("string write");
        }
    }
    (neurons, modules)
}

struct Tables {
    metrics: String,
    verdicts: String,
    changes: String,
    activity: String,
    raw: Vec<u8>,
}

impl Tables {
    fn new() -> Self {
        Self {
            metrics: "seed,engine,variant,condition,palette_error,shape_error,occupied_color_error\n".into(),
            verdicts: "seed,engine,variant,ab_pass,cd_pass,retention_pass,ab_neurons,ab_modules,cd_neurons,cd_modules\n".into(),
            changes: "seed,engine,variant,stage,neuron,row,column,changed_modules\n".into(),
            activity: "seed,engine,variant,condition,max_amplitude,output_max_amplitude,nonfinite\n".into(),
            raw: Vec::new(),
        }
    }

    fn measure(
        &mut self,
        medium: &mut Medium,
        cue: &Pattern,
        target: &Pattern,
        prefix: &str,
    ) -> Metrics {
        let actual = medium.recall(cue, CYCLES);
        self.raw.extend(actual.iter().flatten());
        let result = metrics(&actual, target);
        writeln!(
            self.metrics,
            "{prefix},{:.9},{:.9},{:.9}",
            result.palette, result.shape, result.color
        )
        .expect("string write");
        let (peak, nonfinite) = amplitude_stats(medium);
        let output_peak = if medium.kind == Kind::Ripple {
            patch(medium.side, true)
                .iter()
                .flat_map(|&neuron| medium.amplitude[neuron].iter().flatten())
                .fold(0.0_f64, |maximum, value| maximum.max(value.abs()))
        } else {
            0.0
        };
        writeln!(
            self.activity,
            "{prefix},{peak:.12e},{output_peak:.12e},{nonfinite}"
        )
        .expect("string write");
        result
    }
}

fn amplitude_stats(medium: &Medium) -> (f64, usize) {
    let mut peak = 0.0_f64;
    let mut nonfinite = 0;
    for value in medium.amplitude.iter().flatten().flatten() {
        if value.is_finite() {
            peak = peak.max(value.abs());
        } else {
            nonfinite += 1;
        }
    }
    (peak, nonfinite)
}

fn association_pass(trained: Metrics, untrained: Metrics, unrelated: Metrics) -> bool {
    trained.palette <= 0.10
        && untrained.palette - trained.palette >= 0.15
        && unrelated.palette - trained.palette >= 0.15
}

fn run_trial(
    seed: u64,
    kind: Kind,
    variant: Variant,
    output: &Path,
    tables: &mut Tables,
) -> io::Result<()> {
    let mut rng = Rng(seed);
    let [original_a, original_b, original_c, original_d, original_u] =
        std::array::from_fn(|_| rng.pattern());
    let [input_a, teach_b, input_c, teach_d, unrelated] =
        [original_a, original_b, original_c, original_d, original_u]
            .map(|pattern| variant.pattern(pattern));
    for (label, pattern) in [
        ("a", original_a),
        ("b", original_b),
        ("c", original_c),
        ("d", original_d),
        ("u", original_u),
    ] {
        fs::write(
            output.join(format!("seed-{seed}-stimulus-{label}.bin")),
            pattern.into_iter().flatten().collect::<Vec<_>>(),
        )?;
    }
    let mut medium = Medium::new(SIDE, kind);
    let prefix = format!("{seed},{},{}", kind.name(), variant.name());
    let basename = format!("seed-{seed}-{}-{}", kind.name(), variant.name());
    medium.snapshot(&output.join(format!("{basename}-initial.bin")))?;
    let initial = medium.configuration.clone();
    let untrained = tables.measure(
        &mut medium,
        &incomplete(input_a),
        &original_b,
        &format!("{prefix},untrained_ab"),
    );
    // Independently measure the untrained D baseline using C, not the B score.
    let untrained_cd = tables.measure(
        &mut medium,
        &incomplete(input_c),
        &original_d,
        &format!("{prefix},untrained_cd"),
    );
    medium.train(&input_a, &teach_b, variant, CYCLES);
    let ab_state = medium.configuration.clone();
    medium.snapshot(&output.join(format!("{basename}-ab.bin")))?;
    let (ab_neurons, ab_modules) = record_changes(
        &initial,
        &ab_state,
        &format!("{prefix},ab"),
        &mut tables.changes,
    );
    let trained = tables.measure(
        &mut medium,
        &incomplete(input_a),
        &original_b,
        &format!("{prefix},trained_ab"),
    );
    let unrelated_ab = tables.measure(
        &mut medium,
        &incomplete(unrelated),
        &original_b,
        &format!("{prefix},unrelated_ab"),
    );
    medium.train(&input_c, &teach_d, variant, CYCLES);
    medium.snapshot(&output.join(format!("{basename}-cd.bin")))?;
    let (cd_neurons, cd_modules) = record_changes(
        &ab_state,
        &medium.configuration,
        &format!("{prefix},cd"),
        &mut tables.changes,
    );
    let retained = tables.measure(
        &mut medium,
        &incomplete(input_a),
        &original_b,
        &format!("{prefix},retained_ab"),
    );
    let learned_cd = tables.measure(
        &mut medium,
        &incomplete(input_c),
        &original_d,
        &format!("{prefix},trained_cd"),
    );
    let unrelated_cd = tables.measure(
        &mut medium,
        &incomplete(unrelated),
        &original_d,
        &format!("{prefix},unrelated_cd"),
    );
    let ab_pass = association_pass(trained, untrained, unrelated_ab);
    let cd_pass = association_pass(learned_cd, untrained_cd, unrelated_cd);
    let retention = ab_pass && retained.palette - trained.palette <= 0.05;
    writeln!(tables.verdicts, "{prefix},{ab_pass},{cd_pass},{retention},{ab_neurons},{ab_modules},{cd_neurons},{cd_modules}")
        .expect("string write");
    eprintln!(
        "{prefix}: AB={:.4}, retained={:.4}, CD={:.4}; passes={ab_pass}/{cd_pass}/{retention}",
        trained.palette, retained.palette, learned_cd.palette
    );
    Ok(())
}

fn impulse_diagnostic(output: &Path) -> io::Result<()> {
    let mut medium = Medium::new(SIDE, Kind::Ripple);
    let center = (SIDE / 2) * SIDE + SIDE / 2;
    let module = 4 * MODULE_SIDE + 4;
    medium.amplitude[center][module][0] = 1.0;
    let mut csv = "cycle,max_amplitude,sum_squares,momentum_squares,nonfinite,cardinal,diagonal\n"
        .to_string();
    let samples = [0, 1, 2, 4, 8, 16, 32, 64, 160];
    let mut worst_peak = 0.0_f64;
    let mut nonfinite_observations = 0;
    for cycle in 0..=160 {
        let (peak, nonfinite) = amplitude_stats(&medium);
        worst_peak = worst_peak.max(peak);
        nonfinite_observations += nonfinite;
        if samples.contains(&cycle) {
            let energy: f64 = medium
                .amplitude
                .iter()
                .flatten()
                .flatten()
                .map(|x| x * x)
                .sum();
            let momentum: f64 = medium
                .amplitude
                .iter()
                .flatten()
                .flatten()
                .zip(medium.previous.iter().flatten().flatten())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            writeln!(
                csv,
                "{cycle},{peak:.12e},{energy:.12e},{momentum:.12e},{nonfinite},{:.12e},{:.12e}",
                medium.amplitude[center + 1][module][0],
                medium.amplitude[center + SIDE + 1][module][0]
            )
            .expect("string write");
        }
        if cycle < 160 {
            medium.step(&[[0; MODULES]; 16], None, Variant::Normal, false);
        }
    }
    fs::write(output.join("impulse.csv"), csv)?;
    let passed = nonfinite_observations == 0 && worst_peak <= 10.0;
    fs::write(
        output.join("stability.json"),
        format!(
            "{{\"finite_run_screen_pass\":{passed},\"cycles\":160,\"all_cycles_peak\":{worst_peak:.12e},\"nonfinite_observations\":{nonfinite_observations},\"peak_limit\":10.0}}\n"
        ),
    )
}

/// Run all registered Experiment 0001 trials and export raw local evidence.
///
/// The output directory must not already exist. This prevents accidental replacement
/// of previous evidence. No retrieval target is passed to the inference method.
///
/// # Errors
/// Returns an error if the output directory exists or evidence cannot be written.
pub fn run(output: &Path) -> io::Result<()> {
    fs::create_dir(output)?;
    fs::write(
        output.join("specification.md"),
        include_str!("../EXPERIMENT_0001.md"),
    )?;
    fs::write(
        output.join("engine_source.rs"),
        include_str!("experiment.rs"),
    )?;
    fs::write(
        output.join("parameters.json"),
        "{\"experiment\":\"0001\",\"seeds\":[7,19,41],\"network_side\":64,\"neurons\":4096,\"module_side\":8,\"channels\":4,\"cycles\":80,\"gamma\":0.1,\"c_squared\":0.2,\"source\":0.02,\"decode_threshold\":1e-12,\"rng\":\"SplitMix64\",\"missing_input_tiles\":[1,3,5,7,9,11,13,15]}\n",
    )?;
    let mut tables = Tables::new();
    impulse_diagnostic(output)?;
    for seed in SEEDS {
        for kind in [Kind::Local, Kind::Ripple] {
            for variant in [
                Variant::Normal,
                Variant::Frozen,
                Variant::Color,
                Variant::Shape,
            ] {
                run_trial(seed, kind, variant, output, &mut tables)?;
                // Flush complete trials, preserving partial evidence if a later run fails.
                fs::write(output.join("metrics.csv"), &tables.metrics)?;
                fs::write(output.join("verdicts.csv"), &tables.verdicts)?;
                fs::write(output.join("changes.csv"), &tables.changes)?;
                fs::write(output.join("activity.csv"), &tables.activity)?;
                fs::write(output.join("outputs.bin"), &tables.raw)?;
            }
        }
    }
    fs::write(
        output.join("COMPLETE"),
        "24 registered trials completed; inspect verdicts.csv, not this marker, for scientific outcomes.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_neuron_owns_an_independent_spatial_configuration() {
        let mut medium = Medium::new(SIDE, Kind::Local);
        assert_eq!(medium.configuration.len(), crate::CELL_COUNT);
        for (index, tile) in medium.configuration.iter_mut().enumerate() {
            tile[index % MODULES] = 1 + u8::try_from(index % 4).unwrap();
        }
        let before = medium.configuration.clone();
        medium.configuration[123][17] = 4;
        assert_eq!(&medium.configuration[..123], &before[..123]);
        assert_eq!(&medium.configuration[124..], &before[124..]);
        let mut shape_a = [0; MODULES];
        shape_a[1] = 2;
        let mut shape_b = [0; MODULES];
        shape_b[8] = 2;
        assert_ne!(shape_a, shape_b);
        shape_a.sort_unstable();
        shape_b.sort_unstable();
        assert_eq!(shape_a, shape_b);
    }

    #[test]
    fn colors_and_missing_output_affect_the_defined_metrics() {
        let mut target = [[0; MODULES]; 16];
        let mut actual = target;
        target[0][0] = 2;
        actual[0][0] = 3;
        let result = metrics(&actual, &target);
        assert!((result.palette - 1.0 / 1024.0).abs() < 1e-15);
        assert!(result.shape.abs() < 1e-15);
        assert!((result.color - 1.0).abs() < 1e-15);
        actual[0][0] = 0;
        assert!((metrics(&actual, &target).shape - 1.0 / 1024.0).abs() < 1e-15);
        assert!(metrics(&target, &[[0; MODULES]; 16]).color.abs() < 1e-15);
        for incoming in 1..=4 {
            for desired in 1..=4 {
                assert_eq!(transition(incoming, learn(incoming, desired)), desired);
            }
        }
    }

    #[test]
    fn boundaries_and_two_scale_finite_speed_are_explicit() {
        assert_eq!(neighbors(0, SIDE, true), vec![1, SIDE, SIDE + 1]);
        assert_eq!(neighbors(SIDE + 1, SIDE, true).len(), 8);
        assert_eq!(neighbors(0, MODULE_SIDE, false), vec![1, MODULE_SIDE]);
        let mut medium = Medium::new(8, Kind::Ripple);
        let neuron = 4 * 8 + 4;
        let module = 4 * MODULE_SIDE + 4;
        medium.amplitude[neuron][module][0] = 1.0;
        medium.step(&[[0; MODULES]; 16], None, Variant::Normal, false);
        assert!(medium.amplitude[neuron][module + 1][0] > 0.0);
        assert!(medium.amplitude[neuron + 1][module][0] > 0.0);
        assert!(medium.amplitude[neuron + 2][module][0].abs() < 1e-15);
        assert!(medium.amplitude[neuron][module + 2][0].abs() < 1e-15);
    }

    #[test]
    fn old_snapshots_make_traversal_order_irrelevant() {
        for kind in [Kind::Local, Kind::Ripple] {
            let mut forward = Medium::new(8, kind);
            let mut rng = Rng(7);
            let cue = rng.pattern();
            let teacher = rng.pattern();
            for (index, tile) in forward.configuration.iter_mut().enumerate() {
                *tile = cue[index % 16];
            }
            forward.step(&cue, None, Variant::Normal, false);
            let mut reverse = forward.clone();
            forward.step(&cue, Some(&teacher), Variant::Normal, false);
            reverse.step(&cue, Some(&teacher), Variant::Normal, true);
            assert_eq!(forward.configuration, reverse.configuration);
            assert_eq!(forward.signals, reverse.signals);
            assert_eq!(forward.amplitude, reverse.amplitude);
            assert_eq!(forward.previous, reverse.previous);
        }
    }

    #[test]
    fn color_and_shape_change_wave_propagation_not_just_rendering() {
        let mut medium = Medium::new(8, Kind::Ripple);
        let neuron = 4 * 8 + 4;
        let module = 4 * MODULE_SIDE + 4;
        medium.amplitude[neuron][module][0] = 1.0;
        let empty = medium.wave_step(neuron, module, 0.1);
        medium.configuration[neuron][module] = 1;
        let red = medium.wave_step(neuron, module, 0.1);
        medium.configuration[neuron][module] = 2;
        let green = medium.wave_step(neuron, module, 0.1);
        assert!((empty[0] - red[0]).abs() > 1e-6);
        assert!((red[0] - green[0]).abs() > 1e-6);
        medium.configuration[neuron][module] = 0;
        medium.configuration[neuron][module + 1] = 2;
        let moved = medium.wave_step(neuron, module, 0.1);
        assert!((moved[0] - green[0]).abs() > 1e-6);
    }

    #[test]
    fn reset_erases_amplitude_and_momentum_but_not_configuration() {
        for kind in [Kind::Local, Kind::Ripple] {
            let mut medium = Medium::new(8, kind);
            medium.configuration[0][0] = 3;
            medium.signals[0][0] = 2;
            if kind == Kind::Ripple {
                medium.amplitude[0][0][0] = 0.5;
                medium.previous[0][0][0] = -0.2;
            }
            medium.reset();
            assert_eq!(medium.configuration[0][0], 3);
            assert!(medium.signals.iter().flatten().all(|&x| x == 0));
            assert!(
                medium
                    .amplitude
                    .iter()
                    .flatten()
                    .flatten()
                    .all(|x| x.abs() < 1e-15)
            );
            assert!(
                medium
                    .previous
                    .iter()
                    .flatten()
                    .flatten()
                    .all(|x| x.abs() < 1e-15)
            );
        }
    }

    #[test]
    fn damping_reduces_a_uniform_velocity_and_rng_fixture_is_fixed() {
        let mut medium = Medium::new(8, Kind::Ripple);
        medium.amplitude.fill([[1.0; 4]; MODULES]);
        let damped = medium.wave_step(0, 0, 0.1)[0];
        let undamped = medium.wave_step(0, 0, 0.0)[0];
        assert!(damped < undamped);
        assert_eq!(Rng(0).next(), 0xe220_a839_7b1d_cdaf);
    }

    #[test]
    fn end_to_end_controls_reset_sequential_learning_and_no_evaluator_leak() {
        for kind in [Kind::Local, Kind::Ripple] {
            let mut rng = Rng(19);
            let [cue, target, next_cue, next_target, unrelated] =
                std::array::from_fn(|_| rng.pattern());
            let mut medium = Medium::new(8, kind);
            let untrained = medium.recall(&incomplete(cue), 16);
            let frozen_initial = medium.configuration.clone();
            medium.train(&cue, &target, Variant::Frozen, 16);
            assert_eq!(medium.configuration, frozen_initial);
            assert_eq!(untrained, medium.recall(&incomplete(cue), 16));
            medium.train(&cue, &target, Variant::Normal, 16);
            let trained = medium.configuration.clone();
            assert!(
                trained
                    .iter()
                    .filter(|tile| tile.iter().any(|&x| x != 0))
                    .count()
                    > 1
            );
            let result = medium.recall(&incomplete(cue), 16);
            let _unrelated_result = medium.recall(&incomplete(unrelated), 16);
            let mut evaluator_target = target;
            let _score = metrics(&result, &evaluator_target);
            evaluator_target.fill([4; MODULES]);
            assert_ne!(evaluator_target, target);
            assert_eq!(result, medium.recall(&incomplete(cue), 16));
            assert_eq!(trained, medium.configuration);
            let mut replay = medium.clone();
            medium.train(&next_cue, &next_target, Variant::Normal, 16);
            replay.train(&next_cue, &next_target, Variant::Normal, 16);
            assert_ne!(medium.configuration, trained);
            assert_eq!(medium.configuration, replay.configuration);
            assert_eq!(
                medium.recall(&incomplete(cue), 16),
                replay.recall(&incomplete(cue), 16)
            );
        }
    }
}
