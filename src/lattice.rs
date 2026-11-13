//! Experiment 0003's headless topology adapter, retaining the frozen 0002 equations.

use crate::observation::{Neuron, Snapshot};
use crate::topology::{POPULATION, Topology};
use std::f64::consts::PI;

/// Modules in each neuron's independently owned internal spatial configuration.
pub const MODULES: usize = 64;
/// Fixed nondimensional integration step.
pub const DT: f64 = 0.1;
/// Registered bounded run length.
pub const TICKS: u64 = 160;
/// Physics/observation provenance, not a claim of measured resonance.
pub const ENGINE_VERSION: &str = "qr3-oscillator-v1;physics=0002;registration=5a112c1";

/// Persistent configuration intervention, inherited from Experiment 0002.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Configuration {
    /// Seeded palette values 0–4.
    Heterogeneous = 0,
    /// All modules red/1.
    Homogeneous = 1,
    /// Same per-neuron histogram, sorted except for fixed module 27.
    Rearranged = 2,
}
impl Configuration {
    /// Stable export label.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Heterogeneous => "heterogeneous",
            Self::Homogeneous => "homogeneous",
            Self::Rearranged => "rearranged",
        }
    }
}
impl TryFrom<u8> for Configuration {
    type Error = std::io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Heterogeneous),
            1 => Ok(Self::Homogeneous),
            2 => Ok(Self::Rearranged),
            _ => Err(std::io::Error::other("unknown configuration")),
        }
    }
}

/// Explicit externally applied forcing schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Drive {
    /// Unit initial displacement and no later forcing.
    Impulse = 0,
    /// Equal-energy tone at angular pi/4.
    LowTone = 1,
    /// Equal-energy tone at angular pi/2.
    HighTone = 2,
    /// Zero initial state and zero drive.
    Zero = 3,
}
impl Drive {
    /// Stable waveform label.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Impulse => "impulse",
            Self::LowTone => "tone_pi_4",
            Self::HighTone => "tone_pi_2",
            Self::Zero => "zero",
        }
    }
    /// Angular forcing frequency, or zero for unforced cases.
    #[must_use]
    pub fn omega(self) -> f64 {
        match self {
            Self::LowTone => PI / 4.0,
            Self::HighTone => PI / 2.0,
            _ => 0.0,
        }
    }
}
impl TryFrom<u8> for Drive {
    type Error = std::io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Impulse),
            1 => Ok(Self::LowTone),
            2 => Ok(Self::HighTone),
            3 => Ok(Self::Zero),
            _ => Err(std::io::Error::other("unknown forcing")),
        }
    }
}

/// Complete fixed fixture identity; visual settings are deliberately absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Case {
    /// Open-boundary external neighbor policy.
    pub topology: Topology,
    /// Reproducible initialization seed.
    pub seed: u64,
    /// Persistent initial-configuration class.
    pub configuration: Configuration,
    /// Stable source-neuron ID.
    pub source: usize,
    /// Active independent color channel (0–3).
    pub channel: usize,
    /// Registered excitation.
    pub drive: Drive,
}
impl Default for Case {
    fn default() -> Self {
        Self {
            topology: Topology::Cubic6,
            seed: 7,
            configuration: Configuration::Heterogeneous,
            source: 2184,
            channel: 0,
            drive: Drive::Impulse,
        }
    }
}

/// Every-step boundedness and squared-field diagnostics, not physical energy.
#[derive(Clone, Copy, Debug, Default)]
pub struct Diagnostic {
    /// Global maximum module/channel displacement magnitude.
    pub peak: f64,
    /// Global maximum velocity magnitude.
    pub velocity_peak: f64,
    /// Sum of squared displacements over all modules/channels.
    pub squared: f64,
    /// Sum of squared velocities.
    pub velocity_squared: f64,
    /// Number of nonfinite module/channel pairs.
    pub nonfinite: usize,
}
impl Diagnostic {
    /// Registered finite-run screen, not a general stability proof.
    #[must_use]
    pub fn valid(self) -> bool {
        self.nonfinite == 0 && self.peak <= 100.0 && self.velocity_peak <= 100.0
    }
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut x = self.0;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    }
}

/// Authoritative fixed-step engine; no rendering clock or graphics dependency.
pub struct Lattice {
    case: Case,
    tick: u64,
    configuration: Vec<[u8; 64]>,
    u: Vec<[f64; 4]>,
    v: Vec<[f64; 4]>,
    next_u: Vec<[f64; 4]>,
    next_v: Vec<[f64; 4]>,
    omega_squared: Vec<f64>,
    damping: Vec<f64>,
    conductance: Vec<f64>,
    internal: Vec<Vec<usize>>,
    external: Vec<Vec<usize>>,
    forces: Vec<f64>,
}

