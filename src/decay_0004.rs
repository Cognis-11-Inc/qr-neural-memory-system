//! Every-tick, bounded disk replay for the separately registered 3072-tick extension.
use crate::{
    lattice::{Case, Configuration, Drive},
    observation::{Neuron, Snapshot, checksum},
    propagation_0004::{Arm, CHECKPOINTS, Frame, Medium, SOURCE_A, SOURCE_B},
    topology::{POPULATION, Topology},
};
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

/// Requested maximum simulation tick, inclusive.
pub const LAST_TICK: u64 = 3072;
/// Frozen integration timestep.
pub const DT: f64 = 0.1;
/// Explicit numerical quiet threshold for both displacement and velocity.
pub const QUIET: f64 = 1e-6;
/// Minimum final consecutive quiet ticks before reporting sustained measured quiet.
pub const QUIET_TAIL: u64 = 128;
const MODEL: &[u8] = b"qr4-decay-v1;physics=0002;extension=2c39004";
const PALETTE_BYTES: usize = POPULATION * 64;
const HEADER_SIZE: usize = 52 + MODEL.len() + PALETTE_BYTES + 8;
const FRAME_SIZE: usize = 8 + POPULATION * 52 + 8;
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn number(value: u64) -> f64 {
    f64::from(u32::try_from(value).expect("bounded tick"))
}
fn u64_at(bytes: &[u8], offset: usize) -> io::Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(|| invalid("truncated integer"))?
            .try_into()
            .map_err(io::Error::other)?,
    ))
}
fn float_at(bytes: &[u8], offset: usize) -> io::Result<f64> {
    Ok(f64::from_bits(u64_at(bytes, offset)?))
}
fn header(palette: &[[u8; 64]], last: u64) -> io::Result<Vec<u8>> {
    if palette.len() != POPULATION
        || palette.iter().flatten().any(|&v| v != 1)
        || !(1..=LAST_TICK).contains(&last)
    {
        return Err(invalid("invalid fixed decay fixture"));
    }
    let mut bytes = Vec::with_capacity(HEADER_SIZE);
    bytes.extend_from_slice(b"QR4DEC01");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&7u64.to_le_bytes());
    bytes.extend_from_slice(&DT.to_le_bytes());
    bytes.extend_from_slice(&last.to_le_bytes());
    for source in [SOURCE_A, SOURCE_B] {
        bytes.extend_from_slice(
            &u32::try_from(source)
                .map_err(io::Error::other)?
                .to_le_bytes(),
        );
    }
    bytes.extend_from_slice(&[1, 2, 0, 0]);
    bytes.extend_from_slice(
        &u32::try_from(MODEL.len())
            .map_err(io::Error::other)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(MODEL);
    for p in palette {
        bytes.extend_from_slice(p);
    }
    bytes.extend_from_slice(&checksum(&bytes).to_le_bytes());
    Ok(bytes)
}
fn encode_frame(snapshot: &Snapshot) -> io::Result<Vec<u8>> {
    if snapshot.neurons.len() != POPULATION || snapshot.tick > LAST_TICK {
        return Err(invalid("invalid decay observation"));
    }
    let mut bytes = Vec::with_capacity(FRAME_SIZE);
    bytes.extend_from_slice(&snapshot.tick.to_le_bytes());
    for n in &snapshot.neurons {
        for value in [
            n.peak[0],
            n.velocity_peak[0],
            n.module27[0],
            n.velocity27[0],
            n.squared,
            n.velocity_squared,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&n.nonfinite.to_le_bytes());
    }
    bytes.extend_from_slice(&checksum(&bytes).to_le_bytes());
    Ok(bytes)
}
fn decode_frame(bytes: &[u8], tick: u64, palette: &[[u8; 64]]) -> io::Result<Snapshot> {
    if bytes.len() != FRAME_SIZE
        || u64_at(bytes, 0)? != tick
        || u64_at(bytes, FRAME_SIZE - 8)? != checksum(&bytes[..FRAME_SIZE - 8])
    {
        return Err(invalid("decay frame tick/length/checksum mismatch"));
    }
    let mut neurons = Vec::with_capacity(POPULATION);
    for (id, &tile) in palette.iter().enumerate() {
        let at = 8 + 52 * id;
        let values = (0..6)
            .map(|j| float_at(bytes, at + j * 8))
            .collect::<io::Result<Vec<_>>>()?;
        let nonfinite = u32::from_le_bytes(
            bytes[at + 48..at + 52]
                .try_into()
                .map_err(io::Error::other)?,
        );
        if nonfinite != 0
            || values.iter().any(|v| !v.is_finite())
            || values[0] < 0.0
            || values[1] < 0.0
            || values[0] > 100.0
            || values[1] > 100.0
            || values[4] < 0.0
            || values[5] < 0.0
        {
            return Err(invalid("decay frame numerical screen failed"));
        }
        let mut n = Neuron {
            id,
            position: Topology::Cubic6.coordinates(id),
            palette: tile,
            nonfinite,
            ..Neuron::default()
        };
        n.peak[0] = values[0];
        n.velocity_peak[0] = values[1];
        n.module27[0] = values[2];
        n.velocity27[0] = values[3];
        n.squared = values[4];
        n.velocity_squared = values[5];
        neurons.push(n);
    }
    Ok(Snapshot {
        case: Case {
            topology: Topology::Cubic6,
            seed: 7,
            configuration: Configuration::Homogeneous,
            source: SOURCE_A,
            channel: 0,
            drive: Drive::Zero,
        },
        tick,
        synthetic: false,
        neurons,
    })
}

