//! Versioned, bounded observation-only snapshot and replay format.

use crate::lattice::{Case, Configuration, DT, Diagnostic, Drive, ENGINE_VERSION};
use crate::topology::{POPULATION, Topology};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 8] = b"QR3OBS01";
const SCHEMA: u32 = 1;

/// Per-neuron measured aggregates and actual persistent internal palette.
#[derive(Clone, Debug, PartialEq)]
pub struct Neuron {
    /// Stable, topology-independent address in 0..4096.
    pub id: usize,
    /// Integer lattice coordinates, not camera/display coordinates.
    pub position: [usize; 3],
    /// Actual row-major 8 × 8 persistent modules.
    pub palette: [u8; 64],
    /// Per-channel maximum absolute displacement over all 64 modules.
    pub peak: [f64; 4],
    /// Per-channel maximum absolute velocity over all modules.
    pub velocity_peak: [f64; 4],
    /// Signed per-channel displacement of internal module 27.
    pub module27: [f64; 4],
    /// Signed per-channel velocity of internal module 27.
    pub velocity27: [f64; 4],
    /// Sum of squared displacements, not physical energy.
    pub squared: f64,
    /// Sum of squared velocities.
    pub velocity_squared: f64,
    /// Nonfinite module/channel pair count.
    pub nonfinite: u32,
}
impl Default for Neuron {
    fn default() -> Self {
        Self {
            id: 0,
            position: [0; 3],
            palette: [0; 64],
            peak: [0.0; 4],
            velocity_peak: [0.0; 4],
            module27: [0.0; 4],
            velocity27: [0.0; 4],
            squared: 0.0,
            velocity_squared: 0.0,
            nonfinite: 0,
        }
    }
}

/// Read-only authoritative observation. LIVE/REPLAY is a viewer mode, not physics.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    /// Complete physics fixture identity.
    pub case: Case,
    /// Exact engine tick.
    pub tick: u64,
    /// Explicit synthetic-fixture provenance; false for actual engine observations.
    pub synthetic: bool,
    /// Stable-ID-ordered measured neuron data.
    pub neurons: Vec<Neuron>,
}

/// FNV-1a corruption checksum; this is not cryptographic authentication.
#[must_use]
pub fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn put32(bytes: &mut Vec<u8>, value: usize) -> io::Result<()> {
    bytes.extend_from_slice(
        &u32::try_from(value)
            .map_err(|_| invalid("integer overflow"))?
            .to_le_bytes(),
    );
    Ok(())
}
fn put_float(bytes: &mut Vec<u8>, value: f64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or_else(|| invalid("length overflow"))?;
        let result = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid("truncated snapshot"))?
            .try_into()
            .map_err(|_| invalid("invalid field length"))?;
        self.offset = end;
        Ok(result)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take::<1>()?[0])
    }
    fn u32(&mut self) -> io::Result<usize> {
        usize::try_from(u32::from_le_bytes(self.take()?)).map_err(|_| invalid("integer width"))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn float(&mut self) -> io::Result<f64> {
        Ok(f64::from_le_bytes(self.take()?))
    }
    fn floats(&mut self) -> io::Result<[f64; 4]> {
        Ok([self.float()?, self.float()?, self.float()?, self.float()?])
    }
}

