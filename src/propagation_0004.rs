//! Frozen single-channel medium and full signed observations for Experiment 0004.
use crate::{
    lattice::{Case, Configuration, Drive, Lattice},
    observation::{Neuron, Recording, Snapshot, checksum},
    topology::{POPULATION, Topology},
};
use std::{fs, io, path::Path};

/// Internal modules per neuron.
pub const MODULES: usize = 64;
/// Number of active-channel module values.
pub const FIELDS: usize = POPULATION * MODULES;
/// Fixed source A at (6,8,8).
pub const SOURCE_A: usize = 2182;
/// Fixed source B at (10,8,8).
pub const SOURCE_B: usize = 2186;
/// Registered physical-time checkpoints expressed as dt=0.1 steps.
pub const CHECKPOINTS: [u64; 12] = [0, 8, 16, 32, 64, 128, 160, 256, 384, 512, 768, 1024];
const MODEL: &[u8] = b"qr4-fixed-0002-v1;registration=4a2283a";

/// Independently integrated initial-displacement condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Arm {
    /// Source A only, positive displacement.
    A = 0,
    /// Source B only, positive displacement.
    B = 1,
    /// Both sources positive.
    Plus = 2,
    /// A positive, B negative.
    Minus = 3,
    /// Exactly zero initial and forced state.
    Zero = 4,
    /// Experiment 0003 center-impulse regression.
    Regression = 5,
}
impl Arm {
    /// Stable condition label.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::Plus => "AB+",
            Self::Minus => "AB-",
            Self::Zero => "zero",
            Self::Regression => "regression",
        }
    }
    fn decode(value: u8) -> io::Result<Self> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::Plus),
            3 => Ok(Self::Minus),
            4 => Ok(Self::Zero),
            5 => Ok(Self::Regression),
            _ => Err(io::Error::other("unknown 0004 arm")),
        }
    }
}

