//! Experiment 0005: signed morphic oscillator, split integrator and energy audit.
use crate::{
    lattice::{Case, Configuration, Drive, Lattice},
    observation::{Neuron, Recording, Snapshot, checksum},
    topology::{POPULATION, Topology},
};
use std::{fs, io, path::Path};

/// Active independent channel field size.
pub const FIELDS: usize = POPULATION * 64;
/// Registered probes, including sources, midpoint and boundaries.
pub const PROBES: [usize; 11] = [
    0, 2180, 2182, 2183, 2184, 2185, 2186, 2188, 2191, 2440, 4095,
];

#[derive(Clone)]
struct Edge {
    i: usize,
    j: usize,
    k: f64,
    external: bool,
}

/// Full signed schema-1 checkpoint; separate from all historical formats.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Integration tick.
    pub tick: u64,
    /// Physical step size.
    pub dt: f64,
    /// Initialization seed.
    pub seed: u64,
    /// Initial morphology class (actual palette remains authoritative).
    pub configuration: Configuration,
    /// Complete registered condition identity.
    pub label: String,
    /// Independently owned persistent spatial palettes.
    pub palette: Vec<[u8; 64]>,
    /// Signed displacements, neuron then module.
    pub u: Vec<f64>,
    /// Signed velocities in identical indexing.
    pub v: Vec<f64>,
}
impl Frame {
    fn valid(&self) -> bool {
        self.palette.len() == POPULATION
            && self.u.len() == FIELDS
            && self.v.len() == FIELDS
            && [0.1, 0.05, 0.025].contains(&self.dt)
            && self.tick <= 12288
            && !self.label.is_empty()
            && self.label.len() <= 200
            && self.palette.iter().flatten().all(|&p| p <= 4)
            && self
                .u
                .iter()
                .chain(&self.v)
                .all(|x| x.is_finite() && x.abs() <= 100.0)
    }
    /// Encode exact active-channel state with a corruption checksum.
    ///
    /// # Errors
    /// Rejects invalid fields, metadata or numerical-screen breaches.
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        if !self.valid() {
            return Err(io::Error::other("invalid QR5 state"));
        }
        let mut b = b"QR5FLD01".to_vec();
        for x in [
            1,
            self.tick,
            self.dt.to_bits(),
            self.seed,
            self.configuration as u64,
            u64::try_from(self.label.len()).map_err(io::Error::other)?,
        ] {
            b.extend_from_slice(&x.to_le_bytes());
        }
        b.extend_from_slice(self.label.as_bytes());
        for p in &self.palette {
            b.extend_from_slice(p);
        }
        for a in [&self.u, &self.v] {
            for x in a {
                b.extend_from_slice(&x.to_le_bytes());
            }
        }
        b.extend_from_slice(&checksum(&b).to_le_bytes());
        Ok(b)
    }
    /// Decode bounded fields, rejecting corruption and nonfinite values.
    ///
    /// # Errors
    /// Rejects unsupported schema, dimensions, labels or checksum.
    pub fn decode(b: &[u8]) -> io::Result<Self> {
        let bad = || io::Error::other("QR5 schema/length/checksum/field mismatch");
        if b.len() < 56 || &b[..8] != b"QR5FLD01" {
            return Err(bad());
        }
        let read = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap_or([0; 8]));
        let len = usize::try_from(read(48)).map_err(io::Error::other)?;
        if read(8) != 1
            || len > 200
            || b.len() != 56 + len + FIELDS * 17 + 8
            || checksum(&b[..b.len() - 8]) != read(b.len() - 8)
        {
            return Err(bad());
        }
        let offset = 56 + len;
        let palette = b[offset..offset + FIELDS].as_chunks::<64>().0.to_vec();
        let start = offset + FIELDS;
        let frame = Self {
            tick: read(16),
            dt: f64::from_bits(read(24)),
            seed: read(32),
            configuration: Configuration::try_from(
                u8::try_from(read(40)).map_err(io::Error::other)?,
            )?,
            label: String::from_utf8(b[56..offset].to_vec()).map_err(io::Error::other)?,
            palette,
            u: (0..FIELDS)
                .map(|i| f64::from_bits(read(start + 8 * i)))
                .collect(),
            v: (0..FIELDS)
                .map(|i| f64::from_bits(read(start + 8 * FIELDS + 8 * i)))
                .collect(),
        };
        if !frame.valid() {
            return Err(bad());
        }
        Ok(frame)
    }
    /// Pure presentation conversion; no simulation is started.
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
                    let (u, v) = (self.u[id * 64 + m], self.v[id * 64 + m]);
                    n.peak[0] = n.peak[0].max(u.abs());
                    n.velocity_peak[0] = n.velocity_peak[0].max(v.abs());
                    n.squared += u * u;
                    n.velocity_squared += v * v;
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
                seed: self.seed,
                configuration: self.configuration,
                source: 2182,
                drive: Drive::Zero,
                ..Case::default()
            },
            tick: self.tick,
            synthetic: false,
            neurons,
        }
    }
}