impl Snapshot {
    fn validate(&self) -> io::Result<()> {
        if self.neurons.len() != POPULATION
            || self.case.source >= POPULATION
            || self.case.channel >= 4
            || self.tick > 160
        {
            return Err(invalid("invalid population/source/channel/tick"));
        }
        for (id, n) in self.neurons.iter().enumerate() {
            if n.id != id
                || n.position != self.case.topology.coordinates(id)
                || n.palette.iter().any(|&x| x > 4)
            {
                return Err(invalid("invalid neuron ID/coordinates/palette"));
            }
        }
        Ok(())
    }
    /// Aggregate measured fields; display settings do not enter this calculation.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        self.neurons.iter().fold(Diagnostic::default(), |mut d, n| {
            for k in 0..4 {
                d.peak = d.peak.max(n.peak[k]);
                d.velocity_peak = d.velocity_peak.max(n.velocity_peak[k]);
            }
            d.squared += n.squared;
            d.velocity_squared += n.velocity_squared;
            d.nonfinite = d
                .nonfinite
                .saturating_add(usize::try_from(n.nonfinite).unwrap_or(usize::MAX));
            d
        })
    }
    /// Encode exact measured values in schema 1 with a trailing corruption checksum.
    ///
    /// # Errors
    /// Rejects invalid observation dimensions, IDs, palette or integer fields.
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        self.validate()?;
        let mut b = Vec::with_capacity(1_000_000);
        b.extend_from_slice(MAGIC);
        b.extend_from_slice(&SCHEMA.to_le_bytes());
        b.extend_from_slice(&[
            self.case.topology as u8,
            self.case.configuration as u8,
            self.case.drive as u8,
            u8::try_from(self.case.channel).map_err(|_| invalid("channel"))?,
        ]);
        b.extend_from_slice(&self.case.seed.to_le_bytes());
        put32(&mut b, self.case.source)?;
        b.extend_from_slice(&self.tick.to_le_bytes());
        put_float(&mut b, DT);
        b.push(u8::from(self.synthetic));
        put32(&mut b, ENGINE_VERSION.len())?;
        b.extend_from_slice(ENGINE_VERSION.as_bytes());
        for d in self.case.topology.dimensions() {
            put32(&mut b, d)?;
        }
        put32(&mut b, POPULATION)?;
        for n in &self.neurons {
            put32(&mut b, n.id)?;
            for p in n.position {
                put32(&mut b, p)?;
            }
            b.extend_from_slice(&n.palette);
            for array in [n.peak, n.velocity_peak, n.module27, n.velocity27] {
                for value in array {
                    put_float(&mut b, value);
                }
            }
            put_float(&mut b, n.squared);
            put_float(&mut b, n.velocity_squared);
            b.extend_from_slice(&n.nonfinite.to_le_bytes());
        }
        let hash = checksum(&b);
        b.extend_from_slice(&hash.to_le_bytes());
        Ok(b)
    }
    /// Decode exact observation values; no engine executes during replay.
    ///
    /// # Errors
    /// Rejects corruption, unknown versions, invalid provenance/fields and lengths.
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 64 || bytes.len() > 2_000_000 {
            return Err(invalid("snapshot length outside bounds"));
        }
        let payload = bytes.len() - 8;
        let expected = u64::from_le_bytes(
            bytes[payload..]
                .try_into()
                .map_err(|_| invalid("checksum length"))?,
        );
        if checksum(&bytes[..payload]) != expected {
            return Err(invalid("snapshot checksum mismatch"));
        }
        let mut r = Reader {
            bytes: &bytes[..payload],
            offset: 0,
        };
        if &r.take::<8>()? != MAGIC || r.u32()? != 1 {
            return Err(invalid("unsupported observation schema"));
        }
        let topology = Topology::try_from(r.u8()?)?;
        let configuration = Configuration::try_from(r.u8()?)?;
        let drive = Drive::try_from(r.u8()?)?;
        let channel = usize::from(r.u8()?);
        let seed = r.u64()?;
        let source = r.u32()?;
        let tick = r.u64()?;
        if r.float()?.to_bits() != DT.to_bits() {
            return Err(invalid("incompatible dt"));
        }
        let origin = r.u8()?;
        if origin > 1 {
            return Err(invalid("unknown source provenance"));
        }
        let length = r.u32()?;
        if length != ENGINE_VERSION.len() {
            return Err(invalid("incompatible engine version"));
        }
        for expected in ENGINE_VERSION.bytes() {
            if r.u8()? != expected {
                return Err(invalid("incompatible engine provenance"));
            }
        }
        for expected in topology.dimensions() {
            if r.u32()? != expected {
                return Err(invalid("dimension mismatch"));
            }
        }
        if r.u32()? != POPULATION {
            return Err(invalid("population mismatch"));
        }
        let mut neurons = Vec::with_capacity(POPULATION);
        for _ in 0..POPULATION {
            let id = r.u32()?;
            let position = [r.u32()?, r.u32()?, r.u32()?];
            let palette = r.take::<64>()?;
            let peak = r.floats()?;
            let velocity_peak = r.floats()?;
            let module27 = r.floats()?;
            let velocity27 = r.floats()?;
            let squared = r.float()?;
            let velocity_squared = r.float()?;
            let nonfinite = u32::try_from(r.u32()?).map_err(|_| invalid("nonfinite count"))?;
            neurons.push(Neuron {
                id,
                position,
                palette,
                peak,
                velocity_peak,
                module27,
                velocity27,
                squared,
                velocity_squared,
                nonfinite,
            });
        }
        if r.offset != payload {
            return Err(invalid("unexpected trailing observation data"));
        }
        let snapshot = Self {
            case: Case {
                topology,
                seed,
                configuration,
                source,
                channel,
                drive,
            },
            tick,
            synthetic: origin == 1,
            neurons,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
    /// Stable checksum of the canonical encoded observation.
    ///
    /// # Errors
    /// Returns encoding validation errors.
    pub fn fingerprint(&self) -> io::Result<u64> {
        let b = self.encode()?;
        Ok(checksum(&b[..b.len() - 8]))
    }
    /// Write one exact observation.
    ///
    /// # Errors
    /// Returns validation or filesystem errors.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        fs::write(path, self.encode()?)
    }
    /// Load a bounded observation file, reporting corruption rather than guessing.
    ///
    /// # Errors
    /// Returns size, checksum, schema, field or filesystem errors.
    pub fn load(path: &Path) -> io::Result<Self> {
        if fs::metadata(path)?.len() > 2_000_000 {
            return Err(invalid("snapshot file too large"));
        }
        Self::decode(&fs::read(path)?)
    }
}

