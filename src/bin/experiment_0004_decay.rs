//! Run the 3072-tick, every-tick frozen-medium observation extension.
use std::path::PathBuf;
fn main() -> std::io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let folder = args.next().map_or_else(
        || PathBuf::from("results/experiment_0004-decay-a"),
        PathBuf::from,
    );
    let reference = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err(std::io::Error::other(
            "usage: experiment_0004_decay [fresh-output-directory] [original-arm-directory]",
        ));
    }
    if let Some(parent) = folder.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    qr_neural_memory_system::decay_0004::run(&folder, reference.as_deref())
}