/// Energy components and per-neuron partition of undirected-edge potentials.
#[derive(Default)]
pub struct Energy {
    /// Unit-mass kinetic energy.
    pub kinetic: f64,
    /// Morphology-dependent onsite potential.
    pub onsite: f64,
    /// Once-per-undirected-edge potential.
    pub coupling: f64,
    /// Per-neuron energy, including half of each incident edge potential.
    pub local: Vec<f64>,
    /// Net instantaneous power into each neuron over external edges.
    pub flux: Vec<f64>,
}
impl Energy {
    /// Total stored mechanical energy.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.kinetic + self.onsite + self.coupling
    }
}

/// Dedicated deterministic oscillator; historical engines are never modified.
pub struct Medium {
    /// Authoritative signed and persistent state.
    pub state: Frame,
    omega2: Vec<f64>,
    gamma: Vec<f64>,
    sponge: Vec<f64>,
    edges: Vec<Edge>,
    force: Vec<f64>,
    history: Vec<f64>,
    damping_factor: Vec<f64>,
    /// Cumulative exact intrinsic damping loss.
    pub loss: f64,
    /// Cumulative exact sponge damping loss.
    pub boundary_loss: f64,
    /// Applied constant forcing work.
    pub work: f64,
    /// Explicit work of changing morphology at fixed transient state.
    pub switch_work: f64,
}
fn number(n: usize) -> f64 {
    f64::from(u32::try_from(n).expect("bounded dimension"))
}
fn internal(m: usize) -> Vec<usize> {
    (0..64)
        .filter(|&j| (m / 8).abs_diff(j / 8) + (m % 8).abs_diff(j % 8) == 1)
        .collect()
}
impl Medium {
    /// Initialize exact palettes using the existing unmodified seeded generator.
    ///
    /// # Errors
    /// Propagates unsupported initialization errors.
    pub fn new(
        seed: u64,
        configuration: Configuration,
        dt: f64,
        arm: &str,
        label: &str,
    ) -> io::Result<Self> {
        let palette = Lattice::new(Case {
            seed,
            configuration,
            drive: Drive::Zero,
            ..Case::default()
        })?
        .observe()
        .neurons
        .into_iter()
        .map(|n| n.palette)
        .collect();
        let mut state = Frame {
            tick: 0,
            dt,
            seed,
            configuration,
            label: label.into(),
            palette,
            u: vec![0.; FIELDS],
            v: vec![0.; FIELDS],
        };
        match arm {
            "A" => state.u[2182 * 64 + 27] = 1.,
            "B" => state.u[2186 * 64 + 27] = 1.,
            "AB+" | "AB-" => {
                state.u[2182 * 64 + 27] = 1.;
                state.u[2186 * 64 + 27] = if arm == "AB+" { 1. } else { -1. };
            }
            "zero" | "force" => (),
            _ => return Err(io::Error::other("unknown QR5 arm")),
        }
        if !state.valid() {
            return Err(io::Error::other("invalid QR5 initialization"));
        }
        let mut s = Self {
            state,
            omega2: vec![0.; FIELDS],
            gamma: vec![0.; FIELDS],
            sponge: vec![0.; FIELDS],
            edges: Vec::new(),
            force: vec![0.; FIELDS],
            history: vec![0.; FIELDS],
            damping_factor: vec![1.; FIELDS],
            loss: 0.,
            boundary_loss: 0.,
            work: 0.,
            switch_work: 0.,
        };
        s.rebuild();
        Ok(s)
    }
    fn rebuild(&mut self) {
        let mut g = vec![0.; FIELDS];
        for (id, p) in self.state.palette.iter().enumerate() {
            for m in 0..64 {
                let links = internal(m);
                let occupied = links.iter().filter(|&&j| p[j] != 0).count();
                let w = 0.8
                    + 0.08 * f64::from(p[m])
                    + 0.04 * number(occupied)
                    + if p[m] == 1 { 0.12 } else { 0. };
                let i = id * 64 + m;
                self.omega2[i] = w * w;
                self.gamma[i] = 0.08
                    + 0.01 * number(links.len() - occupied)
                    + if p[m] == 1 { 0. } else { 0.02 };
                g[i] = match p[m] {
                    0 => 0.5,
                    1 => 1.,
                    _ => 0.75,
                };
            }
        }
        self.edges.clear();
        for id in 0..POPULATION {
            for m in 0..64 {
                let i = id * 64 + m;
                for n in internal(m) {
                    if m < n {
                        let j = id * 64 + n;
                        self.edges.push(Edge {
                            i,
                            j,
                            k: 0.15 * g[i] * g[j],
                            external: false,
                        });
                    }
                }
                for n in Topology::Cubic6.neighbors(id) {
                    if id < n {
                        let j = n * 64 + m;
                        self.edges.push(Edge {
                            i,
                            j,
                            k: (0.4 / 6.) * g[i] * g[j],
                            external: true,
                        });
                    }
                }
            }
        }
        self.refresh_damping();
        self.acceleration();
    }
    fn refresh_damping(&mut self) {
        for i in 0..FIELDS {
            self.damping_factor[i] =
                (-(self.gamma[i] + self.sponge[i]) * self.state.dt * 0.5).exp();
        }
    }
    /// Apply labeled damping controls; force coefficients remain symmetric.
    pub fn damping_control(&mut self, intrinsic: bool, sponge: bool) {
        if !intrinsic {
            self.gamma.fill(0.);
        }
        if sponge {
            for id in 0..POPULATION {
                let d = Topology::Cubic6
                    .coordinates(id)
                    .into_iter()
                    .map(|x| x.min(15 - x))
                    .min()
                    .unwrap_or(0);
                let value = if d < 3 {
                    0.5 * ((3. - number(d)) / 3.).powi(2)
                } else {
                    0.
                };
                self.sponge[id * 64..id * 64 + 64].fill(value);
            }
        }
        self.refresh_damping();
    }
    fn acceleration(&mut self) {
        for i in 0..FIELDS {
            self.force[i] = -self.omega2[i] * self.state.u[i];
        }
        for e in &self.edges {
            let f = e.k * (self.state.u[e.j] - self.state.u[e.i]);
            self.force[e.i] += f;
            self.force[e.j] -= f;
        }
    }
    fn damp(&mut self) {
        for i in 0..FIELDS {
            let gamma = self.gamma[i] + self.sponge[i];
            if gamma > 0. {
                let before = self.state.v[i];
                self.state.v[i] *= self.damping_factor[i];
                let loss = 0.5 * (before * before - self.state.v[i] * self.state.v[i]);
                self.loss += loss * self.gamma[i] / gamma;
                self.boundary_loss += loss * self.sponge[i] / gamma;
            }
        }
    }
    /// One synchronous split step, retaining exact substep damping losses.
    ///
    /// # Errors
    /// Stops on nonfinite or excessive fields, without clipping.
    pub fn advance(&mut self, forcing: f64, history: bool) -> io::Result<()> {
        if history {
            for i in 0..FIELDS {
                self.history[i] += 0.5 * self.state.dt * self.state.v[i].powi(2);
            }
        }
        self.damp();
        let dt = self.state.dt;
        let source = 2182 * 64 + 27;
        let old = self.state.u[source];
        for i in 0..FIELDS {
            self.state.v[i] += 0.5 * dt * self.force[i];
        }
        self.state.v[source] += 0.5 * dt * forcing;
        for i in 0..FIELDS {
            self.state.u[i] += dt * self.state.v[i];
        }
        self.acceleration();
        for i in 0..FIELDS {
            self.state.v[i] += 0.5 * dt * self.force[i];
        }
        self.state.v[source] += 0.5 * dt * forcing;
        self.work += forcing * (self.state.u[source] - old);
        self.damp();
        self.state.tick += 1;
        if history {
            for i in 0..FIELDS {
                self.history[i] += 0.5 * dt * self.state.v[i].powi(2);
            }
        }
        if !self
            .state
            .u
            .iter()
            .chain(&self.state.v)
            .all(|x| x.is_finite() && x.abs() <= 100.)
        {
            return Err(io::Error::other("QR5 numerical screen failed"));
        }
        Ok(())
    }
    /// Actual energy and conservative continuous-time external flux partition.
    #[must_use]
    pub fn energy(&self) -> Energy {
        let mut e = Energy {
            local: vec![0.; POPULATION],
            flux: vec![0.; POPULATION],
            ..Energy::default()
        };
        for i in 0..FIELDS {
            let k = 0.5 * self.state.v[i].powi(2);
            let p = 0.5 * self.omega2[i] * self.state.u[i].powi(2);
            e.kinetic += k;
            e.onsite += p;
            e.local[i / 64] += k + p;
        }
        for edge in &self.edges {
            let delta = self.state.u[edge.j] - self.state.u[edge.i];
            let p = 0.5 * edge.k * delta * delta;
            e.coupling += p;
            e.local[edge.i / 64] += 0.5 * p;
            e.local[edge.j / 64] += 0.5 * p;
            if edge.external {
                let power = 0.5 * edge.k * delta * (self.state.v[edge.i] + self.state.v[edge.j]);
                e.flux[edge.i / 64] += power;
                e.flux[edge.j / 64] -= power;
            }
        }
        e
    }
    /// Local physical-time velocity-history integrals, exported for independent rule checks.
    #[must_use]
    pub fn history(&self) -> &[f64] {
        &self.history
    }
    /// Single local-history or generic-time update, explicitly accounting switch work.
    /// Returns number of changed neurons. Disabled mode uses identical history code.
    pub fn adapt(&mut self, mode: &str) -> usize {
        let before = self.energy().total();
        let mut changed = 0;
        for id in 0..POPULATION {
            let mut best = 0;
            for m in 1..64 {
                if self.history[id * 64 + m] > self.history[id * 64 + best] {
                    best = m;
                }
            }
            if mode == "time" || (mode == "local" && self.history[id * 64 + best] >= 1e-8) {
                let m = if mode == "time" { 0 } else { best };
                self.state.palette[id][m] = (self.state.palette[id][m] + 1) % 5;
                changed += 1;
            }
        }
        self.rebuild();
        self.switch_work += self.energy().total() - before;
        changed
    }
    /// Retain actual morphology while erasing every transient/history/audit accumulator.
    pub fn reexpose(&mut self, label: &str) {
        self.state.u.fill(0.);
        self.state.v.fill(0.);
        self.history.fill(0.);
        self.state.u[2182 * 64 + 27] = 1.;
        self.state.tick = 0;
        self.state.label = label.into();
        self.loss = 0.;
        self.boundary_loss = 0.;
        self.work = 0.;
        self.switch_work = 0.;
        self.acceleration();
    }
}

