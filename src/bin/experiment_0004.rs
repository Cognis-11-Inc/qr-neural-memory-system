//! Execute the registered frozen-medium dual-source experiment.
use std::path::PathBuf;
fn main() -> std::io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .map_or_else(|| PathBuf::from("results/experiment_0004-a"), PathBuf::from);
    if args.next().is_some() {
        return Err(std::io::Error::other(
            "usage: experiment_0004 [fresh-output-directory]",
        ));
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    qr_neural_memory_system::experiment_0004::run(&path)
}
