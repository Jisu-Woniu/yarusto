use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The input file (.zip) path
    #[arg(default_value = ".")]
    pub input_path: PathBuf,
    /// The output file (.tar) path
    #[arg(short, long, default_value = "./out")]
    pub output_path: PathBuf,
}
