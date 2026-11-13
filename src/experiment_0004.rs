//! Registered extended propagation and independently integrated dual-wave controls.
use crate::{
    lattice::{Case, Configuration, Lattice},
    propagation_0004::{Arm, CHECKPOINTS, Frame, Medium},
    topology::POPULATION,
};
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

fn number(value: u64) -> f64 {
    f64::from(u32::try_from(value).expect("bounded step"))
}
struct Track {
    peaks: Vec<f64>,
    peaks16: Vec<f64>,
    arrivals: Vec<Option<u64>>,
    at16: usize,
}
impl Track {
    fn new() -> Self {
        Self {
            peaks: vec![0.0; POPULATION],
            peaks16: vec![0.0; POPULATION],
            arrivals: vec![None; POPULATION],
            at16: 0,
        }
    }
}
fn save(frame: &Frame, path: &Path, hashes: &mut impl Write, relative: &str) -> io::Result<()> {
    let bytes = frame.encode()?;
    if Frame::decode(&bytes)? != *frame {
        return Err(io::Error::other("0004 full-field readback mismatch"));
    }
    fs::write(path, &bytes)?;
    writeln!(
        hashes,
        "{relative},{:016x}",
        crate::observation::checksum(&bytes[..bytes.len() - 8])
    )
}
fn finish_track(track: &Track, folder: &Path, dt: f64) -> io::Result<()> {
    let mut out = BufWriter::new(File::create(folder.join("spatial.csv"))?);
    writeln!(out, "id,peak,peak_at_time16,arrival_step,arrival_time")?;
    for (id, &peak) in track.peaks.iter().enumerate() {
        let arrival = track.arrivals[id].map_or(String::new(), |x| x.to_string());
        let time = track.arrivals[id].map_or(String::new(), |x| format!("{:.17e}", number(x) * dt));
        writeln!(
            out,
            "{id},{peak:.17e},{:.17e},{arrival},{time}",
            track.peaks16[id]
        )?;
    }
    fs::write(
        folder.join("reach.csv"),
        format!(
            "at_time16,at_time102_4\n{},{}\n",
            track.at16,
            track.arrivals.iter().flatten().count()
        ),
    )
}
#[derive(Clone, Copy)]
struct Sample {
    squared: [f64; 4],
    vsquared: [f64; 4],
    peak: [f64; 4],
    vpeak: [f64; 4],
    nonfinite: [usize; 4],
    residual_u: [f64; 2],
    residual_v: [f64; 2],
    residual2: [f64; 2],
    expected2: [f64; 2],
    dot_u: f64,
    dot_v: f64,
    scale_u: f64,
    scale_v: f64,
    mid_peak: [f64; 4],
}
fn measure(engines: &[Medium], tracks: &mut [Track], step: u64) -> Sample {
    let mut squared = [0.0; 4];
    let mut vsquared = [0.0; 4];
    let mut peak = [0.0_f64; 4];
    let mut vpeak = [0.0_f64; 4];
    let mut nonfinite = [0; 4];
    let mut residual_u = [0.0_f64; 2];
    let mut residual_v = [0.0_f64; 2];
    let mut residual2 = [0.0; 2];
    let mut expected2 = [0.0; 2];
    let (mut dot_u, mut dot_v) = (0.0, 0.0);
    let (mut scale_u, mut scale_v) = (1.0_f64, 1.0_f64);
    let mut mid_peak = [0.0_f64; 4];
    for neuron in 0..POPULATION {
        let mut local = [0.0_f64; 4];
        for module in 0..64 {
            let index = neuron * 64 + module;
            let u: [f64; 4] = std::array::from_fn(|j| engines[j].state.u[index]);
            let v: [f64; 4] = std::array::from_fn(|j| engines[j].state.v[index]);
            for j in 0..4 {
                squared[j] += u[j] * u[j];
                vsquared[j] += v[j] * v[j];
                peak[j] = peak[j].max(u[j].abs());
                vpeak[j] = vpeak[j].max(v[j].abs());
                nonfinite[j] += usize::from(!u[j].is_finite() || !v[j].is_finite());
                local[j] = local[j].max(u[j].abs());
            }
            dot_u += u[0] * u[1];
            dot_v += v[0] * v[1];
            scale_u = scale_u.max(u[0].abs() + u[1].abs());
            scale_v = scale_v.max(v[0].abs() + v[1].abs());
            for (j, sign) in [1.0, -1.0].into_iter().enumerate() {
                let eu = u[0] + sign * u[1];
                let ev = v[0] + sign * v[1];
                let du = u[j + 2] - eu;
                let dv = v[j + 2] - ev;
                residual_u[j] = residual_u[j].max(du.abs());
                residual_v[j] = residual_v[j].max(dv.abs());
                residual2[j] += du * du + dv * dv;
                expected2[j] += eu * eu + ev * ev;
            }
        }
        for j in 0..4 {
            tracks[j].peaks[neuron] = tracks[j].peaks[neuron].max(local[j]);
            if local[j] >= 1e-6 && tracks[j].arrivals[neuron].is_none() {
                tracks[j].arrivals[neuron] = Some(step);
            }
        }
        if neuron == 2184 {
            mid_peak = local;
        }
    }
    Sample {
        squared,
        vsquared,
        peak,
        vpeak,
        nonfinite,
        residual_u,
        residual_v,
        residual2,
        expected2,
        dot_u,
        dot_v,
        scale_u,
        scale_v,
        mid_peak,
    }
}
fn record_metric(
    engine: &Medium,
    sample: &Sample,
    j: usize,
    metrics: &mut impl Write,
    path: &Path,
) -> io::Result<()> {
    let step = engine.state.step;
    let dt = engine.state.dt;
    let Sample {
        squared,
        vsquared,
        peak,
        vpeak,
        nonfinite,
        residual_u,
        residual_v,
        residual2,
        expected2,
        dot_u,
        dot_v,
        mid_peak,
        ..
    } = *sample;
    let overlap = mid_peak[0] >= 1e-6 && mid_peak[1] >= 1e-6;
    let r = j.saturating_sub(2);
    let (ru, rv, l2, cu, cv) = if j >= 2 {
        (
            residual_u[r],
            residual_v[r],
            if expected2[r] > 0.0 {
                (residual2[r] / expected2[r]).sqrt()
            } else {
                residual2[r].sqrt()
            },
            squared[j] - squared[0] - squared[1],
            vsquared[j] - vsquared[0] - vsquared[1],
        )
    } else {
        (0.0, 0.0, 0.0, 0.0, 0.0)
    };
    let s = &engine.state;
    writeln!(
        metrics,
        "{step},{:.17e},{},{:.17e},{:.17e},{:.17e},{:.17e},{},{:.17e},{:.17e},{:.17e},{ru:.17e},{rv:.17e},{l2:.17e},{cu:.17e},{cv:.17e},{dot_u:.17e},{dot_v:.17e},{overlap}",
        number(step) * dt,
        s.arm.name(),
        peak[j],
        vpeak[j],
        squared[j],
        vsquared[j],
        nonfinite[j],
        s.u[2184 * 64 + 27],
        s.v[2184 * 64 + 27],
        mid_peak[j]
    )?;
    if nonfinite[j] > 0 || peak[j] > 100.0 || vpeak[j] > 100.0 {
        metrics.flush()?;
        fs::write(
            path.join("INVALID"),
            format!("numeric screen failed step{step} arm{j}"),
        )?;
        return Err(io::Error::other(
            "0004 numerical screen failed; evidence retained",
        ));
    }
    Ok(())
}
fn group(
    id: usize,
    seed: u64,
    configuration: Configuration,
    half: bool,
    folder: &Path,
    ledger: &mut impl Write,
    hashes: &mut impl Write,
) -> io::Result<()> {
    let factor = if half { 2 } else { 1 };
    let dt = if half { 0.05 } else { 0.1 };
    let steps = 1024 * factor;
    let path = folder.join(format!("group-{id:02}"));
    fs::create_dir(&path)?;
    let arms = [Arm::A, Arm::B, Arm::Plus, Arm::Minus];
    let mut engines: Vec<_> = arms
        .iter()
        .map(|&a| Medium::new(seed, configuration, dt, a))
        .collect::<io::Result<_>>()?;
    let palette = engines[0].state.palette.clone();
    for (i, arm) in arms.iter().enumerate() {
        fs::create_dir(path.join(i.to_string()))?;
        writeln!(
            ledger,
            "{id},{},{seed},{},{dt},{steps},group-{id:02}/{i}",
            arm.name(),
            configuration.name()
        )?;
    }
    let mut tracks: Vec<_> = (0..4).map(|_| Track::new()).collect();
    let mut metrics = BufWriter::new(File::create(path.join("metrics.csv"))?);
    writeln!(
        metrics,
        "step,time,arm,peak,velocity_peak,sum_u2,sum_v2,nonfinite,mid_u,mid_v,mid_peak,residual_u,residual_v,relative_l2,cross_u,cross_v,dot_u,dot_v,overlap"
    )?;
    let mut first_overlap = None;
    for step in 0..=steps {
        let sample = measure(&engines, &mut tracks, step);
        let Sample {
            residual_u,
            residual_v,
            scale_u,
            scale_v,
            mid_peak,
            ..
        } = sample;
        let overlap = mid_peak[0] >= 1e-6 && mid_peak[1] >= 1e-6;
        if overlap && first_overlap.is_none() {
            first_overlap = Some(step);
        }
        for j in 0..4 {
            if engines[j].state.palette != palette {
                return Err(io::Error::other("0004 persistent palette changed"));
            }
            record_metric(&engines[j], &sample, j, &mut metrics, &path)?;
            let s = &engines[j].state;
            if step == 160 * factor {
                tracks[j].at16 = tracks[j].arrivals.iter().flatten().count();
                let track = &mut tracks[j];
                track.peaks16.clone_from(&track.peaks);
            }
            if CHECKPOINTS.contains(&(step / factor)) && step % factor == 0 {
                let relative = format!("group-{id:02}/{j}/frame-{step:06}.q4f");
                save(s, &folder.join(&relative), hashes, &relative)?;
            }
        }
        for j in 0..2 {
            if residual_u[j] > 1e-10 * scale_u || residual_v[j] > 1e-10 * scale_v {
                fs::write(
                    path.join("SUPERPOSITION_FAILURE"),
                    format!("step{step} sign{j}"),
                )?;
            }
        }
        if step < steps {
            for engine in &mut engines {
                engine.advance();
            }
        }
    }
    finish_group(&path, &tracks, dt, first_overlap)?;
    eprintln!(
        "0004 group {id} seed{seed} {} dt{dt} complete",
        configuration.name()
    );
    Ok(())
}
fn finish_group(
    path: &Path,
    tracks: &[Track],
    dt: f64,
    first_overlap: Option<u64>,
) -> io::Result<()> {
    for (j, track) in tracks.iter().enumerate() {
        finish_track(track, &path.join(j.to_string()), dt)?;
    }
    fs::write(
        path.join("overlap.csv"),
        format!(
            "step,time\n{},{}\n",
            first_overlap.map_or(String::new(), |x| x.to_string()),
            first_overlap.map_or(String::new(), |x| format!("{:.17e}", number(x) * dt))
        ),
    )?;
    fs::write(
        path.join("COMPLETE"),
        "Measured all registered steps; scientific decisions separate.\n",
    )?;
    Ok(())
}
fn controls(folder: &Path, ledger: &mut impl Write, hashes: &mut impl Write) -> io::Result<()> {
    for (j, dt) in [0.1, 0.05].into_iter().enumerate() {
        let path = folder.join(format!("zero-{j}"));
        fs::create_dir(&path)?;
        let mut m = Medium::new(7, Configuration::Heterogeneous, dt, Arm::Zero)?;
        let factor = if j == 0 { 1 } else { 2 };
        let steps = 1024 * factor;
        writeln!(
            ledger,
            "zero-{j},zero,7,heterogeneous,{dt},{steps},zero-{j}"
        )?;
        for step in 0..=steps {
            if m.state.u.iter().chain(&m.state.v).any(|x| x.to_bits() != 0) {
                return Err(io::Error::other("zero drift"));
            }
            if CHECKPOINTS.contains(&(step / factor)) && step % factor == 0 {
                let relative = format!("zero-{j}/frame-{step:06}.q4f");
                save(&m.state, &folder.join(&relative), hashes, &relative)?;
            }
            if step < steps {
                m.advance();
            }
        }
    }
    let mut model = Medium::new(7, Configuration::Heterogeneous, 0.1, Arm::Regression)?;
    let mut old = Lattice::new(Case::default())?;
    let mut out = BufWriter::new(File::create(folder.join("regression.csv"))?);
    writeln!(
        out,
        "step,new_source_u,old_source_u,new_source_v,old_source_v,whole_observation_equal"
    )?;
    writeln!(
        ledger,
        "regression,regression,7,heterogeneous,0.1,160,regression.csv"
    )?;
    for step in 0..=160 {
        let mut observed = model.state.observation();
        observed.case = old.case();
        let previous = old.observe();
        let equal = observed == previous;
        let i = 2184 * 64 + 27;
        writeln!(
            out,
            "{step},{:.17e},{:.17e},{:.17e},{:.17e},{equal}",
            model.state.u[i],
            previous.neurons[2184].module27[0],
            model.state.v[i],
            previous.neurons[2184].velocity27[0]
        )?;
        if !equal {
            return Err(io::Error::other("0003 regression mismatch"));
        }
        if step < 160 {
            model.advance();
            old.advance();
        }
    }
    Ok(())
}
/// Execute the preregistered 35 executions into a fresh directory.
///
/// # Errors
/// Rejects existing folders and propagates I/O, boundedness, invariance and regression failures.
pub fn run(folder: &Path) -> io::Result<()> {
    fs::create_dir(folder)?;
    fs::write(
        folder.join("registration.md"),
        include_str!("../EXPERIMENT_0004.md"),
    )?;
    fs::write(
        folder.join("engine.rs"),
        include_str!("propagation_0004.rs"),
    )?;
    fs::write(folder.join("runner.rs"), include_str!("experiment_0004.rs"))?;
    let mut ledger = BufWriter::new(File::create(folder.join("cases.csv"))?);
    let mut hashes = BufWriter::new(File::create(folder.join("hashes.csv"))?);
    writeln!(ledger, "group,arm,seed,configuration,dt,steps,folder")?;
    writeln!(hashes, "file,fnv1a")?;
    let mut id = 0;
    for seed in [7, 19, 41] {
        for cfg in [Configuration::Heterogeneous, Configuration::Homogeneous] {
            group(id, seed, cfg, false, folder, &mut ledger, &mut hashes)?;
            id += 1;
        }
    }
    for cfg in [Configuration::Heterogeneous, Configuration::Homogeneous] {
        group(id, 7, cfg, true, folder, &mut ledger, &mut hashes)?;
        id += 1;
    }
    controls(folder, &mut ledger, &mut hashes)?;
    ledger.flush()?;
    hashes.flush()?;
    fs::write(
        folder.join("COMPLETE"),
        "All 35 registered executions completed; no memory claim.\n",
    )
}
