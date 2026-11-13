//! Experiment 0006: signed morphic oscillator, split integrator and energy audit.
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
    /// Registered injection module (9 or 27).
    pub module: usize,
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
            && [0.05, 0.025, 0.0125].contains(&self.dt)
            && [9, 27].contains(&self.module)
            && self.tick <= 24576
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
            return Err(io::Error::other("invalid QR6 state"));
        }
        let mut b = b"QR6FLD01".to_vec();
        for x in [
            1,
            self.tick,
            self.dt.to_bits(),
            self.seed,
            self.configuration as u64,
            u64::try_from(self.label.len()).map_err(io::Error::other)?,
            u64::try_from(self.module).map_err(io::Error::other)?,
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
        let bad = || io::Error::other("QR6 schema/length/checksum/field mismatch");
        if b.len() < 64 || &b[..8] != b"QR6FLD01" {
            return Err(bad());
        }
        let read = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap_or([0; 8]));
        let len = usize::try_from(read(48)).map_err(io::Error::other)?;
        if read(8) != 1
            || len > 200
            || b.len() != 64 + len + FIELDS * 17 + 8
            || checksum(&b[..b.len() - 8]) != read(b.len() - 8)
        {
            return Err(bad());
        }
        let offset = 64 + len;
        let palette = b[offset..offset + FIELDS].as_chunks::<64>().0.to_vec();
        let start = offset + FIELDS;
        let frame = Self {
            module: usize::try_from(read(56)).map_err(io::Error::other)?,
            tick: read(16),
            dt: f64::from_bits(read(24)),
            seed: read(32),
            configuration: Configuration::try_from(
                u8::try_from(read(40)).map_err(io::Error::other)?,
            )?,
            label: String::from_utf8(b[64..offset].to_vec()).map_err(io::Error::other)?,
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
    shape_history: Vec<f64>,
    internal_graph: Vec<Vec<usize>>,
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
        module: usize,
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
            module,
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
            "A" => state.u[2182 * 64 + module] = 1.,
            "B" => state.u[2186 * 64 + module] = 1.,
            "AB+" | "AB-" => {
                state.u[2182 * 64 + module] = 1.;
                state.u[2186 * 64 + module] = if arm == "AB+" { 1. } else { -1. };
            }
            "zero" | "force" => (),
            _ => return Err(io::Error::other("unknown QR6 arm")),
        }
        if !state.valid() {
            return Err(io::Error::other("invalid QR6 initialization"));
        }
        let mut s = Self {
            state,
            omega2: vec![0.; FIELDS],
            gamma: vec![0.; FIELDS],
            sponge: vec![0.; FIELDS],
            edges: Vec::new(),
            force: vec![0.; FIELDS],
            history: vec![0.; FIELDS],
            shape_history: vec![0.; FIELDS],
            internal_graph: (0..64).map(internal).collect(),
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
                self.accumulate(i, 0.5 * self.state.dt);
            }
        }
        self.damp();
        let dt = self.state.dt;
        let source = 2182 * 64 + self.state.module;
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
                self.accumulate(i, 0.5 * dt);
            }
        }
        if !self
            .state
            .u
            .iter()
            .chain(&self.state.v)
            .all(|x| x.is_finite() && x.abs() <= 100.)
        {
            return Err(io::Error::other("QR6 numerical screen failed"));
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
    fn accumulate(&mut self, i: usize, weight: f64) {
        let module = i % 64;
        let base = i - module;
        let links = &self.internal_graph[module];
        let strain = links
            .iter()
            .map(|&q| (self.state.u[i] - self.state.u[base + q]).powi(2))
            .sum::<f64>()
            / number(links.len());
        let velocity = self.state.v[i].powi(2);
        self.history[i] += weight * velocity;
        self.shape_history[i] += weight * (velocity + 0.25 * strain);
    }
    /// Full local strain/velocity history H at the current observation window.
    #[must_use]
    pub fn shape_history(&self) -> &[f64] {
        &self.shape_history
    }
    /// Compute one-hop eligibility from simultaneous own-neuron histories.
    #[must_use]
    pub fn scores(&self) -> Vec<f64> {
        local_scores(&self.shape_history)
    }
    /// Apply a registered rule synchronously and record its explicit energy work.
    ///
    /// # Errors
    /// Rejects unsupported rules or invalid field dimensions.
    pub fn adapt_rule(&mut self, rule: u8) -> io::Result<usize> {
        let palette = morph_palette(&self.state.palette, &self.history, &self.scores(), rule)?;
        self.apply_palette(palette)
    }
    /// Replace an entire immutable palette variant, rebuild coefficients, account work.
    ///
    /// # Errors
    /// Rejects illegal colors or dimensions before changing state.
    pub fn apply_palette(&mut self, palette: Vec<[u8; 64]>) -> io::Result<usize> {
        if palette.len() != POPULATION || palette.iter().flatten().any(|&p| p > 4) {
            return Err(io::Error::other("invalid QR6 palette"));
        }
        let before = self.energy().total();
        let count = palette
            .iter()
            .zip(&self.state.palette)
            .filter(|(a, b)| a != b)
            .count();
        self.state.palette = palette;
        self.rebuild();
        self.switch_work += self.energy().total() - before;
        Ok(count)
    }
    /// Reset all transient fields and audit/history arrays; set the registered impulse.
    ///
    /// # Errors
    /// Rejects nonfinite amplitude, illegal module or unsupported arm.
    pub fn reset_impulse(&mut self, module: usize, arm: &str, amplitude: f64) -> io::Result<()> {
        if ![9, 27].contains(&module) || !amplitude.is_finite() {
            return Err(io::Error::other("invalid impulse"));
        }
        self.state.module = module;
        self.state.u.fill(0.);
        self.state.v.fill(0.);
        match arm {
            "A" => self.state.u[2182 * 64 + module] = amplitude,
            "AB+" => {
                self.state.u[2182 * 64 + module] = amplitude;
                self.state.u[2186 * 64 + module] = amplitude;
            }
            "zero" => (),
            _ => return Err(io::Error::other("unsupported QR6 impulse")),
        }
        self.history.fill(0.);
        self.shape_history.fill(0.);
        self.state.tick = 0;
        self.loss = 0.;
        self.boundary_loss = 0.;
        self.work = 0.;
        self.switch_work = 0.;
        self.acceleration();
        Ok(())
    }
    /// Scale a unit impulse to an explicitly supplied mechanical energy.
    ///
    /// # Errors
    /// Rejects nonpositive or nonfinite reference/unit energy.
    pub fn match_energy(&mut self, target: f64) -> io::Result<f64> {
        let unit = self.energy().total();
        if !unit.is_finite() || unit <= 0. || !target.is_finite() || target <= 0. {
            return Err(io::Error::other("nonpositive/nonfinite matching energy"));
        }
        let amplitude = (target / unit).sqrt();
        for value in &mut self.state.u {
            *value *= amplitude;
        }
        self.acceleration();
        Ok(amplitude)
    }
}

