//! Registered 0003 runner: authoritative headless measurements and bounded replay.

use crate::lattice::{Case, Configuration, DT, Drive, Lattice, TICKS};
use crate::observation::{Snapshot, checksum};
use crate::topology::{POPULATION, Topology};
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

/// Registered 111-case schedule; camera/UI settings never appear here.
///
/// # Panics
/// Panics only if a source constant contradicts the registered topology extents.
#[must_use]
pub fn cases() -> Vec<Case> {
    let mut result = Vec::new();
    for topology in [Topology::Cubic6, Topology::Cubic26] {
        for seed in [7, 19, 41] {
            for configuration in [
                Configuration::Heterogeneous,
                Configuration::Homogeneous,
                Configuration::Rearranged,
            ] {
                for position in [[8, 8, 8], [0, 0, 0], [0, 8, 8], [0, 0, 8]] {
                    result.push(Case {
                        topology,
                        seed,
                        configuration,
                        source: topology.id(position).expect("registered source"),
                        channel: 0,
                        drive: Drive::Impulse,
                    });
                }
                for drive in [Drive::LowTone, Drive::HighTone] {
                    result.push(Case {
                        topology,
                        seed,
                        configuration,
                        source: topology.id([8, 8, 8]).expect("center"),
                        channel: 0,
                        drive,
                    });
                }
            }
        }
        result.push(Case {
            topology,
            seed: 7,
            configuration: Configuration::Homogeneous,
            source: 2184,
            channel: 0,
            drive: Drive::Zero,
        });
    }
    result.push(Case {
        topology: Topology::Grid2D8,
        source: 2080,
        ..Case::default()
    });
    result
}

fn geometry(folder: &Path) -> io::Result<()> {
    let mut degree = BufWriter::new(File::create(folder.join("degrees.csv"))?);
    let mut edges = BufWriter::new(File::create(folder.join("link_lengths.csv"))?);
    writeln!(degree, "topology,degree,neurons")?;
    writeln!(edges, "topology,squared_length,undirected_links")?;
    for topology in [Topology::Cubic6, Topology::Cubic26, Topology::Grid2D8] {
        let mut counts = [0usize; 27];
        let mut lengths = [0usize; 4];
        for id in 0..POPULATION {
            let links = topology.neighbors(id);
            counts[links.len()] += 1;
            let a = topology.coordinates(id);
            for j in links {
                if j > id {
                    let b = topology.coordinates(j);
                    let square: usize = (0..3).map(|i| a[i].abs_diff(b[i]).pow(2)).sum();
                    lengths[square] += 1;
                }
            }
        }
        for (d, n) in counts.iter().enumerate().filter(|(_, n)| **n > 0) {
            writeln!(degree, "{},{d},{n}", topology.name())?;
        }
        for (square, n) in lengths.iter().enumerate().skip(1) {
            writeln!(edges, "{},{square},{n}", topology.name())?;
        }
    }
    Ok(())
}