/// Full active-channel measured state; palettes are actual independently owned arrays.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Seed for independently generated initial palettes.
    pub seed: u64,
    /// Fixed configuration class.
    pub configuration: Configuration,
    /// Explicit integration timestep.
    pub dt: f64,
    /// Exact integration step, not a wall-clock frame.
    pub step: u64,
    /// Initial displacement condition.
    pub arm: Arm,
    /// Persistent spatial configurations; never changed during this experiment.
    pub palette: Vec<[u8; MODULES]>,
    /// Signed active-channel displacements in neuron/module order.
    pub u: Vec<f64>,
    /// Signed active-channel velocities in neuron/module order.
    pub v: Vec<f64>,
}
fn source_ids(arm: Arm) -> [usize; 2] {
    if arm == Arm::Regression {
        [2184, 2184]
    } else {
        [SOURCE_A, SOURCE_B]
    }
}
fn count(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("bounded module count"))
}
fn internal(module: usize) -> Vec<usize> {
    let (r, c) = (module / 8, module % 8);
    (0..64)
        .filter(|&j| (r.abs_diff(j / 8) + c.abs_diff(j % 8)) == 1)
        .collect()
}
impl Frame {
    /// Convert measured fields to the existing presentation structure only.
    /// Encoding this as an Experiment 0003 file is intentionally not supported.
    #[must_use]
    pub fn observation(&self) -> Snapshot {
        let neurons = self
            .palette
            .iter()
            .enumerate()
            .map(|(id, &palette)| {
                let mut n = Neuron {
                    id,
                    position: Topology::Cubic6.coordinates(id),
                    palette,
                    ..Neuron::default()
                };
                for m in 0..64 {
                    let i = id * 64 + m;
                    let (u, v) = (self.u[i], self.v[i]);
                    n.peak[0] = n.peak[0].max(u.abs());
                    n.velocity_peak[0] = n.velocity_peak[0].max(v.abs());
                    n.squared += u * u;
                    n.velocity_squared += v * v;
                    n.nonfinite += u32::from(!u.is_finite() || !v.is_finite());
                    if m == 27 {
                        n.module27[0] = u;
                        n.velocity27[0] = v;
                    }
                }
                n
            })
            .collect();
        Snapshot {
            case: Case {
                topology: Topology::Cubic6,
                seed: self.seed,
                configuration: self.configuration,
                source: source_ids(self.arm)[0],
                channel: 0,
                drive: Drive::Zero,
            },
            tick: self.step,
            synthetic: false,
            neurons,
        }
    }
    fn validate(&self) -> io::Result<()> {
        if self.palette.len() != POPULATION
            || self.u.len() != FIELDS
            || self.v.len() != FIELDS
            || ![0.1, 0.05].contains(&self.dt)
            || self.step
                > if self.dt.to_bits() == 0.1_f64.to_bits() {
                    1024
                } else {
                    2048
                }
            || ![Configuration::Heterogeneous, Configuration::Homogeneous]
                .contains(&self.configuration)
            || self.palette.iter().flatten().any(|&p| p > 4)
        {
            return Err(io::Error::other("invalid 0004 frame fields"));
        }
        Ok(())
    }
    /// Encode a versioned full signed observation, with frozen-model provenance.
    ///
    /// # Errors
    /// Rejects invalid field dimensions, timestep or palette values.
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = Vec::with_capacity(4_500_000);
        bytes.extend_from_slice(b"QR4FLD01");
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&[0, self.configuration as u8, self.arm as u8, 0]);
        bytes.extend_from_slice(&self.seed.to_le_bytes());
        bytes.extend_from_slice(&self.step.to_le_bytes());
        bytes.extend_from_slice(&self.dt.to_le_bytes());
        for s in source_ids(self.arm) {
            bytes.extend_from_slice(&u32::try_from(s).map_err(io::Error::other)?.to_le_bytes());
        }
        bytes.extend_from_slice(
            &u32::try_from(MODEL.len())
                .map_err(io::Error::other)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(MODEL);
        for p in &self.palette {
            bytes.extend_from_slice(p);
        }
        for a in [&self.u, &self.v] {
            for x in a {
                bytes.extend_from_slice(&x.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&checksum(&bytes).to_le_bytes());
        Ok(bytes)
    }
    /// Decode bounded full-state observations without executing a scientific engine.
    ///
    /// # Errors
    /// Rejects wrong length/schema/model, corruption, invalid sources and fields.
    ///
    /// # Panics
    /// Only if fixed format offsets contradict the prevalidated exact length.
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let offset = 52 + MODEL.len();
        let expected = offset + FIELDS + 16 * FIELDS + 8;
        if bytes.len() != expected {
            return Err(io::Error::other("0004 frame length mismatch"));
        }
        let read_u64 = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().expect("bounded field"));
        if &bytes[..8] != b"QR4FLD01"
            || bytes[8..12] != 1u32.to_le_bytes()
            || bytes[12] != 0
            || bytes[15] != 0
            || &bytes[52..offset] != MODEL
            || bytes[48..52]
                != u32::try_from(MODEL.len())
                    .map_err(io::Error::other)?
                    .to_le_bytes()
            || read_u64(expected - 8) != checksum(&bytes[..expected - 8])
        {
            return Err(io::Error::other("0004 schema/provenance/checksum mismatch"));
        }
        let arm = Arm::decode(bytes[14])?;
        for (i, s) in source_ids(arm).iter().enumerate() {
            if bytes[40 + i * 4..44 + i * 4]
                != u32::try_from(*s).map_err(io::Error::other)?.to_le_bytes()
            {
                return Err(io::Error::other("0004 source mismatch"));
            }
        }
        let palette = bytes[offset..offset + FIELDS]
            .as_chunks::<64>()
            .0
            .to_vec()
            .into_iter()
            .collect();
        let start = offset + FIELDS;
        let u = (0..FIELDS)
            .map(|i| f64::from_bits(read_u64(start + 8 * i)))
            .collect();
        let v = (0..FIELDS)
            .map(|i| f64::from_bits(read_u64(start + 8 * FIELDS + 8 * i)))
            .collect();
        let frame = Self {
            seed: read_u64(16),
            configuration: Configuration::try_from(bytes[13])?,
            dt: f64::from_bits(read_u64(32)),
            step: read_u64(24),
            arm,
            palette,
            u,
            v,
        };
        frame.validate()?;
        Ok(frame)
    }
}

