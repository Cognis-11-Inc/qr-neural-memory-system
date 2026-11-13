//! Deterministic serial registered Experiment 0006 trial runner.
use qr_neural_memory_system::{
    lattice::Configuration as C,
    morphic_0006::{FIELDS, Medium, PROBES, variant},
};
use std::{
    env,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
    time::Instant,
};
const TIMES: [f64; 12] = [
    0., 0.8, 3.2, 8., 16., 32., 51.2, 102.4, 153.6, 204.8, 256., 307.2,
];
const EARLY: [usize; 6] = [0, 2182, 2183, 2184, 2186, 2191];
struct Trial<'a> {
    id: &'a str,
    seed: u64,
    config: C,
    dt: f64,
    module: usize,
    arm: &'a str,
    budget: &'a str,
    rule: Option<u8>,
    end: f64,
    early: bool,
}
fn binary_floats(path: &Path, values: &[f64]) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    for value in values {
        writer.write_all(&value.to_le_bytes())?;
    }
    writer.flush()
}
#[allow(clippy::too_many_lines)] // One explicit observation loop fixes timing of all measurements.
fn run(
    root: &Path,
    trial: &Trial<'_>,
    palette: Option<Vec<[u8; 64]>>,
    target: Option<f64>,
) -> io::Result<()> {
    let folder = root.join(trial.id);
    fs::create_dir(&folder)?;
    let start = Instant::now();
    let mut medium = Medium::new(
        trial.seed,
        trial.config,
        trial.dt,
        trial.arm,
        trial.module,
        trial.id,
    )?;
    if let Some(p) = palette {
        medium.apply_palette(p)?;
        medium.reset_impulse(trial.module, trial.arm, 1.)?;
    }
    let before = medium.state.palette.clone();
    let amplitude = if trial.budget == "E1" {
        medium.match_energy(1.)?
    } else if let Some(energy) = target {
        medium.match_energy(energy)?
    } else if trial.arm == "zero" {
        0.
    } else {
        1.
    };
    let initial = medium.energy().total();
    fs::write(folder.join("before.palette"), before.concat())?;
    fs::write(
        folder.join("parameters.csv"),
        format!(
            "id,seed,configuration,dt,module,arm,budget,rule,end,amplitude,initial_energy\n{},{},{},{},{},{},{},{},{},{amplitude:.17e},{initial:.17e}\n",
            trial.id,
            trial.seed,
            trial.config.name(),
            trial.dt,
            trial.module,
            trial.arm,
            trial.budget,
            trial.rule.map_or_else(|| "OFF".into(), |r| r.to_string()),
            trial.end
        ),
    )?;
    let mut global = BufWriter::new(File::create(folder.join("metrics.csv"))?);
    writeln!(
        global,
        "tick,time,kinetic,onsite,coupling,loss,switch_work,drift,peak,velocity_peak,nonfinite"
    )?;
    let mut probes = BufWriter::new(File::create(folder.join("probes.csv"))?);
    writeln!(
        probes,
        "tick,time,id,u_source,v_source,u9,v9,u27,v27,energy,net_external_power"
    )?;
    let mut early = if trial.early {
        let mut file = BufWriter::new(File::create(folder.join("early.q6p"))?);
        file.write_all(b"QR6PRB01")?;
        Some(file)
    } else {
        None
    };
    let mut switched = false;
    loop {
        let tick = medium.state.tick;
        let time = f64::from(u32::try_from(tick).map_err(io::Error::other)?) * trial.dt;
        let energy = medium.energy();
        let peak = medium.state.u.iter().map(|x| x.abs()).fold(0., f64::max);
        let vmax = medium.state.v.iter().map(|x| x.abs()).fold(0., f64::max);
        let drift = energy.total() - initial + medium.loss - medium.switch_work;
        writeln!(
            global,
            "{tick},{time:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{drift:.17e},{peak:.17e},{vmax:.17e},0",
            energy.kinetic, energy.onsite, energy.coupling, medium.loss, medium.switch_work
        )?;
        for id in PROBES {
            let offset = id * 64;
            writeln!(
                probes,
                "{tick},{time:.17e},{id},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                medium.state.u[offset + trial.module],
                medium.state.v[offset + trial.module],
                medium.state.u[offset + 9],
                medium.state.v[offset + 9],
                medium.state.u[offset + 27],
                medium.state.v[offset + 27],
                energy.local[id],
                energy.flux[id]
            )?;
        }
        if let Some(writer) = &mut early
            && time <= 16. + 1e-9
        {
            writer.write_all(&tick.to_le_bytes())?;
            for id in EARLY {
                for m in 0..64 {
                    writer.write_all(&medium.state.u[id * 64 + m].to_le_bytes())?;
                    writer.write_all(&medium.state.v[id * 64 + m].to_le_bytes())?;
                }
            }
        }
        if TIMES.iter().any(|&t| (time - t).abs() < 1e-9) {
            let bytes = medium.state.encode()?;
            fs::write(folder.join(format!("{tick:06}.q6f")), bytes)?;
            if trial.end > 32. && time >= 32. {
                println!("{} checkpoint {time}", trial.id);
            }
        }
        if time >= trial.end - 1e-9 {
            break;
        }
        medium.advance(0., trial.early && time < 8. - 1e-9)?;
        if trial.early && !switched && (time + trial.dt - 8.).abs() < 1e-9 {
            let scores = medium.scores();
            binary_floats(&folder.join("velocity-history.bin"), medium.history())?;
            binary_floats(&folder.join("shape-history.bin"), medium.shape_history())?;
            binary_floats(&folder.join("scores.bin"), &scores)?;
            let switch = folder.join("switch");
            fs::create_dir(&switch)?;
            fs::write(switch.join("pre.q6f"), medium.state.encode()?)?;
            let pre = medium.energy();
            medium.adapt_rule(trial.rule.unwrap_or(0))?;
            let post = medium.energy();
            fs::write(switch.join("post.q6f"), medium.state.encode()?)?;
            let mut audit = BufWriter::new(File::create(switch.join("energy.csv"))?);
            writeln!(audit, "id,pre_energy,post_energy,switch_work")?;
            for id in 0..4096 {
                writeln!(
                    audit,
                    "{id},{:.17e},{:.17e},{:.17e}",
                    pre.local[id],
                    post.local[id],
                    post.local[id] - pre.local[id]
                )?;
            }
            audit.flush()?;
            switched = true;
        }
    }
    fs::write(folder.join("after.palette"), medium.state.palette.concat())?;
    global.flush()?;
    probes.flush()?;
    if let Some(mut writer) = early {
        writer.flush()?;
    }
    fs::write(
        folder.join("COMPLETE"),
        format!("tick={}\n", medium.state.tick),
    )?;
    println!(
        "{} complete {:.2}s",
        trial.id,
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
fn m_id(seed: u64, module: usize, budget: &str, rule: u8, dt: f64) -> String {
    format!("m-s{seed}-m{module}-{budget}-R{rule}-d{dt}")
}
fn palette(path: &Path) -> io::Result<Vec<[u8; 64]>> {
    let bytes = fs::read(path)?;
    if bytes.len() != FIELDS {
        return Err(io::Error::other("palette length"));
    }
    Ok(bytes.as_chunks::<64>().0.to_vec())
}
fn reuse(root: &Path, id: &str, reference: &str) -> io::Result<()> {
    if !root.join(reference).join("COMPLETE").exists() {
        return Err(io::Error::other("missing reuse reference"));
    }
    let folder = root.join(id);
    fs::create_dir(&folder)?;
    fs::write(folder.join("REUSED"), reference)?;
    println!("{id} REUSED {reference}");
    Ok(())
}
#[allow(clippy::too_many_lines)] // Direct transcription of the registered finite matrix.
fn suite(root: &Path, stage: &str) -> io::Result<()> {
    fs::create_dir_all(root)?;
    if ["d", "m", "c"].contains(&stage)
        && !fs::read_to_string(root.join("GATES.csv"))?
            .lines()
            .nth(1)
            .is_some_and(|x| x.starts_with("true,"))
    {
        return Err(io::Error::other("M6-N short gate required"));
    }
    let marker = root.join(format!("STARTED-{stage}"));
    if marker.exists() {
        return Err(io::Error::other(
            "stage already attempted; preserve outputs and use fresh root",
        ));
    }
    fs::write(marker, stage)?;
    let sources = root.join("sources").join(stage);
    fs::create_dir_all(&sources)?;
    fs::write(
        sources.join("morphic_0006.rs"),
        include_str!("../morphic_0006.rs"),
    )?;
    fs::write(
        sources.join("runner.rs"),
        include_str!("experiment_0006.rs"),
    )?;
    if !root.join("RULES.md").exists() {
        fs::write(
            root.join("RULES.md"),
            include_str!("../../EXPERIMENT_0006.md"),
        )?;
    }
    if stage == "n" {
        for (seed, config) in [
            (7, C::Homogeneous),
            (7, C::Heterogeneous),
            (19, C::Heterogeneous),
        ] {
            for dt in [0.05, 0.025, 0.0125] {
                let id = format!("n-{}-s{seed}-d{dt}", config.name());
                run(
                    root,
                    &Trial {
                        id: &id,
                        seed,
                        config,
                        dt,
                        module: 27,
                        arm: "AB+",
                        budget: "UNIT",
                        rule: None,
                        end: 307.2,
                        early: false,
                    },
                    None,
                    None,
                )?;
            }
        }
        for dt in [0.05, 0.025, 0.0125] {
            let id = format!("n-zero-d{dt}");
            run(
                root,
                &Trial {
                    id: &id,
                    seed: 7,
                    config: C::Heterogeneous,
                    dt,
                    module: 27,
                    arm: "zero",
                    budget: "UNIT",
                    rule: None,
                    end: 32.,
                    early: false,
                },
                None,
                None,
            )?;
        }
    } else if stage == "d" {
        for seed in [7, 19] {
            for module in [27, 9] {
                for budget in ["UNIT", "E1"] {
                    let id = format!("d-s{seed}-m{module}-{budget}");
                    run(
                        root,
                        &Trial {
                            id: &id,
                            seed,
                            config: C::Heterogeneous,
                            dt: 0.025,
                            module,
                            arm: "A",
                            budget,
                            rule: None,
                            end: 16.,
                            early: true,
                        },
                        None,
                        None,
                    )?;
                }
            }
            let id = format!("d-s{seed}-zero");
            run(
                root,
                &Trial {
                    id: &id,
                    seed,
                    config: C::Heterogeneous,
                    dt: 0.025,
                    module: 27,
                    arm: "zero",
                    budget: "UNIT",
                    rule: None,
                    end: 16.,
                    early: true,
                },
                None,
                None,
            )?;
        }
    } else if stage == "m" || stage == "repeat" {
        for seed in [7, 19] {
            for module in [27, 9] {
                for budget in ["UNIT", "E1"] {
                    for rule in 0..4 {
                        if stage == "repeat"
                            && (seed != 19 || module != 9 || budget != "E1" || rule != 3)
                        {
                            continue;
                        }
                        let id = m_id(seed, module, budget, rule, 0.025);
                        run(
                            root,
                            &Trial {
                                id: &id,
                                seed,
                                config: C::Heterogeneous,
                                dt: 0.025,
                                module,
                                arm: "A",
                                budget,
                                rule: Some(rule),
                                end: 32.,
                                early: true,
                            },
                            None,
                            None,
                        )?;
                    }
                }
            }
        }
        if stage == "m" {
            for seed in [7, 19] {
                for rule in 0..4 {
                    let id = format!("m-s{seed}-zero-R{rule}");
                    run(
                        root,
                        &Trial {
                            id: &id,
                            seed,
                            config: C::Heterogeneous,
                            dt: 0.025,
                            module: 27,
                            arm: "zero",
                            budget: "UNIT",
                            rule: Some(rule),
                            end: 32.,
                            early: true,
                        },
                        None,
                        None,
                    )?;
                }
                for module in [27, 9] {
                    let id = m_id(seed, module, "E1", 4, 0.025);
                    run(
                        root,
                        &Trial {
                            id: &id,
                            seed,
                            config: C::Heterogeneous,
                            dt: 0.025,
                            module,
                            arm: "A",
                            budget: "E1",
                            rule: Some(4),
                            end: 32.,
                            early: true,
                        },
                        None,
                        None,
                    )?;
                }
            }
            for (seed, module) in [(7, 27), (19, 9)] {
                for rule in [2, 3] {
                    let id = m_id(seed, module, "E1", rule, 0.0125);
                    run(
                        root,
                        &Trial {
                            id: &id,
                            seed,
                            config: C::Heterogeneous,
                            dt: 0.0125,
                            module,
                            arm: "A",
                            budget: "E1",
                            rule: Some(rule),
                            end: 32.,
                            early: true,
                        },
                        None,
                        None,
                    )?;
                }
            }
        }
    } else if stage == "c" {
        for dt in [0.025_f64, 0.0125] {
            for seed in [7, 19] {
                for module in [27, 9] {
                    if dt.to_bits() == 0.0125_f64.to_bits()
                        && ![(7, 27), (19, 9)].contains(&(seed, module))
                    {
                        continue;
                    }
                    let mut virgin_reference: Option<String> = None;
                    for rule in [2, 3] {
                        let exposure = root.join(m_id(seed, module, "E1", rule, dt));
                        let before = palette(&exposure.join("before.palette"))?;
                        let after = palette(&exposure.join("after.palette"))?;
                        let base = Medium::new(
                            seed,
                            C::Heterogeneous,
                            dt,
                            "A",
                            module,
                            "energy-reference",
                        )?;
                        let reference_energy = base.energy().total();
                        for kind in [
                            "VIRGIN",
                            "SOURCE_ONLY",
                            "OFF_SOURCE_ONLY",
                            "ALL",
                            "SHUFFLE_POSITION",
                        ] {
                            for budget in ["UNIT", "MATCH"] {
                                let id =
                                    format!("c-s{seed}-m{module}-R{rule}-d{dt}-{kind}-{budget}");
                                if kind == "VIRGIN" {
                                    if dt.to_bits() == 0.025_f64.to_bits() {
                                        reuse(root, &id, &m_id(seed, module, "UNIT", 0, dt))?;
                                        continue;
                                    }
                                    if let Some(reference) = &virgin_reference {
                                        reuse(root, &id, reference)?;
                                        continue;
                                    }
                                    virgin_reference = Some(id.clone());
                                }
                                let p = variant(&before, &after, kind, rule, seed)?;
                                run(
                                    root,
                                    &Trial {
                                        id: &id,
                                        seed,
                                        config: C::Heterogeneous,
                                        dt,
                                        module,
                                        arm: "A",
                                        budget,
                                        rule: None,
                                        end: 32.,
                                        early: false,
                                    },
                                    Some(p),
                                    if budget == "MATCH" {
                                        Some(reference_energy)
                                    } else {
                                        None
                                    },
                                )?;
                            }
                        }
                    }
                }
            }
        }
    } else {
        return Err(io::Error::other("stage must be n,d,m,c,repeat"));
    }
    fs::write(
        root.join(format!("FINISHED-{stage}")),
        "all stage entries attempted without numerical-screen errors\n",
    )?;
    Ok(())
}
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err(io::Error::other("usage: experiment_0006 OUTPUT STAGE"));
    }
    suite(Path::new(&args[1]), &args[2])
}
