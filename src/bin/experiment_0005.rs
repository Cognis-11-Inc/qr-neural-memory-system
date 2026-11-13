//! Registered Experiment 0005 runner; data are streamed to fresh ignored folders.
use qr_neural_memory_system::{
    lattice::Configuration as C,
    morphic_0005::{Medium, PROBES},
    topology::{POPULATION, Topology},
};
use std::{
    env,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
    time::Instant,
};

struct Spec<'a> {
    stage: &'a str,
    seed: u64,
    c: C,
    dt: f64,
    arm: &'a str,
    end: f64,
    sponge: bool,
    undamped: bool,
    plastic: &'a str,
}
#[allow(clippy::too_many_lines)] // One streamed observation loop keeps tick accounting explicit.
fn run(
    root: &Path,
    id: &str,
    s: &Spec<'_>,
    retained: Option<Medium>,
) -> io::Result<Option<Medium>> {
    let path = root.join(id);
    fs::create_dir(&path)?;
    let start = Instant::now();
    let mut m = if let Some(mut m) = retained {
        m.reexpose(id);
        m
    } else {
        Medium::new(s.seed, s.c, s.dt, s.arm, id)?
    };
    m.damping_control(!s.undamped, s.sponge);
    let before = m.state.palette.clone();
    fs::write(path.join("before.palette"), before.concat())?;
    let initial = m.energy().total();
    let mut global = BufWriter::new(File::create(path.join("metrics.csv"))?);
    writeln!(
        global,
        "tick,time,kinetic,onsite,coupling,loss,sponge_loss,work,switch_work,drift,peak,velocity_peak,active,participation,changes"
    )?;
    let mut probes = BufWriter::new(File::create(path.join("probes.csv"))?);
    writeln!(
        probes,
        "tick,time,id,u27,v27,peak,velocity_peak,energy,net_external_power"
    )?;
    let mut peak = vec![0.0_f64; POPULATION];
    let mut energy_peak = vec![0.0_f64; POPULATION];
    let mut first = vec![None; POPULATION];
    let mut crossings = vec![0; POPULATION];
    let mut peaks = vec![0; POPULATION];
    let mut last_peak = vec![-1e6; POPULATION];
    let mut previous = vec![0.; POPULATION];
    let mut older = vec![0.; POPULATION];
    let mut on = vec![false; POPULATION];
    let mut changes = 0;
    let mut final_envelope = vec![0.; POPULATION];
    loop {
        let tick = m.state.tick;
        let time = f64::from(u32::try_from(tick).expect("bounded tick")) * s.dt;
        let e = m.energy();
        let mut envelope = vec![0.0_f64; POPULATION];
        let mut velocity = vec![0.0_f64; POPULATION];
        for i in 0..POPULATION {
            for j in 0..64 {
                envelope[i] = envelope[i].max(m.state.u[i * 64 + j].abs());
                velocity[i] = velocity[i].max(m.state.v[i * 64 + j].abs());
            }
        }
        for i in 0..POPULATION {
            peak[i] = peak[i].max(envelope[i]);
            energy_peak[i] = energy_peak[i].max(e.local[i]);
            let active = envelope[i] >= 1e-6;
            if active && !on[i] {
                crossings[i] += 1;
                if first[i].is_none() {
                    first[i] = Some(time);
                }
            }
            on[i] = active;
            if tick >= 2
                && previous[i] > older[i]
                && previous[i] > envelope[i]
                && previous[i] >= 1e-6
                && time - s.dt - last_peak[i] >= 1.0 - 1e-12
            {
                peaks[i] += 1;
                last_peak[i] = time - s.dt;
            }
        }
        let max = envelope.iter().copied().fold(0., f64::max);
        let vmax = velocity.iter().copied().fold(0., f64::max);
        let total = e.total();
        let square = e.local.iter().map(|x| x * x).sum::<f64>();
        let participation = if square > 0. {
            total * total / square
        } else {
            0.
        };
        let drift = total - initial + m.loss + m.boundary_loss - m.work - m.switch_work;
        writeln!(
            global,
            "{tick},{time:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{drift:.17e},{max:.17e},{vmax:.17e},{},{participation:.17e},{changes}",
            e.kinetic,
            e.onsite,
            e.coupling,
            m.loss,
            m.boundary_loss,
            m.work,
            m.switch_work,
            on.iter().filter(|&&x| x).count()
        )?;
        for id in PROBES {
            writeln!(
                probes,
                "{tick},{time:.17e},{id},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                m.state.u[id * 64 + 27],
                m.state.v[id * 64 + 27],
                envelope[id],
                velocity[id],
                e.local[id],
                e.flux[id]
            )?;
        }
        if [0., 0.8, 3.2, 16., 32., 102.4, 307.2]
            .iter()
            .any(|&t| (time - t).abs() < 1e-9)
        {
            let bytes = m.state.encode()?;
            fs::write(path.join(format!("{tick:06}.q5f")), bytes)?;
        }
        if time >= s.end - 1e-9 {
            final_envelope.copy_from_slice(&envelope);
            break;
        }
        older.copy_from_slice(&previous);
        previous.copy_from_slice(&envelope);
        let history = !s.plastic.is_empty() && time < 8. - 1e-9;
        m.advance(if s.arm == "force" { 0.1 } else { 0. }, history)?;
        if !s.plastic.is_empty() && (time + s.dt - 8.).abs() < 1e-9 {
            let pre = m.energy().total();
            let history = m
                .history()
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<_>>();
            fs::write(path.join("history.bin"), history)?;
            changes = m.adapt(s.plastic);
            fs::write(
                path.join("switch.csv"),
                format!(
                    "time,pre_energy,post_energy,work,changes\n8,{pre:.17e},{:.17e},{:.17e},{changes}\n",
                    m.energy().total(),
                    m.switch_work
                ),
            )?;
        }
    }
    let mut spatial = BufWriter::new(File::create(path.join("spatial.csv"))?);
    writeln!(
        spatial,
        "id,x,y,z,hops_A,peak,peak_energy,first_crossing,crossings,separated_peaks,final_peak"
    )?;
    for id in 0..POPULATION {
        let [px, py, pz] = Topology::Cubic6.coordinates(id);
        let arrival = first[id].map_or_else(String::new, |x| format!("{x:.17e}"));
        writeln!(
            spatial,
            "{id},{px},{py},{pz},{},{:.17e},{:.17e},{arrival},{},{},{:.17e}",
            Topology::Cubic6.hops(2182, id),
            peak[id],
            energy_peak[id],
            crossings[id],
            peaks[id],
            final_envelope[id]
        )?;
    }
    fs::write(path.join("after.palette"), m.state.palette.concat())?;
    fs::write(
        path.join("COMPLETE"),
        format!("tick={} changes={changes}\n", m.state.tick),
    )?;
    global.flush()?;
    probes.flush()?;
    spatial.flush()?;
    println!("{id} complete {:.2}s", start.elapsed().as_secs_f64());
    Ok(Some(m))
}
#[allow(clippy::too_many_lines)] // Direct transcription of the frozen condition matrix.
fn suite(root: &Path, stage: &str) -> io::Result<()> {
    fs::create_dir_all(root)?;
    if stage == "b" || stage == "c" {
        let gate = fs::read_to_string(root.join("NUMERICAL_GATES.csv"))?;
        let allowed = if stage == "b" {
            gate.lines().nth(1) == Some("true,true")
        } else {
            gate.lines().nth(1).is_some_and(|x| x.starts_with("true,"))
        };
        if !allowed {
            return Err(io::Error::other(
                "registered numerical gate failed: stage remains unexecuted",
            ));
        }
    }
    if root.join(format!("ledger-{stage}.csv")).exists() {
        return Err(io::Error::other(
            "stage ledger already exists; use a fresh output root",
        ));
    }
    let sources = root.join("sources").join(stage);
    fs::create_dir_all(&sources)?;
    fs::write(
        sources.join("morphic_0005.rs"),
        include_str!("../morphic_0005.rs"),
    )?;
    fs::write(
        sources.join("experiment_0005.rs"),
        include_str!("experiment_0005.rs"),
    )?;
    if !root.join("RULES.md").exists() {
        fs::write(
            root.join("RULES.md"),
            include_str!("../../EXPERIMENT_0005.md"),
        )?;
    }

    let mut ledger = BufWriter::new(File::create(root.join(format!("ledger-{stage}.csv")))?);
    writeln!(
        ledger,
        "id,stage,seed,configuration,dt,arm,end,sponge,undamped,plastic"
    )?;
    let mut execute = |id: &str, s: &Spec<'_>, retained| {
        writeln!(
            ledger,
            "{id},{},{},{},{},{},{},{},{},{}",
            s.stage,
            s.seed,
            s.c.name(),
            s.dt,
            s.arm,
            s.end,
            s.sponge,
            s.undamped,
            s.plastic
        )?;
        ledger.flush()?;
        run(root, id, s, retained)
    };
    if stage == "numerical" || stage == "benchmark" {
        for c in [C::Homogeneous, C::Heterogeneous] {
            for dt in [0.1, 0.05, 0.025] {
                if stage == "benchmark"
                    && (c != C::Homogeneous || f64::to_bits(dt) != 0.1_f64.to_bits())
                {
                    continue;
                }
                let id = format!("n-{}-{dt}", c.name());
                execute(
                    &id,
                    &Spec {
                        stage: "M5-0",
                        seed: 7,
                        c,
                        dt,
                        arm: "AB+",
                        end: if stage == "benchmark" { 3.2 } else { 307.2 },
                        sponge: false,
                        undamped: false,
                        plastic: "",
                    },
                    None,
                )?;
            }
        }
        if stage == "numerical" {
            for arm in ["undamped", "force"] {
                for dt in [0.1, 0.05, 0.025] {
                    execute(
                        &format!("n-{arm}-{dt}"),
                        &Spec {
                            stage: "M5-0",
                            seed: 7,
                            c: C::Homogeneous,
                            dt,
                            arm: if arm == "undamped" { "AB+" } else { "force" },
                            end: if arm == "undamped" { 32. } else { 16. },
                            sponge: false,
                            undamped: arm == "undamped",
                            plastic: "",
                        },
                        None,
                    )?;
                }
            }
        }
    } else if stage == "a" || stage == "repeat" {
        for (seed, c) in [
            (7, C::Homogeneous),
            (7, C::Heterogeneous),
            (19, C::Heterogeneous),
            (7, C::Rearranged),
            (19, C::Rearranged),
        ] {
            for arm in ["A", "B", "AB+", "AB-", "zero"] {
                if stage == "repeat" && (seed != 7 || c != C::Heterogeneous || arm != "AB+") {
                    continue;
                }
                execute(
                    &format!("a-{}-{seed}-{arm}", c.name()),
                    &Spec {
                        stage: "M5-A",
                        seed,
                        c,
                        dt: 0.05,
                        arm,
                        end: 32.,
                        sponge: false,
                        undamped: false,
                        plastic: "",
                    },
                    None,
                )?;
            }
        }
    } else if stage == "b" {
        for (c, arm) in [
            (C::Homogeneous, "AB+"),
            (C::Heterogeneous, "AB+"),
            (C::Heterogeneous, "AB-"),
        ] {
            for sponge in [false, true] {
                let id = format!("b-{}-{arm}-{sponge}", c.name());
                if !sponge && arm == "AB+" {
                    let source = root.join(format!("n-{}-0.05", c.name()));
                    if !source.join("COMPLETE").exists() {
                        return Err(io::Error::other("missing numerical reuse"));
                    }
                    fs::write(
                        root.join(format!("{id}.REUSED")),
                        source.file_name().unwrap_or_default().as_encoded_bytes(),
                    )?;
                } else {
                    execute(
                        &id,
                        &Spec {
                            stage: "M5-B",
                            seed: 7,
                            c,
                            dt: 0.05,
                            arm,
                            end: 307.2,
                            sponge,
                            undamped: false,
                            plastic: "",
                        },
                        None,
                    )?;
                }
            }
        }
    } else if stage == "c" {
        for dt in [0.05, 0.025] {
            for mode in ["frozen", "disabled", "local", "time", "zero"] {
                let id = format!("c-{mode}-{dt}");
                let s = Spec {
                    stage: "M5-C",
                    seed: 7,
                    c: C::Heterogeneous,
                    dt,
                    arm: if mode == "zero" { "zero" } else { "A" },
                    end: if ["frozen", "disabled"].contains(&mode) {
                        32.
                    } else {
                        16.
                    },
                    sponge: false,
                    undamped: false,
                    plastic: match mode {
                        "frozen" => "",
                        "zero" => "local",
                        _ => mode,
                    },
                };
                let m = execute(&id, &s, None)?;
                if ["local", "time"].contains(&mode) {
                    execute(
                        &format!("c-reexpose-{mode}-{dt}"),
                        &Spec {
                            plastic: "",
                            end: 32.,
                            ..s
                        },
                        m,
                    )?;
                }
            }
        }
    } else {
        return Err(io::Error::other(
            "stage must be numerical,a,b,c,repeat,benchmark",
        ));
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err(io::Error::other("usage: experiment_0005 OUTPUT STAGE"));
    }
    suite(Path::new(&args[1]), &args[2])
}