/// Checksum of exactly the displayed observation fields in stream encoding order.
///
/// # Errors
/// Rejects invalid observation dimensions, tick or encoding errors.
pub fn observation_checksum(snapshot: &Snapshot) -> io::Result<u64> {
    let bytes = encode_frame(snapshot)?;
    u64_at(&bytes, FRAME_SIZE - 8)
}

/// Indexed on-disk replay; resident state is bounded independently of recording length.
pub struct DecayRecording {
    file: BufReader<File>,
    palette: Vec<[u8; 64]>,
    last: u64,
}
impl DecayRecording {
    /// Open a completed, size-bounded measured recording without a physics engine.
    ///
    /// # Errors
    /// Rejects missing completion, unknown model, corruption, invalid size or fixture.
    pub fn open(folder: &Path) -> io::Result<Self> {
        if !folder.join("COMPLETE").exists() {
            return Err(invalid("decay recording incomplete"));
        }
        let mut file = BufReader::new(File::open(folder.join("replay.q4d"))?);
        let mut bytes = vec![0; HEADER_SIZE];
        file.read_exact(&mut bytes)?;
        if &bytes[..8] != b"QR4DEC01"
            || bytes[8..12] != 1u32.to_le_bytes()
            || u64_at(&bytes, 12)? != 7
            || float_at(&bytes, 20)?.to_bits() != DT.to_bits()
            || bytes[36..40] != 2182u32.to_le_bytes()
            || bytes[40..44] != 2186u32.to_le_bytes()
            || bytes[44..48] != [1, 2, 0, 0]
            || bytes[48..52]
                != u32::try_from(MODEL.len())
                    .map_err(io::Error::other)?
                    .to_le_bytes()
            || &bytes[52..52 + MODEL.len()] != MODEL
            || u64_at(&bytes, HEADER_SIZE - 8)? != checksum(&bytes[..HEADER_SIZE - 8])
        {
            return Err(invalid("decay header/schema/provenance/checksum mismatch"));
        }
        let last = u64_at(&bytes, 28)?;
        if !(1..=LAST_TICK).contains(&last)
            || file.get_ref().metadata()?.len()
                != u64::try_from(HEADER_SIZE).map_err(io::Error::other)?
                    + (last + 1) * u64::try_from(FRAME_SIZE).map_err(io::Error::other)?
        {
            return Err(invalid("decay recording length/tick bounds mismatch"));
        }
        let palette = bytes[52 + MODEL.len()..HEADER_SIZE - 8]
            .as_chunks::<64>()
            .0
            .to_vec();
        if palette.iter().flatten().any(|&x| x != 1) {
            return Err(invalid("decay homogeneous palette mismatch"));
        }
        Ok(Self {
            file,
            palette,
            last,
        })
    }
    /// Total contiguous observations, including tick0.
    #[must_use]
    pub fn len(&self) -> usize {
        usize::try_from(self.last + 1).unwrap_or(0)
    }
    /// Whether the recording has no frames; validated recordings are never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
    /// Read exactly one requested measured tick, never advancing a simulation.
    ///
    /// # Errors
    /// Rejects out-of-range seeks, I/O failures and corrupted/invalid frame data.
    pub fn frame(&mut self, index: usize) -> io::Result<Snapshot> {
        let tick = u64::try_from(index).map_err(io::Error::other)?;
        if tick > self.last {
            return Err(invalid("decay seek out of range"));
        }
        let offset = u64::try_from(HEADER_SIZE + index * FRAME_SIZE).map_err(io::Error::other)?;
        self.file.seek(SeekFrom::Start(offset))?;
        let mut bytes = vec![0; FRAME_SIZE];
        self.file.read_exact(&mut bytes)?;
        decode_frame(&bytes, tick, &self.palette)
    }
}

