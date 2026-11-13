//! Deterministic supplemental replay exporting the exact morphology-switch fields.
use qr_neural_memory_system::{lattice::Configuration, morphic_0005::Medium};
use std::{env, fs, io, path::Path};
fn audit(root: &Path) -> io::Result<()> {
    if !fs::read_to_string(root.join("NUMERICAL_GATES.csv"))?.contains("\ntrue,") {
        return Err(io::Error::other("short-window numerical gate required"));
    }
    let output = root.join("transition-audit");
    fs::create_dir(&output)?;
    for dt in [0.05, 0.025] {
        for mode in ["local", "time"] {
            let id = format!("c-{mode}-{dt}");
            let folder = output.join(&id);
            fs::create_dir(&folder)?;
            let mut medium = Medium::new(7, Configuration::Heterogeneous, dt, "A", &id)?;
            loop {
                let time =
                    f64::from(u32::try_from(medium.state.tick).map_err(io::Error::other)?) * dt;
                if time >= 8. - 1e-9 {
                    break;
                }
                medium.advance(0., true)?;
            }
            fs::write(folder.join("pre.q5f"), medium.state.encode()?)?;
            let history: Vec<u8> = medium
                .history()
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect();
            if history != fs::read(root.join(&id).join("history.bin"))? {
                return Err(io::Error::other("history replay mismatch"));
            }
            medium.adapt(mode);
            fs::write(folder.join("post.q5f"), medium.state.encode()?)?;
            loop {
                let time =
                    f64::from(u32::try_from(medium.state.tick).map_err(io::Error::other)?) * dt;
                if time >= 16. - 1e-9 {
                    break;
                }
                medium.advance(0., false)?;
            }
            let original = root.join(&id).join(format!("{:06}.q5f", medium.state.tick));
            if medium.state.encode()? != fs::read(original)? {
                return Err(io::Error::other("post-switch full-field replay mismatch"));
            }
            fs::write(
                folder.join("COMPLETE"),
                "Local history and time16 full checkpoint match primary bytes; pre/post fields are supplemental deterministic replay.\n",
            )?;
            println!("{id}: exact history and time16 field replay");
        }
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let root = env::args()
        .nth(1)
        .ok_or_else(|| io::Error::other("usage: experiment_0005_transition_audit OUTPUT"))?;
    audit(Path::new(&root))
}
