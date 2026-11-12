//! Execute the registered experiment and export its evidence to a fresh directory.

use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let mut arguments = std::env::args_os().skip(1);
    let output = arguments
        .next()
        .map_or_else(|| PathBuf::from("results/experiment_0001"), PathBuf::from);
    if arguments.next().is_some() {
        return Err(std::io::Error::other(
            "usage: experiment_0001 [fresh-output-directory]",
        ));
    }
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    qr_neural_memory_system::experiment::run(&output)
}