/// Load at most twelve measured QR6 checkpoints for replay only.
///
/// # Errors
/// Rejects mixed identity, corruption and non-increasing ticks.
pub fn load_replay(path: &Path) -> io::Result<(Recording, Frame)> {
    let mut paths = fs::read_dir(path)?
        .map(|e| e.map(|x| x.path()))
        .collect::<io::Result<Vec<_>>>()?;
    paths.retain(|p| p.extension().is_some_and(|x| x == "q5f"));
    paths.sort();
    if paths.is_empty() || paths.len() > 12 {
        return Err(io::Error::other("QR6 replay requires1..12 checkpoints"));
    }
    let mut first: Option<Frame> = None;
    let mut frames = Vec::new();
    let mut previous = None;
    for p in paths {
        if fs::metadata(&p)?.len() > 4_500_000 {
            return Err(io::Error::other("QR6 frame too large"));
        }
        let f = Frame::decode(&fs::read(p)?)?;
        if previous.is_some_and(|t| t >= f.tick) {
            return Err(io::Error::other("QR6 unordered ticks"));
        }
        if let Some(a) = &first {
            if a.label != f.label
                || a.seed != f.seed
                || a.configuration != f.configuration
                || a.module != f.module
                || a.dt.to_bits() != f.dt.to_bits()
            {
                return Err(io::Error::other("QR6 mixed identity"));
            }
        } else {
            first = Some(f.clone());
        }
        previous = Some(f.tick);
        frames.push(f.observation());
    }
    Ok((
        Recording { frames },
        first.ok_or_else(|| io::Error::other("empty QR6 replay"))?,
    ))
}