fn number(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("bounded count"))
}
fn internal_neighbors(i: usize) -> Vec<usize> {
    let (row, col) = (i / 8, i % 8);
    let mut result = Vec::new();
    for r in row.saturating_sub(1)..=(row + 1).min(7) {
        for c in col.saturating_sub(1)..=(col + 1).min(7) {
            if (r != row || c != col) && (r == row || c == col) {
                result.push(r * 8 + c);
            }
        }
    }
    result
}

impl Lattice {
    /// Initialize a fully specified fixture, with independent per-neuron arrays.
    ///
    /// # Errors
    /// Rejects an out-of-range source ID or channel.
    ///
    /// # Panics
    /// Panics only if internal fixed-size palette/module invariants are violated.
    pub fn new(case: Case) -> std::io::Result<Self> {
        if case.source >= POPULATION || case.channel >= 4 {
            return Err(std::io::Error::other("invalid fixture source/channel"));
        }
        let mut rng = Random(case.seed);
        let mut configuration: Vec<[u8; 64]> = (0..POPULATION)
            .map(|_| std::array::from_fn(|_| u8::try_from(rng.next() % 5).expect("palette")))
            .collect();
        for tile in &mut configuration {
            match case.configuration {
                Configuration::Homogeneous => tile.fill(1),
                Configuration::Rearranged => {
                    let mut rest: Vec<_> = tile
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &v)| (i != 27).then_some(v))
                        .collect();
                    rest.sort_unstable();
                    let mut next = rest.into_iter();
                    for (i, v) in tile.iter_mut().enumerate() {
                        if i != 27 {
                            *v = next.next().expect("63 modules");
                        }
                    }
                }
                Configuration::Heterogeneous => (),
            }
        }
        let internal: Vec<_> = (0..64).map(internal_neighbors).collect();
        let (mut omega_squared, mut damping, mut conductance) =
            (Vec::new(), Vec::new(), Vec::new());
        for tile in &configuration {
            for (module, &color) in tile.iter().enumerate() {
                let occupied = internal[module].iter().filter(|&&j| tile[j] != 0).count();
                let omega = 0.8
                    + 0.08 * f64::from(color)
                    + 0.04 * number(occupied)
                    + if usize::from(color) == case.channel + 1 {
                        0.12
                    } else {
                        0.0
                    };
                omega_squared.push(omega * omega);
                damping.push(
                    0.08 + 0.01 * number(internal[module].len() - occupied)
                        + if usize::from(color) == case.channel + 1 {
                            0.0
                        } else {
                            0.02
                        },
                );
                conductance.push(if color == 0 {
                    0.5
                } else if usize::from(color) == case.channel + 1 {
                    1.0
                } else {
                    0.75
                });
            }
        }
        let mut forces = vec![0.0; 161];
        if case.drive.omega() > 0.0 {
            for (t, f) in forces.iter_mut().enumerate().skip(1) {
                *f = (case.drive.omega() * number(t) * DT).sin();
            }
            let energy: f64 = forces.iter().map(|x| x * x * DT).sum();
            let scale = energy.sqrt().recip();
            for f in &mut forces {
                *f *= scale;
            }
        }
        let mut value = Self {
            case,
            tick: 0,
            configuration,
            u: vec![[0.0; 4]; POPULATION * 64],
            v: vec![[0.0; 4]; POPULATION * 64],
            next_u: vec![[0.0; 4]; POPULATION * 64],
            next_v: vec![[0.0; 4]; POPULATION * 64],
            omega_squared,
            damping,
            conductance,
            internal,
            external: (0..POPULATION)
                .map(|i| case.topology.neighbors(i))
                .collect(),
            forces,
        };
        value.initialize_transients();
        Ok(value)
    }
    fn initialize_transients(&mut self) {
        self.tick = 0;
        self.u.fill([0.0; 4]);
        self.v.fill([0.0; 4]);
        self.next_u.fill([0.0; 4]);
        self.next_v.fill([0.0; 4]);
        if self.case.drive == Drive::Impulse {
            self.u[self.case.source * 64 + 27][self.case.channel] = 1.0;
        }
    }
    /// Clear every transient field while preserving persistent configuration.
    pub fn reset(&mut self) {
        self.tick = 0;
        self.u.fill([0.0; 4]);
        self.v.fill([0.0; 4]);
        self.next_u.fill([0.0; 4]);
        self.next_v.fill([0.0; 4]);
    }
    /// Current authoritative simulation tick.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }
    /// Fixture identity, independent of the viewer.
    #[must_use]
    pub const fn case(&self) -> Case {
        self.case
    }
    /// Exact externally supplied source force at a registered tick.
    #[must_use]
    pub fn force(&self, tick: usize) -> Option<f64> {
        self.forces.get(tick).copied()
    }
    /// Advance exactly one registered timestep; returns false at tick 160.
    pub fn advance(&mut self) -> bool {
        self.step_order(false)
    }
    fn step_order(&mut self, reverse: bool) -> bool {
        if self.tick >= TICKS {
            return false;
        }
        let channel = self.case.channel;
        for position in 0..self.u.len() {
            let index = if reverse {
                self.u.len() - 1 - position
            } else {
                position
            };
            let value = self.u[index][channel];
            let mut a =
                -self.omega_squared[index] * value - self.damping[index] * self.v[index][channel];
            let module = index % 64;
            let base = index - module;
            let g = self.conductance[index];
            for &neighbor in &self.internal[module] {
                let j = base + neighbor;
                a += (0.6 / 4.0) * g * self.conductance[j] * (self.u[j][channel] - value);
            }
            for &neuron in &self.external[index / 64] {
                let j = neuron * 64 + module;
                a += (0.4 / number(self.case.topology.maximum_degree()))
                    * g
                    * self.conductance[j]
                    * (self.u[j][channel] - value);
            }
            if index == self.case.source * 64 + 27 {
                a += self.forces[usize::try_from(self.tick + 1).expect("bounded tick")];
            }
            self.next_v[index][channel] = self.v[index][channel] + DT * a;
            self.next_u[index][channel] = value + DT * self.next_v[index][channel];
        }
        std::mem::swap(&mut self.u, &mut self.next_u);
        std::mem::swap(&mut self.v, &mut self.next_v);
        self.tick += 1;
        true
    }
    /// Measured per-neuron state, used identically by recorder and live viewer.
    #[must_use]
    pub fn observe(&self) -> Snapshot {
        let neurons = (0..POPULATION)
            .map(|id| {
                let mut n = Neuron {
                    id,
                    position: self.case.topology.coordinates(id),
                    palette: self.configuration[id],
                    ..Neuron::default()
                };
                for module in 0..64 {
                    let i = id * 64 + module;
                    for k in 0..4 {
                        let (u, v) = (self.u[i][k], self.v[i][k]);
                        n.peak[k] = n.peak[k].max(u.abs());
                        n.velocity_peak[k] = n.velocity_peak[k].max(v.abs());
                        n.squared += u * u;
                        n.velocity_squared += v * v;
                        if !u.is_finite() || !v.is_finite() {
                            n.nonfinite += 1;
                        }
                        if module == 27 {
                            n.module27[k] = u;
                            n.velocity27[k] = v;
                        }
                    }
                }
                n
            })
            .collect();
        Snapshot {
            case: self.case,
            tick: self.tick,
            synthetic: false,
            neurons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn update_order_reset_independent_state_and_finite_speed() {
        let case = Case::default();
        let mut a = Lattice::new(case).unwrap();
        let mut b = Lattice::new(case).unwrap();
        a.step_order(false);
        b.step_order(true);
        assert_eq!(a.u, b.u);
        assert_eq!(a.v, b.v);
        let far = case.topology.id([10, 8, 8]).unwrap();
        assert!(a.u[far * 64 + 27][0].abs() < 1e-15);
        let near = case.topology.id([9, 8, 8]).unwrap();
        assert!(a.u[near * 64 + 27][0] > 0.0);
        let before = a.configuration.clone();
        a.configuration[123][9] = (a.configuration[123][9] + 1) % 5;
        assert_eq!(&a.configuration[..123], &before[..123]);
        assert_eq!(&a.configuration[124..], &before[124..]);
        let stored = a.configuration.clone();
        a.reset();
        assert_eq!(a.configuration, stored);
        assert!(
            a.u.iter()
                .flatten()
                .chain(a.v.iter().flatten())
                .all(|x| x.abs() < 1e-15)
        );
        assert_eq!(a.tick, 0);
        assert!(a.observe().diagnostic().valid());
    }
    #[test]
    fn tones_are_equal_energy_and_do_not_mutate_structure() {
        for drive in [Drive::LowTone, Drive::HighTone] {
            let mut m = Lattice::new(Case {
                drive,
                ..Case::default()
            })
            .unwrap();
            let p = m.configuration.clone();
            let e: f64 = m.forces.iter().map(|x| x * x * DT).sum();
            assert!((e - 1.0).abs() < 1e-12);
            m.advance();
            assert_eq!(m.configuration, p);
        }
    }
}