/// Separate fixed medium; no old experiment engine is modified.
pub struct Medium {
    /// Authoritative signed state and persistent palette.
    pub state: Frame,
    next_u: Vec<f64>,
    next_v: Vec<f64>,
    omega_squared: Vec<f64>,
    damping: Vec<f64>,
    conductance: Vec<f64>,
    internal: Vec<Vec<usize>>,
    external: Vec<Vec<usize>>,
}
impl Medium {
    /// Initialize independently integrated, explicitly signed source conditions.
    ///
    /// # Errors
    /// Rejects unsupported timestep/configuration; propagates legacy initialization errors.
    pub fn new(seed: u64, configuration: Configuration, dt: f64, arm: Arm) -> io::Result<Self> {
        let legacy = Lattice::new(Case {
            seed,
            configuration,
            drive: Drive::Zero,
            ..Case::default()
        })?
        .observe();
        let palette: Vec<_> = legacy.neurons.into_iter().map(|n| n.palette).collect();
        let neighbors: Vec<_> = (0..64).map(internal).collect();
        let (mut omega_squared, mut damping, mut conductance) =
            (Vec::new(), Vec::new(), Vec::new());
        for tile in &palette {
            for (m, &color) in tile.iter().enumerate() {
                let occupied = neighbors[m].iter().filter(|&&j| tile[j] != 0).count();
                let omega = 0.8
                    + 0.08 * f64::from(color)
                    + 0.04 * count(occupied)
                    + if color == 1 { 0.12 } else { 0.0 };
                omega_squared.push(omega * omega);
                damping.push(
                    0.08 + 0.01 * count(neighbors[m].len() - occupied)
                        + if color == 1 { 0.0 } else { 0.02 },
                );
                conductance.push(if color == 0 {
                    0.5
                } else if color == 1 {
                    1.0
                } else {
                    0.75
                });
            }
        }
        let mut state = Frame {
            seed,
            configuration,
            dt,
            step: 0,
            arm,
            palette,
            u: vec![0.0; FIELDS],
            v: vec![0.0; FIELDS],
        };
        for (source, amplitude) in match arm {
            Arm::A => vec![(SOURCE_A, 1.0)],
            Arm::B => vec![(SOURCE_B, 1.0)],
            Arm::Plus => vec![(SOURCE_A, 1.0), (SOURCE_B, 1.0)],
            Arm::Minus => vec![(SOURCE_A, 1.0), (SOURCE_B, -1.0)],
            Arm::Zero => vec![],
            Arm::Regression => vec![(2184, 1.0)],
        } {
            state.u[source * 64 + 27] = amplitude;
        }
        state.validate()?;
        Ok(Self {
            state,
            next_u: vec![0.0; FIELDS],
            next_v: vec![0.0; FIELDS],
            omega_squared,
            damping,
            conductance,
            internal: neighbors,
            external: (0..POPULATION)
                .map(|i| Topology::Cubic6.neighbors(i))
                .collect(),
        })
    }
    /// One synchronous semi-implicit Euler step, with all coefficients frozen.
    /// No wall clock or renderer participates.
    pub fn advance(&mut self) {
        self.step_order(false);
    }
    fn step_order(&mut self, reverse: bool) {
        for position in 0..FIELDS {
            let i = if reverse {
                FIELDS - 1 - position
            } else {
                position
            };
            let value = self.state.u[i];
            let mut a = -self.omega_squared[i] * value - self.damping[i] * self.state.v[i];
            let module = i % 64;
            let base = i - module;
            let g = self.conductance[i];
            for &neighbor in &self.internal[module] {
                let j = base + neighbor;
                a += (0.6 / 4.0) * g * self.conductance[j] * (self.state.u[j] - value);
            }
            for &neuron in &self.external[i / 64] {
                let j = neuron * 64 + module;
                a += (0.4 / 6.0) * g * self.conductance[j] * (self.state.u[j] - value);
            }
            self.next_v[i] = self.state.v[i] + self.state.dt * a;
            self.next_u[i] = value + self.state.dt * self.next_v[i];
        }
        std::mem::swap(&mut self.state.u, &mut self.next_u);
        std::mem::swap(&mut self.state.v, &mut self.next_v);
        self.state.step += 1;
    }
}

