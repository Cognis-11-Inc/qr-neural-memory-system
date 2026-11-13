//! Run the registered volumetric diagnostics and bounded observation recorder.
use std::path::PathBuf;
fn main() -> std::io::Result<()> {
    let mut arguments = std::env::args_os().skip(1);
    let folder = arguments
        .next()
        .map_or_else(|| PathBuf::from("results/experiment_0003-a"), PathBuf::from);
    if arguments.next().is_some() {
        return Err(std::io::Error::other(
            "usage: experiment_0003 [fresh-output-directory]",
        ));
    }
    if let Some(parent) = folder.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    qr_neural_memory_system::experiment_0003::run(&folder)
}