/// Within-neuron one-hop scores; no external information or in-place smoothing.
#[must_use]
pub fn local_scores(history: &[f64]) -> Vec<f64> {
    (0..history.len())
        .map(|i| {
            let module = i % 64;
            let links = internal(module);
            history[i]
                + 0.5 * links.iter().map(|&q| history[i - module + q]).sum::<f64>()
                    / number(links.len())
        })
        .collect()
}
/// Frozen registered R0/R1/R2/R3 palette transformation from old local fields.
/// Rule4 is disabled R2. No target or mandatory minimum edit quota.
///
/// # Errors
/// Rejects invalid rules, colors, dimensions or nonfinite/negative histories.
pub fn morph_palette(
    before: &[[u8; 64]],
    velocity: &[f64],
    scores: &[f64],
    rule: u8,
) -> io::Result<Vec<[u8; 64]>> {
    if rule > 4
        || before.len() != POPULATION
        || velocity.len() != FIELDS
        || scores.len() != FIELDS
        || before.iter().flatten().any(|&p| p > 4)
        || velocity
            .iter()
            .chain(scores)
            .any(|&x| !x.is_finite() || x < 0.)
    {
        return Err(io::Error::other("invalid local morphology input"));
    }
    let mut after = before.to_vec();
    for id in 0..POPULATION {
        let v = &velocity[id * 64..id * 64 + 64];
        let s = &scores[id * 64..id * 64 + 64];
        let max = s.iter().copied().fold(0., f64::max);
        if rule == 1 {
            let mut best = 0;
            for m in 1..64 {
                if v[m] > v[best] {
                    best = m;
                }
            }
            if v[best] >= 1e-8 {
                after[id][best] = (before[id][best] + 1) % 5;
            }
        } else if rule == 2 {
            let mut eligible: Vec<_> = (0..64)
                .filter(|&m| s[m] >= 1e-8 && s[m] >= 0.05 * max)
                .collect();
            eligible.sort_by(|&a, &b| s[b].total_cmp(&s[a]).then(a.cmp(&b)));
            for &m in eligible.iter().take(8) {
                after[id][m] = (before[id][m] + 1) % 5;
            }
        } else if rule == 3 {
            let mut edges = Vec::new();
            for m in 0..64 {
                for n in internal(m) {
                    if m < n
                        && before[id][m] != before[id][n]
                        && s[m] >= 1e-8
                        && s[n] >= 1e-8
                        && (s[m] >= 0.05 * max || s[n] >= 0.05 * max)
                    {
                        edges.push((m, n));
                    }
                }
            }
            edges.sort_by(|&(a, b), &(c, d)| {
                (s[c] + s[d])
                    .total_cmp(&(s[a] + s[b]))
                    .then((a, b).cmp(&(c, d)))
            });
            let mut used = [false; 64];
            let mut count = 0;
            for (m, n) in edges {
                if count < 4 && !used[m] && !used[n] {
                    after[id][m] = before[id][n];
                    after[id][n] = before[id][m];
                    used[m] = true;
                    used[n] = true;
                    count += 1;
                }
            }
        }
    }
    Ok(after)
}
fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}
/// Construct immutable causal masks or the registered seeded position control.
///
/// # Errors
/// Rejects unknown variants, illegal dimensions and infeasible count matching.
///
/// # Panics
/// Only if a prevalidated module index cannot fit in a u64.
pub fn variant(
    before: &[[u8; 64]],
    after: &[[u8; 64]],
    kind: &str,
    rule: u8,
    seed: u64,
) -> io::Result<Vec<[u8; 64]>> {
    if before.len() != POPULATION || after.len() != POPULATION {
        return Err(io::Error::other("variant dimensions"));
    }
    let mut palette = before.to_vec();
    match kind {
        "VIRGIN" => (),
        "ALL" => palette = after.to_vec(),
        "SOURCE_ONLY" => palette[2182] = after[2182],
        "OFF_SOURCE_ONLY" => {
            palette = after.to_vec();
            palette[2182] = before[2182];
        }
        "SHUFFLE_POSITION" => {
            for id in 0..POPULATION {
                let count = before[id]
                    .iter()
                    .zip(after[id])
                    .filter(|(a, b)| **a != *b)
                    .count();
                let mut order: Vec<_> = (0..64).collect();
                order.sort_by_key(|&m| {
                    (
                        splitmix(
                            seed ^ u64::try_from(id * 64 + m).expect("bounded index") ^ 0x6006,
                        ),
                        m,
                    )
                });
                if rule == 2 {
                    for &m in order.iter().take(count) {
                        palette[id][m] = (before[id][m] + 1) % 5;
                    }
                } else {
                    let mut groups: Vec<Vec<usize>> = (0..5)
                        .map(|color| {
                            order
                                .iter()
                                .copied()
                                .filter(|&m| usize::from(before[id][m]) == color)
                                .rev()
                                .collect()
                        })
                        .collect();
                    for _ in 0..count / 2 {
                        let mut colors: Vec<_> = (0..5).collect();
                        colors.sort_by_key(|&c| (std::cmp::Reverse(groups[c].len()), c));
                        if groups[colors[1]].is_empty() {
                            return Err(io::Error::other("infeasible shuffled swap count"));
                        }
                        let a = groups[colors[0]]
                            .pop()
                            .ok_or_else(|| io::Error::other("empty color group"))?;
                        let b = groups[colors[1]]
                            .pop()
                            .ok_or_else(|| io::Error::other("empty color group"))?;
                        palette[id][a] = before[id][b];
                        palette[id][b] = before[id][a];
                    }
                }
            }
        }
        _ => return Err(io::Error::other("unknown causal variant")),
    }
    Ok(palette)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_physics_reset_energy_and_serialization() {
        let mut old = crate::morphic_0005::Medium::new(
            7,
            Configuration::Heterogeneous,
            0.025,
            "A",
            "regression",
        )
        .unwrap();
        let mut new = Medium::new(
            7,
            Configuration::Heterogeneous,
            0.025,
            "A",
            27,
            "regression",
        )
        .unwrap();
        for _ in 0..4 {
            old.advance(0., true).unwrap();
            new.advance(0., true).unwrap();
        }
        assert_eq!(old.state.u, new.state.u);
        assert_eq!(old.state.v, new.state.v);
        assert_eq!(old.history(), new.history());
        assert_eq!(new.energy().total(), old.energy().total());
        let original = new.state.palette.clone();
        let mut other = original.clone();
        other[0][0] = (other[0][0] + 1) % 5;
        assert_ne!(other[0], original[0]);
        assert_eq!(other[1], original[1]);
        new.reset_impulse(9, "A", 1.).unwrap();
        let amplitude = new.match_energy(1.).unwrap();
        assert!(amplitude > 0.);
        assert!((new.energy().total() - 1.).abs() < 1e-12);
        let bytes = new.state.encode().unwrap();
        assert_eq!(Frame::decode(&bytes).unwrap(), new.state);
        let mut bad = bytes;
        bad[90] ^= 1;
        assert!(Frame::decode(&bad).is_err());
        new.reset_impulse(9, "zero", 0.).unwrap();
        assert!(new.match_energy(1.).is_err());
        new.advance(0., true).unwrap();
        assert_eq!(new.adapt_rule(2).unwrap(), 0);
        let before = new.energy().total();
        new.apply_palette(other).unwrap();
        assert!((new.energy().total() - before - new.switch_work).abs() < 1e-12);
    }
    #[test]
    fn multi_edit_ties_boundaries_swaps_and_causal_masks() {
        let mut before = vec![[0; 64]; POPULATION];
        for (m, c) in before[2182].iter_mut().enumerate() {
            *c = u8::try_from(m % 5).unwrap();
        }
        let zero = vec![0.; FIELDS];
        let mut scores = zero.clone();
        scores[2182 * 64..2182 * 64 + 10].fill(1e-8);
        let r2 = morph_palette(&before, &zero, &scores, 2).unwrap();
        assert_eq!(
            r2[2182]
                .iter()
                .zip(before[2182])
                .filter(|(a, b)| **a != *b)
                .count(),
            8
        );
        for m in 0..8 {
            assert_eq!(r2[2182][m], (before[2182][m] + 1) % 5);
        }
        assert_eq!(r2[2182][8], before[2182][8]);
        let r3 = morph_palette(&before, &zero, &scores, 3).unwrap();
        let mut a = before[2182];
        let mut b = r3[2182];
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
        let edits = r3[2182]
            .iter()
            .zip(before[2182])
            .filter(|(a, b)| **a != *b)
            .count();
        assert_eq!(edits, 8);
        assert_eq!(internal(0), vec![1, 8]);
        assert!(!internal(7).contains(&8));
        let frozen = morph_palette(&before, &zero, &scores, 4).unwrap();
        assert_eq!(frozen, before);
        let source = variant(&before, &r2, "SOURCE_ONLY", 2, 7).unwrap();
        assert_eq!(source, r2);
        assert_eq!(
            variant(&before, &r2, "OFF_SOURCE_ONLY", 2, 7).unwrap(),
            before
        );
        let shuffled = variant(&before, &r3, "SHUFFLE_POSITION", 3, 7).unwrap();
        let mut s = shuffled[2182];
        s.sort_unstable();
        assert_eq!(s, a);
        assert_eq!(
            shuffled[2182]
                .iter()
                .zip(before[2182])
                .filter(|(a, b)| **a != *b)
                .count(),
            8
        );
        assert_eq!(before[0], [0; 64]);
    }
    #[test]
    fn uniform_analytic_mode_refines_without_graph_force() {
        let mut errors = Vec::new();
        let omega = 1.4_f64;
        let gamma = 0.1_f64;
        let time = 0.1_f64;
        let frequency = (omega * omega - gamma * gamma / 4.).sqrt();
        let exact = (-gamma * time / 2.).exp()
            * ((frequency * time).cos() + gamma / (2. * frequency) * (frequency * time).sin());
        for dt in [0.05, 0.025, 0.0125] {
            let mut medium = Medium::new(
                7,
                Configuration::Homogeneous,
                dt,
                "zero",
                27,
                "analytic-test",
            )
            .unwrap();
            medium.omega2.fill(omega * omega);
            medium.gamma.fill(gamma);
            medium.refresh_damping();
            medium.state.u.fill(1.);
            medium.acceleration();
            while f64::from(u32::try_from(medium.state.tick).unwrap()) * dt < time - 1e-12 {
                medium.advance(0., false).unwrap();
            }
            errors.push((medium.state.u[0] - exact).abs());
            assert_eq!(medium.state.u[0], medium.state.u[FIELDS - 1]);
        }
        assert!(errors[1] / errors[0] < 0.26 && errors[2] / errors[1] < 0.26);
    }
}
