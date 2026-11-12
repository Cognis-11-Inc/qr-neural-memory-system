//! Run the separately registered Experiment 0002 T/R suite.

use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let folder = args
        .next()
        .map_or_else(|| PathBuf::from("results/experiment_0002-a"), PathBuf::from);
    if args.next().is_some() {
        return Err(std::io::Error::other(
            "usage: experiment_0002 [fresh-output-directory]",
        ));
    }
    if let Some(parent) = folder.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    qr_neural_memory_system::experiment_0002::run(&folder)
}