/// Bounded immutable recorded timeline; seeking performs lookup, not simulation.
pub struct Recording {
    /// Frames in strictly increasing tick order.
    pub frames: Vec<Snapshot>,
}
impl Recording {
    /// Read up to 256 observations from one file or a case folder.
    ///
    /// # Errors
    /// Rejects malformed files, mixed provenance, duplicate/unordered ticks or bounds.
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut paths: Vec<PathBuf> = if path.is_file() {
            vec![path.to_path_buf()]
        } else {
            fs::read_dir(path)?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "qrs"))
                .collect()
        };
        paths.sort();
        if paths.is_empty() || paths.len() > 256 {
            return Err(invalid("recording needs 1..256 frames"));
        }
        let frames: Vec<_> = paths
            .iter()
            .map(|p| Snapshot::load(p))
            .collect::<io::Result<_>>()?;
        if frames.windows(2).any(|p| {
            p[0].tick >= p[1].tick || p[0].case != p[1].case || p[0].synthetic != p[1].synthetic
        }) {
            return Err(invalid("unordered/mixed recording"));
        }
        Ok(Self { frames })
    }
    /// Exact tick lookup, leaving scientific and displayed values unchanged.
    #[must_use]
    pub fn seek(&self, tick: u64) -> Option<usize> {
        self.frames.binary_search_by_key(&tick, |f| f.tick).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::Lattice;
    #[test]
    fn exact_roundtrip_provenance_and_corruption_rejection() {
        let model = Lattice::new(Case::default()).unwrap();
        let snapshot = model.observe();
        let bytes = snapshot.encode().unwrap();
        assert_eq!(Snapshot::decode(&bytes).unwrap(), snapshot);
        let mut corrupted = bytes.clone();
        corrupted[300] ^= 1;
        assert!(Snapshot::decode(&corrupted).is_err());
        assert!(Snapshot::decode(&bytes[..30]).is_err());
        let mut synthetic = snapshot.clone();
        synthetic.synthetic = true;
        assert!(
            Snapshot::decode(&synthetic.encode().unwrap())
                .unwrap()
                .synthetic
        );
        let recording = Recording {
            frames: vec![snapshot],
        };
        assert_eq!(recording.seek(0), Some(0));
        assert_eq!(recording.seek(8), None);
    }
}