fn record(folder: &Path, reference: Option<&Path>, last: u64) -> io::Result<()> {
    fs::create_dir(folder)?;
    fs::write(
        folder.join("registration.md"),
        include_str!("../EXPERIMENT_0004_EXTENSION.md"),
    )?;
    fs::write(folder.join("recorder.rs"), include_str!("decay_0004.rs"))?;
    let mut medium = Medium::new(7, Configuration::Homogeneous, DT, Arm::Plus)?;
    let frozen = medium.state.palette.clone();
    let mut stream = BufWriter::new(File::create(folder.join("replay.q4d"))?);
    stream.write_all(&header(&frozen, last)?)?;
    let mut metrics = BufWriter::new(File::create(folder.join("metrics.csv"))?);
    let mut prefix = BufWriter::new(File::create(folder.join("prefix.csv"))?);
    writeln!(
        metrics,
        "tick,time,peak,velocity_peak,sum_u2,sum_v2,nonfinite,active_neurons,mid_u27,mid_v27,below_threshold"
    )?;
    writeln!(prefix, "tick,full_state_equal")?;
    let mut last_above = 0;
    let (mut final_u, mut final_v) = (0.0, 0.0);
    for tick in 0..=last {
        if medium.state.palette != frozen {
            return Err(invalid("decay persistent palette changed"));
        }
        let snapshot = medium.state.observation();
        let d = snapshot.diagnostic();
        let quiet = d.peak < QUIET && d.velocity_peak < QUIET;
        if !quiet {
            last_above = tick;
        }
        let mid = &snapshot.neurons[2184];
        let active = snapshot
            .neurons
            .iter()
            .filter(|n| n.peak[0] >= QUIET)
            .count();
        writeln!(
            metrics,
            "{tick},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{active},{:.17e},{:.17e},{quiet}",
            number(tick) * DT,
            d.peak,
            d.velocity_peak,
            d.squared,
            d.velocity_squared,
            d.nonfinite,
            mid.module27[0],
            mid.velocity27[0]
        )?;
        if !d.valid() {
            metrics.flush()?;
            fs::write(
                folder.join("INVALID"),
                format!("Numerical screen failed tick{tick}"),
            )?;
            return Err(invalid("decay stopped on numerical screen"));
        }
        if let Some(path) = reference.filter(|_| CHECKPOINTS.contains(&tick)) {
            let old = Frame::decode(&fs::read(path.join(format!("frame-{tick:06}.q4f")))?)?;
            let same = old == medium.state;
            writeln!(prefix, "{tick},{same}")?;
            if !same {
                return Err(invalid("decay original prefix mismatch"));
            }
        }
        let bytes = encode_frame(&snapshot)?;
        let decoded = decode_frame(&bytes, tick, &frozen)?;
        if decoded != snapshot {
            return Err(invalid("decay exact observation readback mismatch"));
        }
        stream.write_all(&bytes)?;
        final_u = d.peak;
        final_v = d.velocity_peak;
        if tick % 256 == 0 {
            eprintln!(
                "decay tick{tick}/{last} maxu={:.6e} maxv={:.6e}",
                d.peak, d.velocity_peak
            );
        }
        if tick < last {
            medium.advance();
        }
    }
    let tail = last - last_above;
    fs::write(
        folder.join("summary.csv"),
        format!(
            "last_tick,final_peak,final_velocity_peak,last_at_or_above,quiet_tail,sustained_quiet,exact_zero\n{last},{final_u:.17e},{final_v:.17e},{last_above},{tail},{},{}\n",
            tail >= QUIET_TAIL,
            final_u.to_bits() == 0 && final_v.to_bits() == 0
        ),
    )?;
    stream.flush()?;
    metrics.flush()?;
    prefix.flush()?;
    fs::write(
        folder.join("COMPLETE"),
        "Every integer tick recorded, frozen fixture; below-threshold is not mathematical zero.\n",
    )
}
/// Run the user-requested, separately registered 3072-tick frozen-fixture extension.
///
/// # Errors
/// Rejects existing output, numerical or prefix mismatch, and I/O failures.
pub fn run(folder: &Path, reference: Option<&Path>) -> io::Result<()> {
    record(folder, reference, LAST_TICK)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_tick_random_seek_prefix_and_corruption() {
        let folder = std::env::temp_dir().join(format!("qr-decay-test-{}", std::process::id()));
        if folder.exists() {
            fs::remove_dir_all(&folder).unwrap();
        }
        record(&folder, None, 3).unwrap();
        let mut replay = DecayRecording::open(&folder).unwrap();
        assert_eq!(replay.len(), 4);
        let mut expected = Medium::new(7, Configuration::Homogeneous, DT, Arm::Plus).unwrap();
        let mut snapshots = vec![];
        for _ in 0..4 {
            snapshots.push(expected.state.observation());
            expected.advance();
        }
        for i in [3, 0, 2, 1, 3] {
            assert_eq!(replay.frame(i).unwrap(), snapshots[i]);
        }
        assert!(replay.frame(4).is_err());
        let path = folder.join("replay.q4d");
        let mut bytes = fs::read(&path).unwrap();
        bytes[HEADER_SIZE + FRAME_SIZE + 100] ^= 1;
        fs::write(&path, &bytes).unwrap();
        let mut corrupt = DecayRecording::open(&folder).unwrap();
        assert!(corrupt.frame(1).is_err());
        assert!(corrupt.frame(0).is_ok());
        bytes[100] ^= 1;
        fs::write(path, bytes).unwrap();
        assert!(DecayRecording::open(&folder).is_err());
        fs::remove_dir_all(folder).unwrap();
    }
}