fn record_case(id: usize, case: Case, folder: &Path, hashes: &mut impl Write) -> io::Result<()> {
    let path = folder.join(format!("case-{id:04}"));
    fs::create_dir(&path)?;
    let mut model = Lattice::new(case)?;
    let mut metrics = BufWriter::new(File::create(path.join("metrics.csv"))?);
    writeln!(
        metrics,
        "tick,force,peak,velocity_peak,sum_u2,sum_v2,nonfinite,source_u27,source_v27"
    )?;
    let mut peaks = vec![0.0_f64; POPULATION];
    let mut arrivals = vec![None; POPULATION];
    let initial = model.observe();
    let initial_palette: Vec<_> = initial.neurons.iter().flat_map(|n| n.palette).collect();
    let initial_hash = checksum(&initial_palette);
    let mut valid = true;
    loop {
        let snapshot = model.observe();
        let tick = snapshot.tick;
        let d = snapshot.diagnostic();
        valid &= d.valid();
        let n = &snapshot.neurons[case.source];
        writeln!(
            metrics,
            "{tick},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{:.17e},{:.17e}",
            model
                .force(usize::try_from(tick).expect("tick"))
                .unwrap_or(0.0),
            d.peak,
            d.velocity_peak,
            d.squared,
            d.velocity_squared,
            d.nonfinite,
            n.module27[case.channel],
            n.velocity27[case.channel]
        )?;
        for neuron in &snapshot.neurons {
            let peak = neuron.peak.iter().copied().fold(0.0_f64, f64::max);
            peaks[neuron.id] = peaks[neuron.id].max(peak);
            if arrivals[neuron.id].is_none() && peak >= 1e-6 {
                arrivals[neuron.id] = Some(tick);
            }
        }
        if tick.is_multiple_of(8) || !valid || tick == TICKS {
            let file = path.join(format!("frame-{tick:06}.qrs"));
            snapshot.save(&file)?;
            let readback = Snapshot::load(&file)?;
            if readback != snapshot {
                return Err(io::Error::other("snapshot readback mismatch"));
            }
            writeln!(hashes, "{id},{tick},{:016x}", snapshot.fingerprint()?)?;
        }
        if !valid || !model.advance() {
            break;
        }
    }
    let last = model.observe();
    let final_palette: Vec<_> = last.neurons.iter().flat_map(|n| n.palette).collect();
    if initial_palette != final_palette {
        return Err(io::Error::other("persistent configuration changed"));
    }
    let mut spatial = BufWriter::new(File::create(path.join("spatial.csv"))?);
    writeln!(spatial, "id,x,y,z,hops,euclidean,peak,arrival_tick")?;
    let source = case.topology.coordinates(case.source);
    for neuron in &last.neurons {
        let squared: u32 = (0..3)
            .map(|i| {
                u32::try_from(neuron.position[i].abs_diff(source[i]).pow(2)).expect("distance")
            })
            .sum();
        writeln!(
            spatial,
            "{},{},{},{},{},{:.17e},{:.17e},{}",
            neuron.id,
            neuron.position[0],
            neuron.position[1],
            neuron.position[2],
            case.topology.hops(case.source, neuron.id),
            f64::from(squared).sqrt(),
            peaks[neuron.id],
            arrivals[neuron.id].map_or_else(String::new, |t| t.to_string())
        )?;
    }
    fs::write(
        path.join("status.txt"),
        format!(
            "valid={valid}\ncompleted_tick={}\npersistent_fnv1a={initial_hash:016x}\n",
            model.tick()
        ),
    )?;
    eprintln!(
        "0003 case {id}: {} seed{} {} {} source{} valid={valid}",
        case.topology.name(),
        case.seed,
        case.configuration.name(),
        case.drive.name(),
        case.source
    );
    Ok(())
}

/// Execute all registered cases, recording exact state without a graphics context.
///
/// # Errors
/// Rejects an existing folder, failed snapshot consistency, or filesystem errors.
pub fn run(folder: &Path) -> io::Result<()> {
    fs::create_dir(folder)?;
    geometry(folder)?;
    fs::write(
        folder.join("registration.md"),
        include_str!("../EXPERIMENT_0003.md"),
    )?;
    fs::write(folder.join("engine_source.rs"), include_str!("lattice.rs"))?;
    fs::write(
        folder.join("topology_source.rs"),
        include_str!("topology.rs"),
    )?;
    fs::write(
        folder.join("observation_source.rs"),
        include_str!("observation.rs"),
    )?;
    fs::write(
        folder.join("runner_source.rs"),
        include_str!("experiment_0003.rs"),
    )?;
    let mut manifest = BufWriter::new(File::create(folder.join("cases.csv"))?);
    let mut hashes = BufWriter::new(File::create(folder.join("hashes.csv"))?);
    writeln!(
        manifest,
        "case,topology,seed,configuration,source,x,y,z,drive,channel,dt,ticks,external_divisor"
    )?;
    writeln!(hashes, "case,tick,fnv1a")?;
    let cases = cases();
    if cases.len() != 111 {
        return Err(io::Error::other("schedule coverage mismatch"));
    }
    for (id, case) in cases.into_iter().enumerate() {
        let [x, y, z] = case.topology.coordinates(case.source);
        writeln!(
            manifest,
            "{id},{},{},{},{},{x},{y},{z},{},1,{DT},{TICKS},{}",
            case.topology.name(),
            case.seed,
            case.configuration.name(),
            case.source,
            case.drive.name(),
            case.topology.maximum_degree()
        )?;
        manifest.flush()?;
        record_case(id, case, folder, &mut hashes)?;
        hashes.flush()?;
    }
    fs::write(
        folder.join("COMPLETE"),
        "111 registered cases attempted. Inspect validity and measured diagnostics; completion is not a scientific success claim.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schedule_and_legacy_reference_reproduction() {
        assert_eq!(cases().len(), 111);
        let case = *cases().last().unwrap();
        let mut model = Lattice::new(case).unwrap();
        for row in include_str!("../tests/fixtures/0002-grid-reference.csv")
            .lines()
            .skip(1)
        {
            let fields: Vec<_> = row.split(',').collect();
            let tick: u64 = fields[0].parse().unwrap();
            while model.tick() < tick {
                model.advance();
            }
            let snapshot = model.observe();
            let n = &snapshot.neurons[case.source];
            let expected: f64 = fields[1].parse().unwrap();
            assert_eq!(n.module27[0].to_bits(), expected.to_bits());
        }
    }
}