/// Load at most seven measured QR5 checkpoints for replay only.
///
/// # Errors
/// Rejects mixed identity, corruption and non-increasing ticks.
pub fn load_replay(path: &Path) -> io::Result<(Recording, Frame)> {
    let mut paths = fs::read_dir(path)?
        .map(|e| e.map(|x| x.path()))
        .collect::<io::Result<Vec<_>>>()?;
    paths.retain(|p| p.extension().is_some_and(|x| x == "q5f"));
    paths.sort();
    if paths.is_empty() || paths.len() > 7 {
        return Err(io::Error::other("QR5 replay requires1..7 checkpoints"));
    }
    let mut first: Option<Frame> = None;
    let mut frames = Vec::new();
    let mut previous = None;
    for p in paths {
        if fs::metadata(&p)?.len() > 4_500_000 {
            return Err(io::Error::other("QR5 frame too large"));
        }
        let f = Frame::decode(&fs::read(p)?)?;
        if previous.is_some_and(|t| t >= f.tick) {
            return Err(io::Error::other("QR5 unordered ticks"));
        }
        if let Some(a) = &first {
            if a.label != f.label
                || a.seed != f.seed
                || a.configuration != f.configuration
                || a.dt.to_bits() != f.dt.to_bits()
            {
                return Err(io::Error::other("QR5 mixed identity"));
            }
        } else {
            first = Some(f.clone());
        }
        previous = Some(f.tick);
        frames.push(f.observation());
    }
    Ok((
        Recording { frames },
        first.ok_or_else(|| io::Error::other("empty QR5 replay"))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn medium() -> Medium {
        Medium::new(7, Configuration::Heterogeneous, 0.05, "A", "test").unwrap()
    }
    #[test]
    fn energy_gradient_graph_flux_and_synchronous_causality() {
        let mut m = medium();
        assert_eq!(m.edges.len(), 1_196_032);
        assert!(
            m.edges
                .iter()
                .all(|e| e.i < e.j && e.k > 0. && (!e.external || e.i % 64 == e.j % 64))
        );
        let index = 2182 * 64 + 27;
        let force = m.force[index];
        let h = 1e-6;
        m.state.u[index] += h;
        let plus = m.energy().total();
        m.state.u[index] -= 2. * h;
        let minus = m.energy().total();
        m.state.u[index] += h;
        assert!(((plus - minus) / (2. * h) + force).abs() < 1e-8);
        m.advance(0., false).unwrap();
        assert_eq!(m.state.u[0], 0.);
        let e = m.energy();
        assert!((e.local.iter().sum::<f64>() - e.total()).abs() < 1e-10);
        assert!(e.flux.iter().sum::<f64>().abs() < 1e-12);
        let mut reference = medium();
        reference.advance(0., false).unwrap();
        assert_eq!(m.state.u, reference.state.u);
    }
    #[test]
    fn signed_superposition_energy_controls_and_local_rule() {
        let mut a = medium();
        let mut b = Medium::new(7, Configuration::Heterogeneous, 0.05, "B", "test").unwrap();
        let mut plus = Medium::new(7, Configuration::Heterogeneous, 0.05, "AB+", "test").unwrap();
        for _ in 0..8 {
            a.advance(0., true).unwrap();
            b.advance(0., false).unwrap();
            plus.advance(0., false).unwrap();
        }
        assert!(
            a.state
                .u
                .iter()
                .zip(&b.state.u)
                .zip(&plus.state.u)
                .all(|((a, b), p)| (p - a - b).abs() < 1e-14)
        );
        let prior = a.state.palette.clone();
        assert_eq!(a.adapt("disabled"), 0);
        assert_eq!(a.state.palette, prior);
        let edits = a.adapt("local");
        assert!(edits > 0);
        assert!(
            a.state.palette.iter().zip(&prior).all(|(a, b)| a
                .iter()
                .zip(b)
                .filter(|(a, b)| a != b)
                .count()
                <= 1)
        );
        let mut undamped = medium();
        undamped.damping_control(false, false);
        let initial = undamped.energy().total();
        for _ in 0..16 {
            undamped.advance(0., false).unwrap();
        }
        assert_eq!(undamped.loss, 0.);
        assert!((undamped.energy().total() - initial).abs() / initial < 0.005);
        let mut forced = Medium::new(7, Configuration::Homogeneous, 0.05, "zero", "force").unwrap();
        forced.advance(0.1, false).unwrap();
        assert!(forced.work > 0. && forced.loss > 0. && forced.energy().total() > 0.);
    }
    #[test]
    fn replay_zero_adaptation_work_and_failure() {
        let mut m = Medium::new(7, Configuration::Heterogeneous, 0.05, "zero", "test").unwrap();
        m.advance(0., true).unwrap();
        assert_eq!(m.energy().total(), 0.);
        assert_eq!(m.adapt("local"), 0);
        let b = m.state.encode().unwrap();
        assert_eq!(Frame::decode(&b).unwrap(), m.state);
        let mut bad = b;
        bad[120] ^= 1;
        assert!(Frame::decode(&bad).is_err());
        m.reexpose("test");
        let before = m.energy().total();
        assert_eq!(m.adapt("time"), 4096);
        assert!((m.energy().total() - before - m.switch_work).abs() < 1e-12);
        m.reexpose("test");
        assert_eq!(m.state.v.iter().sum::<f64>(), 0.);
        assert_eq!(m.switch_work, 0.);
        m.state.u[0] = f64::NAN;
        assert!(m.advance(0., false).is_err());
    }
}