/// Load one bounded 0004 arm for read-only presentation, retaining its metadata.
///
/// # Errors
/// Rejects corrupt, mixed, unordered or oversized recordings.
///
/// # Panics
/// Only if a validated nonempty file list yields no frame.
pub fn load_replay(path: &Path) -> io::Result<(Recording, Frame)> {
    let mut files: Vec<_> = fs::read_dir(path)?
        .map(|e| e.map(|x| x.path()))
        .collect::<io::Result<_>>()?;
    files.retain(|p| p.extension().is_some_and(|e| e == "q4f"));
    files.sort();
    if files.is_empty() || files.len() > 12 {
        return Err(io::Error::other("0004 replay needs 1..12 frames"));
    }
    let mut frames = Vec::new();
    let mut first: Option<Frame> = None;
    let mut previous = None;
    for file in files {
        if fs::metadata(&file)?.len() > 4_500_000 {
            return Err(io::Error::other("0004 file too large"));
        }
        let f = Frame::decode(&fs::read(file)?)?;
        if previous.is_some_and(|t| t >= f.step) {
            return Err(io::Error::other("0004 unordered frames"));
        }
        if let Some(a) = &first {
            if a.seed != f.seed
                || a.configuration != f.configuration
                || a.arm != f.arm
                || a.dt.to_bits() != f.dt.to_bits()
                || a.palette != f.palette
            {
                return Err(io::Error::other("0004 mixed recording provenance"));
            }
        } else {
            first = Some(f.clone());
        }
        previous = Some(f.step);
        frames.push(f.observation());
    }
    Ok((
        Recording { frames },
        first.expect("nonempty validated frames"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_dual_sources_obey_signed_superposition() {
        let mut models: Vec<_> = [Arm::A, Arm::B, Arm::Plus, Arm::Minus]
            .into_iter()
            .map(|arm| Medium::new(7, Configuration::Homogeneous, 0.1, arm).unwrap())
            .collect();
        let palette = models[0].state.palette.clone();
        for _ in 0..12 {
            for name in 0..2 {
                let fields: Vec<_> = models
                    .iter()
                    .map(|m| if name == 0 { &m.state.u } else { &m.state.v })
                    .collect();
                for (((a, b), plus), minus) in fields[0]
                    .iter()
                    .zip(fields[1])
                    .zip(fields[2])
                    .zip(fields[3])
                {
                    assert!((plus - (a + b)).abs() < 1e-13);
                    assert!((minus - (a - b)).abs() < 1e-13);
                }
            }
            for m in &mut models {
                assert_eq!(m.state.palette, palette);
                m.advance();
            }
        }
        assert!(models[2].state.u[2184 * 64 + 27].abs() > 1e-6);
        assert!(models[3].state.u[2184 * 64 + 27].abs() < 1e-15);
    }
    #[test]
    fn regression_order_roundtrip_and_independent_palette() {
        let mut a = Medium::new(7, Configuration::Heterogeneous, 0.1, Arm::Regression).unwrap();
        let mut b = Medium::new(7, Configuration::Heterogeneous, 0.1, Arm::Regression).unwrap();
        let mut old = Lattice::new(Case::default()).unwrap();
        for _ in 0..9 {
            let mut observed = a.state.observation();
            observed.case = old.case();
            assert_eq!(observed, old.observe());
            a.advance();
            b.step_order(true);
            old.advance();
            assert_eq!(a.state, b.state);
        }
        let mut bytes = a.state.encode().unwrap();
        assert_eq!(Frame::decode(&bytes).unwrap(), a.state);
        bytes[300] ^= 1;
        assert!(Frame::decode(&bytes).is_err());
        let p = a.state.palette.clone();
        a.state.palette[0][0] = (p[0][0] + 1) % 5;
        assert_eq!(&a.state.palette[1..], &p[1..]);
    }
}
